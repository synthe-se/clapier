//! Composition of the burrow: state, router, request recording.
//!
//! The pieces live in their own crates - `clapier-vl` speaks the rabbit's
//! dialect, `clapier-journal` remembers requests, `clapier-pages` renders
//! for humans. This crate only wires them together.

use std::{
    net::{IpAddr, SocketAddr},
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

use axum::{
    Json, Router,
    body::Body,
    extract::{ConnectInfo, Path, State},
    http::{Method, StatusCode, Uri, header},
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
};
use clapier_fleet::Fleet;
use clapier_journal::{Hit, Journal};
use clapier_pages as pages;
use clapier_vl::ContentTree;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::info;

/// Number of requests kept for the status page.
const HISTORY: usize = 200;

pub struct AppState {
    pub tree: ContentTree,
    pub rabbit: Option<IpAddr>,
    pub fleet: Fleet,
    /// The speech pipeline lives Mac-side (TTS model, encoder); the
    /// pilot page shells out to it when configured.
    pub say_script: Option<PathBuf>,
    /// The garenne bytecode to install for rabbits whose burrow has no
    /// bc.jsp yet (the adoption); unset turns adoption off.
    pub garenne: Option<PathBuf>,
    started: Instant,
    journal: Journal,
}

impl AppState {
    pub fn new(
        base: Option<PathBuf>,
        overlay: Option<PathBuf>,
        rabbit: Option<IpAddr>,
        say_script: Option<PathBuf>,
        garenne: Option<PathBuf>,
    ) -> Arc<Self> {
        Arc::new(Self {
            tree: ContentTree { base, overlay },
            rabbit,
            fleet: Fleet::new(),
            say_script,
            garenne,
            started: Instant::now(),
            journal: Journal::new(HISTORY),
        })
    }

    /// The adoption: install garenne as `bc.jsp` in the rabbit's burrow.
    /// Atomic within the overlay filesystem, so the rabbit never fetches
    /// half a brain. Returns false when adoption is off or impossible.
    fn adopt(&self, mac: &str) -> bool {
        let (Some(bin), Some(overlay)) = (&self.garenne, &self.tree.overlay) else {
            return false;
        };
        let dir = overlay.join("rabbits").join(mac).join("vl");
        let install = || -> std::io::Result<()> {
            std::fs::create_dir_all(&dir)?;
            let tmp = dir.join(".bc.jsp.tmp");
            std::fs::copy(bin, &tmp)?;
            std::fs::rename(&tmp, dir.join("bc.jsp"))
        };
        match install() {
            Ok(()) => {
                info!("🐰 adopted {mac}: garenne installed in its burrow");
                true
            }
            Err(e) => {
                info!("could not adopt {mac}: {e}");
                false
            }
        }
    }
}

/// A name fits on a tile: a word or two, no control characters.
pub const NAME_MAX: usize = 32;

/// The rabbit's name, kept in its burrow (`rabbits/<mac>/name`) so it
/// travels with the overlay like everything else about the rabbit.
fn name_path(overlay: &std::path::Path, mac: &str) -> PathBuf {
    overlay.join("rabbits").join(mac).join("name")
}

fn read_name(overlay: &std::path::Path, mac: &str) -> Option<String> {
    let text = std::fs::read_to_string(name_path(overlay, mac)).ok()?;
    let name = text.lines().next()?.trim();
    (!name.is_empty()).then(|| name.chars().take(NAME_MAX).collect())
}

/// A name as typed, made fit to keep: trimmed, one line, bounded. Empty
/// means "forget the name".
pub fn vet_name(raw: &str) -> Result<String, &'static str> {
    let name = raw.trim();
    if name.chars().count() > NAME_MAX {
        return Err("name_length");
    }
    if name.chars().any(char::is_control) {
        return Err("name_chars");
    }
    Ok(name.to_string())
}

/// Ask a rabbit its `/status` page, in its own dialect: HTTP/1.0, read
/// until the connection closes.
async fn fetch_status(ip: IpAddr) -> anyhow::Result<String> {
    let exchange = async {
        let mut sock = tokio::net::TcpStream::connect((ip, 80)).await?;
        sock.write_all(b"GET /status HTTP/1.0\r\n\r\n").await?;
        let mut buf = Vec::new();
        sock.read_to_end(&mut buf).await?;
        let text = String::from_utf8_lossy(&buf).into_owned();
        Ok(text
            .split_once("\r\n\r\n")
            .map(|(_, body)| body.to_string())
            .unwrap_or_default())
    };
    tokio::time::timeout(Duration::from_secs(8), exchange).await?
}

