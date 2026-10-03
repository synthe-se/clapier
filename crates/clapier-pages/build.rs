//! Embeds the built web app (`web/dist`, made by `bun run build` in
//! `web/`) into the binary, so one file still carries the whole burrow.
//! Without a build the table holds only a page saying how to make one:
//! the server, its tests and the rabbit's file service never need Node.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

fn main() {
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("manifest dir"));
    let dist = manifest.join("../../web/dist");
    // A directory is scanned whole; a missing one reruns every build,
    // so a fresh `bun run build` is always picked up.
    println!("cargo:rerun-if-changed={}", dist.display());

    let mut files = Vec::new();
    if dist.join("index.html").is_file() {
        collect(&dist, &dist, &mut files);
    } else {
        println!(
            "cargo:warning=web/dist not found: the burrow's web app is a placeholder (cd web && bun install && bun run build)"
        );
        files.push(("index.html".to_string(), manifest.join("src/missing.html")));
    }
    files.sort();

    let mut out = String::from("pub(crate) static ASSETS: &[(&str, &[u8])] = &[\n");
    for (path, file) in &files {
        let _ = writeln!(
            out,
            "    ({path:?}, include_bytes!({:?})),",
            file.canonicalize()
                .expect("asset path")
                .display()
                .to_string()
        );
    }
    out.push_str("];\n");
    let dest = PathBuf::from(std::env::var("OUT_DIR").expect("out dir")).join("assets.rs");
    std::fs::write(dest, out).expect("write assets.rs");
}

fn collect(root: &Path, dir: &Path, out: &mut Vec<(String, PathBuf)>) {
    for entry in std::fs::read_dir(dir).expect("read web/dist") {
        let path = entry.expect("dist entry").path();
        if path.is_dir() {
            collect(root, &path, out);
        } else {
            let rel = path.strip_prefix(root).expect("under dist");
            let rel = rel
                .components()
                .map(|c| c.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join("/");
            out.push((rel, path));
        }
    }
}
