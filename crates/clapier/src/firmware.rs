//! Firmware images a rabbit can fetch and flash on its usual network
//! (garenne's `fw` verb): kept raw under the overlay's
//! `common/vl/fw/`, where every rabbit can ask for them.
//!
//! The rabbit refuses whatever it cannot verify, and it verifies
//! against what this side announces: the length and the Adler-32 are
//! computed here, from the very file that will be served.

use std::path::{Path, PathBuf};

/// A firmware's size: more than a stub, and below the config sector.
pub const MIN: usize = 0x1000;
pub const MAX: usize = 0x1F000;

/// Where the images live, as the rabbits ask for them.
pub const URL_DIR: &str = "/vl/fw";

pub fn dir(overlay: &Path) -> PathBuf {
    overlay.join("common/vl/fw")
}

/// Why an upload or an image was refused; the code is the contract.
#[derive(Debug, PartialEq, Eq)]
pub enum Rejected {
    /// Not a `.sim` (the `-violet-` frame) nor a raw image.
    Shape,
    /// Decodes to something that is not a firmware's size.
    Size,
    /// Does not open on the five ARM vectors every firmware carries.
    Vectors,
    /// A file name the overlay would not keep.
    Name,
}

impl Rejected {
    pub fn code(&self) -> &'static str {
        match self {
            Rejected::Shape => "fw_shape",
            Rejected::Size => "fw_size",
            Rejected::Vectors => "fw_vectors",
            Rejected::Name => "fw_name",
        }
    }
}

/// Adler-32, the checksum garenne recomputes before it flashes.
pub fn adler32(data: &[u8]) -> u32 {
    let (mut a, mut b) = (1u32, 0u32);
    for &byte in data {
        a = (a + u32::from(byte)) % 65521;
        b = (b + a) % 65521;
    }
    (b << 16) | a
}

/// Reset, undef, swi and the two aborts: five `ldr pc, [pc, #n]`.
fn vectors_ok(bin: &[u8]) -> bool {
    bin.len() >= 32
        && (0..5).all(|i| {
            bin[4 * i + 3] == 0xE5 && bin[4 * i + 2] == 0x9F && bin[4 * i + 1] & 0xF0 == 0xF0
        })
}

/// The firmware inside a `.sim`: `-violet-`, eight hex digits of
/// length, the payload in hex under the Violet cipher, `-violet-`.
fn decode_sim(sim: &[u8]) -> Option<Vec<u8>> {
    let inner = sim.strip_prefix(b"-violet-")?.strip_suffix(b"-violet-")?;
    let (len, hex) = inner.split_at_checked(8)?;
    let len = usize::from_str_radix(std::str::from_utf8(len).ok()?, 16).ok()?;
    if len != hex.len() || !len.is_multiple_of(2) {
        return None;
    }
    let mut key: u8 = 0x47;
    let mut bin = Vec::with_capacity(len / 2);
    for pair in hex.chunks(2) {
        let byte = u8::from_str_radix(std::str::from_utf8(pair).ok()?, 16).ok()?;
        let plain = byte.wrapping_sub(47).wrapping_mul(key);
        bin.push(plain);
        key = plain.wrapping_add(plain).wrapping_add(1);
    }
    Some(bin)
}

/// What an upload holds, as the raw image to serve: a `.sim` is
/// decoded, a raw image is taken as it is, anything else is refused.
pub fn image(upload: &[u8]) -> Result<Vec<u8>, Rejected> {
    let bin = if upload.starts_with(b"-violet-") {
        decode_sim(upload).ok_or(Rejected::Shape)?
    } else {
        upload.to_vec()
    };
    check(&bin)?;
    Ok(bin)
}

/// A raw image a rabbit may be handed.
pub fn check(bin: &[u8]) -> Result<(), Rejected> {
    if bin.len() < MIN || bin.len() > MAX {
        return Err(Rejected::Size);
    }
    if !vectors_ok(bin) {
        return Err(Rejected::Vectors);
    }
    Ok(())
}