/// Enrich the fleet with each named rabbit's `/status`, forever. One
/// rabbit at a time and a gentle cadence: the radio is shared and the
/// dashboard is a guest on it.
pub async fn poll_statuses(app: Arc<AppState>, every: Duration) {
    loop {
        tokio::time::sleep(every).await;
        let targets: Vec<IpAddr> = app
            .fleet
            .snapshot()
            .iter()
            .filter(|r| r.mac.is_some())
            .map(|r| r.ip)
            .collect();
        for ip in targets {
            if let Ok(body) = fetch_status(ip).await {
                app.fleet
                    .status(ip, clapier_fleet::parse_status(&body), Instant::now());
            }
        }
    }
}

/// One command over the rabbit's UDP control port, reply awaited.
pub async fn ctl_send(ip: IpAddr, cmd: &str) -> anyhow::Result<String> {
    let sock = tokio::net::UdpSocket::bind(("0.0.0.0", 0)).await?;
    // Broadcast targets are legal here, like the Python tool allowed.
    socket2::SockRef::from(&sock).set_broadcast(true)?;
    sock.send_to(format!("grn1 {cmd}").as_bytes(), (ip, 9998))
        .await?;
    let mut buf = [0u8; 2048];
    let (n, _) = tokio::time::timeout(Duration::from_secs(3), sock.recv_from(&mut buf)).await??;
    Ok(String::from_utf8_lossy(&buf[..n]).into_owned())
}

/// Why the gate refused a command. The web app words each one itself
/// (`code` is the contract), in the person's language.
#[derive(Debug, PartialEq, Eq)]
pub enum Refusal {
    Empty,
    VolFloor,
    VolNumber,
    Ears,
    NotAllowed,
}

impl Refusal {
    pub fn code(&self) -> &'static str {
        match self {
            Refusal::Empty => "cmd_empty",
            Refusal::VolFloor => "vol_floor",
            Refusal::VolNumber => "vol_number",
            Refusal::Ears => "ears_range",
            Refusal::NotAllowed => "cmd_not_allowed",
        }
    }
}

/// The pilot's gate: only commands a button should reach, with their
/// arguments bounded. The volume floor is not a style choice: below 40
/// the amplifier's draw has already crashed a supply and wedged the
/// radio.
pub fn vet_ctl(cmd: &str) -> Result<String, Refusal> {
    let words: Vec<&str> = cmd.split_whitespace().collect();
    match words.as_slice() {
        [] => Err(Refusal::Empty),
        [verb @ ("ping" | "dance" | "reboot" | "stop" | "conf" | "heap" | "fetch")] => {
            Ok((*verb).to_string())
        }
        ["vol", n] => match n.parse::<u32>() {
            Ok(n) if (40..=254).contains(&n) => Ok(format!("vol {n}")),
            Ok(_) => Err(Refusal::VolFloor),
            Err(_) => Err(Refusal::VolNumber),
        },
        ["color", c] if c.len() == 6 && c.chars().all(|ch| ch.is_ascii_hexdigit()) => {
            Ok(format!("color {c}"))
        }
        ["led", i, c]
            if i.len() == 1
                && ('0'..='4').contains(&i.chars().next().unwrap_or('9'))
                && c.len() == 6
                && c.chars().all(|ch| ch.is_ascii_hexdigit()) =>
        {
            Ok(format!("led {i} {c}"))
        }
        ["ears", a, b] => match (a.parse::<u32>(), b.parse::<u32>()) {
            (Ok(a), Ok(b)) if a <= 16 && b <= 16 => Ok(format!("ears {a} {b}")),
            _ => Err(Refusal::Ears),
        },
        [verb @ ("chor" | "play"), path]
            if path.starts_with("/vl/") && !path.contains("..") && path.len() < 128 =>
        {
            Ok(format!("{verb} {path}"))
        }
        _ => Err(Refusal::NotAllowed),
    }
}

