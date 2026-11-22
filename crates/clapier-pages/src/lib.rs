//! Human-facing side of the burrow. The rabbit never reads these pages;
//! humans checking on the rabbit do. The web app (`web/`, Svelte) is
//! embedded at build time and served from memory; directory listings
//! are rendered here. Presentation only - no I/O, no state.

use std::fmt::Write as _;

include!(concat!(env!("OUT_DIR"), "/assets.rs"));

/// One file of the embedded web app.
pub struct Asset {
    pub bytes: &'static [u8],
    pub mime: &'static str,
    /// Vite names what it builds after its content: such a file never
    /// changes under its name and can be cached forever.
    pub immutable: bool,
}

/// The web app's file at `path` (relative to its base, no leading
/// slash), if the build has one.
pub fn asset(path: &str) -> Option<Asset> {
    ASSETS
        .iter()
        .find(|(name, _)| *name == path)
        .map(|(name, bytes)| Asset {
            bytes,
            mime: mime(name),
            immutable: hashed(name),
        })
}

/// `index-Dzi4fA_z.js`: a dash and eight characters of content hash
/// before the extension, which the files we name ourselves never have.
fn hashed(name: &str) -> bool {
    name.rsplit_once('.').is_some_and(|(stem, _)| {
        stem.len() > 9
            && stem.is_char_boundary(stem.len() - 9)
            && stem.as_bytes()[stem.len() - 9] == b'-'
    })
}

/// The web app's shell, served for every page of it.
pub fn index() -> Asset {
    asset("index.html").expect("the build always embeds an index.html")
}

fn mime(name: &str) -> &'static str {
    match name.rsplit_once('.').map(|(_, ext)| ext) {
        Some("html") => "text/html; charset=utf-8",
        Some("js") => "text/javascript; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("ico") => "image/x-icon",
        Some("woff2") => "font/woff2",
        Some("json" | "webmanifest") => "application/json",
        Some("txt") => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}

pub fn render_listing(path: &str, entries: &[String]) -> String {
    let title = escape(path);
    let mut page = format!(
        "<!doctype html><html lang=\"en\"><meta charset=\"utf-8\"><title>{title}</title>\
<body><h1>{title}</h1><ul>"
    );
    for entry in entries {
        let esc = escape(entry);
        let _ = write!(page, "<li><a href=\"{esc}\">{esc}</a></li>");
    }
    page.push_str("</ul></body></html>");
    page
}

pub fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_neutralizes_html() {
        assert_eq!(
            escape("<a href=\"x\">&"),
            "&lt;a href=&quot;x&quot;&gt;&amp;"
        );
    }

    #[test]
    fn listing_escapes_entries() {
        let page = render_listing("/vl/", &["<script>".to_string()]);
        assert!(!page.contains("<script>"));
    }

    #[test]
    fn the_shell_is_always_there() {
        let index = index();
        assert_eq!(index.mime, "text/html; charset=utf-8");
        assert!(!index.immutable);
        assert!(asset("../Cargo.toml").is_none());
    }

    #[test]
    fn hashed_assets_are_immutable() {
        assert_eq!(
            mime("assets/index-abc.js"),
            "text/javascript; charset=utf-8"
        );
        assert_eq!(mime("fonts/CalSansVF.woff2"), "font/woff2");
    }
}