/// The name an image is kept under: the upload's, without its folder
/// and extension, reduced to what a URL carries untouched.
pub fn file_name(upload_name: &str) -> Result<String, Rejected> {
    let base = upload_name.rsplit(['/', '\\']).next().unwrap_or_default();
    let stem = base
        .strip_suffix(".sim")
        .or_else(|| base.strip_suffix(".bin"))
        .unwrap_or(base);
    let ok = !stem.is_empty()
        && stem.len() <= 48
        && !stem.starts_with('.')
        && stem
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'));
    if ok {
        Ok(format!("{stem}.bin"))
    } else {
        Err(Rejected::Name)
    }
}

/// A listed image's file name, back from the web app: one of ours,
/// never a path.
pub fn listed_name(name: &str) -> Option<&str> {
    (file_name(name).ok()?.as_str() == name).then_some(name)
}

/// The verb's arguments for this image: `/vl/fw/NAME LEN ADLER32`.
pub fn verb_args(name: &str, bin: &[u8]) -> String {
    format!("{URL_DIR}/{name} {} {:08x}", bin.len(), adler32(bin))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn firmware(len: usize) -> Vec<u8> {
        let mut bin = vec![0u8; len];
        for i in 0..5 {
            bin[4 * i..4 * i + 4].copy_from_slice(&[0x18, 0xF0, 0x9F, 0xE5]);
        }
        bin
    }

    fn sim_of(bin: &[u8]) -> Vec<u8> {
        let inv = |k: u8| (0..=255u8).find(|x| x.wrapping_mul(k) == 1).unwrap();
        let mut key = 0x47u8;
        let mut hex = String::new();
        for &v in bin {
            hex += &format!("{:02x}", 47u8.wrapping_add(v.wrapping_mul(inv(key))));
            key = v.wrapping_add(v).wrapping_add(1);
        }
        format!("-violet-{:08x}{hex}-violet-", hex.len()).into_bytes()
    }

    #[test]
    fn adler32_matches_the_reference() {
        assert_eq!(adler32(b"Wikipedia"), 0x11E6_0398);
    }

    #[test]
    fn a_sim_and_a_raw_image_give_the_same_firmware() {
        let bin = firmware(MIN);
        assert_eq!(image(&sim_of(&bin)).as_deref(), Ok(bin.as_slice()));
        assert_eq!(image(&bin).as_deref(), Ok(bin.as_slice()));
    }

    #[test]
    fn what_is_not_a_firmware_is_refused() {
        assert_eq!(image(&firmware(MIN - 1)), Err(Rejected::Size));
        assert_eq!(image(&firmware(MAX + 1)), Err(Rejected::Size));
        assert_eq!(image(&vec![b'x'; MIN]), Err(Rejected::Vectors));
        let mut cut = sim_of(&firmware(MIN));
        cut.truncate(cut.len() - 9);
        assert_eq!(image(&cut), Err(Rejected::Shape));
    }

    #[test]
    fn names_stay_inside_the_folder() {
        assert_eq!(file_name("Nab-r2.sim").as_deref(), Ok("Nab-r2.bin"));
        assert_eq!(file_name("C:\\fw\\Nab.bin").as_deref(), Ok("Nab.bin"));
        assert_eq!(file_name("../../etc/passwd"), Ok("passwd.bin".to_string()));
        assert_eq!(file_name(".sim"), Err(Rejected::Name));
        assert_eq!(file_name("a b.sim"), Err(Rejected::Name));
        assert_eq!(listed_name("Nab-r2.bin"), Some("Nab-r2.bin"));
        assert_eq!(listed_name("../Nab-r2.bin"), None);
        assert_eq!(listed_name("Nab-r2"), None);
    }

    #[test]
    fn the_verb_names_what_the_rabbit_must_find() {
        let bin = firmware(MIN);
        assert_eq!(
            verb_args("nab.bin", &bin),
            format!("/vl/fw/nab.bin 4096 {:08x}", adler32(&bin))
        );
    }
}