fn mac_ok(mac: &str) -> bool {
    mac.len() == 12
        && mac
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_gate_lets_buttons_through() {
        assert_eq!(vet_ctl("ping"), Ok("ping".to_string()));
        assert_eq!(vet_ctl("  dance  "), Ok("dance".to_string()));
        assert_eq!(vet_ctl("vol 40"), Ok("vol 40".to_string()));
        assert_eq!(vet_ctl("color 7c5cff"), Ok("color 7c5cff".to_string()));
        assert_eq!(vet_ctl("ears 8 8"), Ok("ears 8 8".to_string()));
        assert_eq!(
            vet_ctl("chor /vl/config/chor/taichi.chor"),
            Ok("chor /vl/config/chor/taichi.chor".to_string())
        );
    }

    #[test]
    fn the_gate_remembers_the_supply() {
        assert_eq!(vet_ctl("vol 39"), Err(Refusal::VolFloor));
        assert_eq!(vet_ctl("vol 0"), Err(Refusal::VolFloor));
    }

    #[test]
    fn the_gate_stops_everything_else() {
        assert!(vet_ctl("").is_err());
        assert!(vet_ctl("log 0").is_err());
        assert!(vet_ctl("reassoc").is_err());
        assert!(vet_ctl("chor /etc/passwd").is_err());
        assert!(vet_ctl("chor /vl/../secret").is_err());
        assert!(vet_ctl("ears 99 0").is_err());
        assert!(vet_ctl("color zzzzzz").is_err());
    }

    #[test]
    fn names_are_one_short_line() {
        assert_eq!(vet_name("  Garen \n"), Ok("Garen".to_string()));
        assert_eq!(vet_name(""), Ok(String::new()));
        assert_eq!(vet_name("Ga\tren"), Err("name_chars"));
        assert_eq!(vet_name(&"a".repeat(33)), Err("name_length"));
    }

    #[test]
    fn macs_are_twelve_lowercase_hex() {
        assert!(mac_ok("0019db9c2815"));
        assert!(!mac_ok("0019DB9C2815"));
        assert!(!mac_ok("0019db9c28"));
        assert!(!mac_ok("0019db9c28xy"));
    }
}

/// The log channel is shared: the ad-hoc listener scripts want the same
/// broadcast datagrams clapier does. SO_REUSEPORT makes the port a
/// party line instead of a fight - every bound socket receives each
/// broadcast (they also set it).
pub fn log_socket(port: u16) -> std::io::Result<tokio::net::UdpSocket> {
    use socket2::{Domain, Protocol, Socket, Type};
    let sock = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    // Windows has no SO_REUSEPORT; the party line stays a unix affair and
    // the fleet listener simply owns the port there.
    #[cfg(unix)]
    sock.set_reuse_port(true)?;
    sock.set_nonblocking(true)?;
    sock.bind(&SocketAddr::from(([0, 0, 0, 0], port)).into())?;
    tokio::net::UdpSocket::from_std(sock.into())
}

/// Bind the log port, then prove the socket actually hears: on macOS a
/// SO_REUSEPORT join races the dying previous instance's socket, the
/// kernel's group steering freezes on the corpse, and the survivor
/// stays deaf to broadcasts while looking perfectly bound. The roll
/// call catches it: send ourselves a datagram, and rebind until it
/// arrives.
async fn log_socket_verified(port: u16) -> std::io::Result<tokio::net::UdpSocket> {
    for attempt in 1..=5u32 {
        let sock = log_socket(port)?;
        // The probe must be a broadcast: a deaf socket still receives
        // unicast, which fooled the first version of this roll call.
        // Success is any datagram at all - the probe looping back, or a
        // rabbit's own two-second pulse.
        let probe = tokio::net::UdpSocket::bind(("0.0.0.0", 0)).await?;
        socket2::SockRef::from(&probe).set_broadcast(true)?;
        let _ = probe
            .send_to(b"clapier roll call", ("255.255.255.255", port))
            .await;
        let mut buf = [0u8; 64];
        match tokio::time::timeout(Duration::from_millis(2500), sock.recv_from(&mut buf)).await {
            Ok(Ok(_)) => return Ok(sock),
            _ => {
                tracing::warn!("udp {port} bound but deaf (attempt {attempt}), rebinding");
                drop(sock);
                tokio::time::sleep(Duration::from_millis(200)).await;
            }
        }
    }
    // Last try is returned unverified rather than giving up the port.
    log_socket(port)
}

/// Feeds the fleet register from the rabbits' UDP log channel. The
/// channel is broadcast on the LAN, so hearing every rabbit only takes
/// joining the port. Non-pulse chatter (`button: click`, reassoc
/// traces) is left to the human listener scripts.
pub async fn listen_pulses(app: Arc<AppState>, port: u16) {
    let sock = match log_socket_verified(port).await {
        Ok(sock) => sock,
        Err(err) => {
            tracing::warn!("cannot hear pulses, udp {port} unavailable: {err}");
            return;
        }
    };
    info!("hearing pulses on udp {port}");
    let mut buf = [0u8; 512];
    loop {
        let (n, src) = match sock.recv_from(&mut buf).await {
            Ok(received) => received,
            Err(err) => {
                tracing::warn!("pulse listener stopped: {err}");
                return;
            }
        };
        let line = String::from_utf8_lossy(&buf[..n]);
        if let Some(pulse) = clapier_fleet::parse_pulse(&line) {
            app.fleet.pulse(src.ip(), pulse, Instant::now());
        }
    }
}

pub fn router(app: Arc<AppState>) -> Router {
    Router::new()
        // The web app lives at the root: rabbits only ever ask under /vl.
        // Its files and API stay under /_clapier, out of the rabbits' way;
        // the old addresses lead home.
        .route("/", get(web_shell))
        .route("/_clapier", get(|| async { Redirect::permanent("/") }))
        .route("/_clapier/", get(|| async { Redirect::permanent("/") }))
        .route(
            "/_clapier/pilot",
            get(|| async { Redirect::permanent("/") }),
        )
        .route("/_clapier/health", get(|| async { "ok" }))
        .route("/_clapier/api/status", get(api_status))
        .route("/_clapier/api/ctl", post(api_ctl))
        .route("/_clapier/api/dance", post(api_dance))
        .route("/_clapier/api/say", post(api_say))
        .route("/_clapier/api/name", post(api_name))
        .route("/_clapier/{*path}", get(web_asset))
        .fallback(serve_content)
        .with_state(app)
}

/// The web app's policy: our own scripts only (the theme is read before
/// first paint by a file, not an inline script), inline styles allowed
/// for the floating menus' positions, nothing framed, nothing sent away.
const CSP: &str = "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; \
img-src 'self' data:; font-src 'self'; connect-src 'self'; frame-ancestors 'none'; \
base-uri 'none'; form-action 'self'; object-src 'none'";

fn web_response(asset: pages::Asset) -> Response {
    let cache = if asset.immutable {
        "public, max-age=31536000, immutable"
    } else {
        // the shell names the current build's files: always asked again
        "no-cache"
    };
    Response::builder()
        .header(header::CONTENT_TYPE, asset.mime)
        .header(header::CACHE_CONTROL, cache)
        .header(header::CONTENT_SECURITY_POLICY, CSP)
        .header(header::X_CONTENT_TYPE_OPTIONS, "nosniff")
        .header(header::REFERRER_POLICY, "no-referrer")
        .body(Body::from(asset.bytes))
        .expect("static response")
}

async fn web_shell() -> Response {
    web_response(pages::index())
}

async fn web_asset(Path(path): Path<String>) -> Response {
    match pages::asset(&path) {
        Some(asset) if path != "index.html" => web_response(asset),
        _ => (StatusCode::NOT_FOUND, "not found").into_response(),
    }
}

/// Every .chor the overlay can serve, as the /vl paths the rabbits ask.
async fn chor_library(app: &AppState) -> Vec<String> {
    async fn collect(dir: PathBuf, prefix: &str, out: &mut Vec<String>) {
        if let Ok(mut entries) = tokio::fs::read_dir(dir).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let name = entry.file_name().to_string_lossy().into_owned();
                if name.ends_with(".chor") {
                    out.push(format!("{prefix}/{name}"));
                }
            }
        }
    }
    let mut out = Vec::new();
    if let Some(overlay) = &app.tree.overlay {
        collect(
            overlay.join("common/vl/config/chor"),
            "/vl/config/chor",
            &mut out,
        )
        .await;
        for rabbit in app.fleet.snapshot() {
            if let Some(mac) = rabbit.mac {
                collect(
                    overlay.join("rabbits").join(&mac).join("vl/chor"),
                    "/vl/chor",
                    &mut out,
                )
                .await;
            }
        }
    }
    out.sort();
    out.dedup();
    out
}

/// A refusal the web app words itself: a stable `code`, sometimes a
/// technical `detail` (the encoder's reason) shown as is.
pub struct ApiError {
    status: StatusCode,
    code: &'static str,
    detail: Option<String>,
}

impl ApiError {
    fn new(status: StatusCode, code: &'static str) -> Self {
        Self {
            status,
            code,
            detail: None,
        }
    }
}

impl From<Refusal> for ApiError {
    fn from(refusal: Refusal) -> Self {
        Self::new(StatusCode::UNPROCESSABLE_ENTITY, refusal.code())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        #[derive(serde::Serialize)]
        struct Body {
            code: &'static str,
            #[serde(skip_serializing_if = "Option::is_none")]
            detail: Option<String>,
        }
        let body = Body {
            code: self.code,
            detail: self.detail,
        };
        (self.status, Json(body)).into_response()
    }
}

/// What the rabbit said back.
#[derive(serde::Serialize)]
pub struct Reply {
    reply: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    bytes: Option<usize>,
}

/// An IP the fleet knows, or a refusal: the pilot never aims the radio
/// at an address it was handed.
fn known_rabbit(app: &AppState, ip: &str) -> Result<IpAddr, ApiError> {
    let ip: IpAddr = ip
        .parse()
        .map_err(|_| ApiError::new(StatusCode::BAD_REQUEST, "bad_ip"))?;
    if !app.fleet.snapshot().iter().any(|r| r.ip == ip) {
        return Err(ApiError::new(StatusCode::NOT_FOUND, "unknown_rabbit"));
    }
    Ok(ip)
}

#[derive(serde::Deserialize)]
pub struct CtlRequest {
    ip: String,
    cmd: String,
}

async fn api_ctl(
    State(app): State<Arc<AppState>>,
    Json(req): Json<CtlRequest>,
) -> Result<Json<Reply>, ApiError> {
    let ip = known_rabbit(&app, &req.ip)?;
    let cmd = vet_ctl(&req.cmd)?;
    match ctl_send(ip, &cmd).await {
        Ok(reply) => Ok(Json(Reply {
            reply: Some(reply),
            bytes: None,
        })),
        Err(_) => Err(ApiError::new(StatusCode::GATEWAY_TIMEOUT, "no_reply")),
    }
}

#[derive(serde::Deserialize)]
pub struct DanceRequest {
    mac: String,
    ip: String,
    cdl: String,
}

async fn api_dance(
    State(app): State<Arc<AppState>>,
    Json(req): Json<DanceRequest>,
) -> Result<Json<Reply>, ApiError> {
    if !mac_ok(&req.mac) {
        return Err(ApiError::new(StatusCode::BAD_REQUEST, "bad_mac"));
    }
    let ip = known_rabbit(&app, &req.ip)?;
    let Some(overlay) = app.tree.overlay.clone() else {
        return Err(ApiError::new(StatusCode::CONFLICT, "no_overlay"));
    };
    let bytes = clapier_chor::encode_cdl(&req.cdl).map_err(|reason| ApiError {
        detail: Some(reason.to_string()),
        ..ApiError::new(StatusCode::UNPROCESSABLE_ENTITY, "chor_rejected")
    })?;
    let dir = overlay.join("rabbits").join(&req.mac).join("vl/chor");
    let tmp = dir.join(".pilot.chor.tmp");
    let dest = dir.join("pilot.chor");
    let written = async {
        tokio::fs::create_dir_all(&dir).await?;
        tokio::fs::write(&tmp, &bytes).await?;
        tokio::fs::rename(&tmp, &dest).await
    }
    .await;
    if written.is_err() {
        return Err(ApiError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "install_failed",
        ));
    }
    // Installed either way; a silent rabbit is said, not failed.
    let reply = ctl_send(ip, "chor /vl/chor/pilot.chor").await.ok();
    Ok(Json(Reply {
        reply,
        bytes: Some(bytes.len()),
    }))
}

#[derive(serde::Deserialize)]
pub struct NameRequest {
    mac: String,
    name: String,
}

async fn api_name(
    State(app): State<Arc<AppState>>,
    Json(req): Json<NameRequest>,
) -> Result<StatusCode, ApiError> {
    if !mac_ok(&req.mac) {
        return Err(ApiError::new(StatusCode::BAD_REQUEST, "bad_mac"));
    }
    let name = vet_name(&req.name)
        .map_err(|code| ApiError::new(StatusCode::UNPROCESSABLE_ENTITY, code))?;
    let Some(overlay) = app.tree.overlay.clone() else {
        return Err(ApiError::new(StatusCode::CONFLICT, "no_overlay"));
    };
    let path = name_path(&overlay, &req.mac);
    let written = if name.is_empty() {
        match tokio::fs::remove_file(&path).await {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e),
            _ => Ok(()),
        }
    } else {
        let dir = path.parent().expect("name has a burrow").to_path_buf();
        let tmp = dir.join(".name.tmp");
        async {
            tokio::fs::create_dir_all(&dir).await?;
            tokio::fs::write(&tmp, format!("{name}\n")).await?;
            tokio::fs::rename(&tmp, &path).await
        }
        .await
    };
    written
        .map(|()| StatusCode::NO_CONTENT)
        .map_err(|_| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, "name_failed"))
}

#[derive(serde::Deserialize)]
pub struct SayRequest {
    text: String,
}

async fn api_say(
    State(app): State<Arc<AppState>>,
    Json(req): Json<SayRequest>,
) -> Result<StatusCode, ApiError> {
    let Some(script) = app.say_script.clone() else {
        return Err(ApiError::new(StatusCode::CONFLICT, "say_off"));
    };
    let text = req.text.trim().to_string();
    if text.is_empty() || text.chars().count() > 300 {
        return Err(ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "say_length",
        ));
    }
    // Generation takes tens of seconds on the TTS model; fire and
    // forget, the rabbit will start speaking when the MP3 lands.
    match tokio::process::Command::new(&script).arg(&text).spawn() {
        Ok(_) => Ok(StatusCode::ACCEPTED),
        Err(_) => Err(ApiError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "say_failed",
        )),
    }
}

async fn serve_content(
    State(app): State<Arc<AppState>>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    method: Method,
    uri: Uri,
) -> Response {
    let mut resp = clapier_vl::respond(&app.tree, &method, &uri).await;
    if let Some(mac) = clapier_vl::rabbit_id(&uri) {
        let boot = uri.path().ends_with("bc.jsp");
        // A rabbit asking for its bytecode and finding an empty burrow
        // gets garenne installed on the spot, then served normally.
        if boot
            && method == Method::GET
            && resp.status() == axum::http::StatusCode::NOT_FOUND
            && app.adopt(&mac)
        {
            resp = clapier_vl::respond(&app.tree, &method, &uri).await;
        }
        app.fleet.seen_http(&mac, peer.ip(), boot, Instant::now());
    }
    record(&app, peer.ip(), method.clone(), &uri, &resp);
    if method == Method::HEAD {
        *resp.body_mut() = Body::empty();
    }
    resp
}

fn record(app: &AppState, peer: IpAddr, method: Method, uri: &Uri, resp: &Response) {
    // Browser reflexes are not traffic worth remembering.
    if uri.path() == "/favicon.ico" {
        return;
    }
    let bytes = resp
        .headers()
        .get(header::CONTENT_LENGTH)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    let status = resp.status();
    let tag = if Some(peer) == app.rabbit {
        "🐰 "
    } else {
        ""
    };
    info!("{tag}{peer} {method} {uri} → {} {bytes} B", status.as_u16());
    app.journal.record(Hit {
        at: Instant::now(),
        peer,
        method,
        uri: uri.to_string(),
        status,
        bytes,
    });
}

/// Everything the web app shows, in one answer: it asks every few
/// seconds while it is looked at.
#[derive(serde::Serialize)]
pub struct StatusView {
    version: &'static str,
    uptime_s: u64,
    serving: String,
    /// The `--rabbit` IP, and how long ago it last asked anything.
    tagged: Option<Tagged>,
    say: bool,
    rabbits: Vec<RabbitView>,
    chors: Vec<String>,
    journal: Vec<HitView>,
}

#[derive(serde::Serialize)]
pub struct Tagged {
    ip: String,
    seen_s: Option<u64>,
}

/// One rabbit as the wire showed it; ages in seconds, `None` for what
/// it has not said yet.
#[derive(serde::Serialize)]
pub struct RabbitView {
    mac: Option<String>,
    /// What the person called it; `None` until named.
    name: Option<String>,
    ip: String,
    version: Option<String>,
    boot_s: Option<u64>,
    pulse_s: Option<u64>,
    status_s: Option<u64>,
    uptime_s: Option<u64>,
    link: Option<u8>,
    rssi: Option<i64>,
    ears: Option<String>,
    audio: Option<String>,
    choreo: Option<String>,
    reassoc: Option<u64>,
    tcp_retx: Option<u64>,
}

#[derive(serde::Serialize)]
pub struct HitView {
    ago_ms: u64,
    peer: String,
    method: String,
    uri: String,
    status: u16,
    bytes: usize,
    /// The peer is a rabbit of the fleet (or the `--rabbit` one).
    rabbit: bool,
}

async fn api_status(State(app): State<Arc<AppState>>) -> Json<StatusView> {
    let journal = app.journal.snapshot();
    let fleet = app.fleet.snapshot();
    let age = |at: Option<Instant>| at.map(|at| at.elapsed().as_secs());
    let tagged = app.rabbit.map(|ip| Tagged {
        ip: ip.to_string(),
        seen_s: journal
            .iter()
            .rev()
            .find(|h| h.peer == ip)
            .map(|h| h.at.elapsed().as_secs()),
    });
    let is_rabbit = |peer: IpAddr| Some(peer) == app.rabbit || fleet.iter().any(|r| r.ip == peer);
    let journal = journal
        .iter()
        .rev()
        .map(|hit| HitView {
            ago_ms: u64::try_from(hit.at.elapsed().as_millis()).unwrap_or(u64::MAX),
            peer: hit.peer.to_string(),
            method: hit.method.to_string(),
            uri: hit.uri.clone(),
            status: hit.status.as_u16(),
            bytes: hit.bytes,
            rabbit: is_rabbit(hit.peer),
        })
        .collect();
    let rabbits = fleet
        .iter()
        .map(|r| {
            let status = r.status.as_ref();
            RabbitView {
                mac: r.mac.clone(),
                name: match (&app.tree.overlay, &r.mac) {
                    (Some(overlay), Some(mac)) => read_name(overlay, mac),
                    _ => None,
                },
                ip: r.ip.to_string(),
                version: r.pulse.as_ref().map(|p| p.version.clone()),
                boot_s: age(r.last_boot),
                pulse_s: age(r.last_pulse),
                status_s: age(r.last_status),
                uptime_s: r.pulse.as_ref().map(|p| p.uptime_s),
                link: r.pulse.as_ref().map(|p| p.link),
                rssi: status.and_then(|s| s.rssi),
                ears: status.and_then(|s| s.ears.clone()),
                audio: status.and_then(|s| s.audio.clone()),
                choreo: status.and_then(|s| s.choreo.clone()),
                reassoc: status.and_then(|s| s.reassoc),
                tcp_retx: status.and_then(|s| s.tcp_retx),
            }
        })
        .collect();
    let serving = match (&app.tree.base, &app.tree.overlay) {
        (Some(base), Some(overlay)) => {
            format!("{} + overlay {}", base.display(), overlay.display())
        }
        (Some(base), None) => base.display().to_string(),
        (None, Some(overlay)) => format!("overlay {}", overlay.display()),
        (None, None) => "nothing (empty tree)".to_string(),
    };
    Json(StatusView {
        version: env!("CARGO_PKG_VERSION"),
        uptime_s: app.started.elapsed().as_secs(),
        serving,
        tagged,
        say: app.say_script.is_some(),
        rabbits,
        chors: chor_library(&app).await,
        journal,
    })
}
