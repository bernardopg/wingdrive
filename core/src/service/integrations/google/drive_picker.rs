//! # Google Drive picker
//!
//! WingDrive's own Google client asks only for `drive.file`, so Drive returns
//! just the files WingDrive created or the user opened with it. The Google
//! Picker is how a user opens existing files with WingDrive: every file picked
//! there joins the `drive.file` grant and shows up in the Drive volume.
//!
//! The Picker is a JavaScript widget, so WingDrive serves a small page on a
//! random loopback port and opens it in the browser. The page loads the Picker
//! with the account's access token, WingDrive's API key and the Cloud project
//! number (the app ID that ties picked files to WingDrive's grant), and reports
//! back through the same listener when the user picks or cancels. Picked file
//! IDs never travel back: Google records the grant itself.

use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use specta::Type;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tracing::{debug, warn};

use super::oauth::random_token;

/// Path of the page that shows the Picker.
const PAGE_PATH: &str = "/drive-picker";

/// Path the page calls when the user picks files.
const DONE_PATH: &str = "/drive-picker/done";

/// Path the page calls when the user closes the Picker without picking.
const CANCEL_PATH: &str = "/drive-picker/cancel";

/// Where a Drive picker session stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum DrivePickerState {
	/// The page is open and the user has not picked yet.
	Waiting,
	/// The user picked `count` files or folders, now part of the grant.
	Picked { count: u32 },
	/// The user closed the Picker without picking.
	Cancelled,
	/// The session timed out before the user finished.
	Expired,
}

/// What the page needs to build the Picker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DrivePickerConfig {
	/// Browser API key of WingDrive's Cloud project, restricted to the Picker API.
	pub api_key: String,
	/// Cloud project number, the prefix of the OAuth client ID.
	pub app_id: String,
	/// Access token of the account whose Drive the Picker shows.
	pub access_token: String,
	/// UI language for the Picker, such as `pt-BR`.
	pub locale: Option<String>,
}

/// Cloud project number of an OAuth client, which the Picker needs as app ID.
///
/// Google client IDs start with it: `49100968698-abc.apps.googleusercontent.com`.
pub fn project_number(client_id: &str) -> Option<&str> {
	let number = client_id.split('-').next()?;
	(!number.is_empty() && number.bytes().all(|b| b.is_ascii_digit())).then_some(number)
}

/// A Picker page served on a loopback port until the user finishes or it expires.
#[derive(Debug, Clone)]
pub struct DrivePickerServer {
	url: String,
	state: Arc<Mutex<DrivePickerState>>,
}

impl DrivePickerServer {
	/// Binds `127.0.0.1` on a port the OS picks and serves the page for `timeout`.
	pub async fn start(config: DrivePickerConfig, timeout: Duration) -> std::io::Result<Self> {
		let listener = TcpListener::bind(("127.0.0.1", 0)).await?;
		let port = listener.local_addr()?.port();
		let token = random_token();
		let state = Arc::new(Mutex::new(DrivePickerState::Waiting));
		let page = picker_page(&config, &token);
		let serving = state.clone();
		let expected = token.clone();
		tokio::spawn(async move {
			let serve = serve(listener, &expected, &page, &serving);
			if tokio::time::timeout(timeout, serve).await.is_err() {
				let mut state = serving.lock().await;
				if *state == DrivePickerState::Waiting {
					*state = DrivePickerState::Expired;
				}
			}
		});
		Ok(Self {
			url: format!("http://127.0.0.1:{port}{PAGE_PATH}?state={token}"),
			state,
		})
	}

	/// Address of the page to open in the browser.
	pub fn url(&self) -> &str {
		&self.url
	}

	/// Current state of the session.
	pub async fn state(&self) -> DrivePickerState {
		*self.state.lock().await
	}
}

/// What a request to the listener asks for.
#[derive(Debug, PartialEq, Eq)]
enum PickerRequest {
	Page,
	Done { count: u32 },
	Cancel,
}

/// Parses a request line such as `GET /drive-picker/done?state=x&count=2 HTTP/1.1`.
///
/// Returns `None` for other paths and for a wrong or missing `state`.
fn parse_request(request_line: &str, expected_state: &str) -> Option<PickerRequest> {
	let mut parts = request_line.split_whitespace();
	if parts.next()? != "GET" {
		return None;
	}
	let url = url::Url::parse(&format!("http://127.0.0.1{}", parts.next()?)).ok()?;
	let query = |name: &str| {
		url.query_pairs()
			.find(|(key, _)| key == name)
			.map(|(_, value)| value.into_owned())
	};
	if query("state").as_deref() != Some(expected_state) {
		return None;
	}
	match url.path() {
		PAGE_PATH => Some(PickerRequest::Page),
		DONE_PATH => Some(PickerRequest::Done {
			count: query("count").and_then(|c| c.parse().ok()).unwrap_or(0),
		}),
		CANCEL_PATH => Some(PickerRequest::Cancel),
		_ => None,
	}
}

/// Answers requests until the user picks or cancels.
async fn serve(
	listener: TcpListener,
	expected_state: &str,
	page: &str,
	state: &Mutex<DrivePickerState>,
) {
	loop {
		let mut stream = match listener.accept().await {
			Ok((stream, _)) => stream,
			Err(error) => {
				warn!(%error, "Drive picker listener stopped");
				return;
			}
		};
		let request_line = read_request_line(&mut stream).await;
		let finished = match parse_request(&request_line, expected_state) {
			Some(PickerRequest::Page) => {
				let _ = respond(&mut stream, 200, "text/html; charset=utf-8", page).await;
				false
			}
			Some(PickerRequest::Done { count }) => {
				*state.lock().await = DrivePickerState::Picked { count };
				let _ = respond(&mut stream, 200, "text/plain; charset=utf-8", "ok").await;
				debug!(count, "Drive picker finished");
				true
			}
			Some(PickerRequest::Cancel) => {
				*state.lock().await = DrivePickerState::Cancelled;
				let _ = respond(&mut stream, 200, "text/plain; charset=utf-8", "ok").await;
				true
			}
			None => {
				let _ = respond(&mut stream, 404, "text/plain; charset=utf-8", "Not found").await;
				false
			}
		};
		if finished {
			return;
		}
	}
}

/// Reads up to the end of the headers and returns the request line.
async fn read_request_line(stream: &mut TcpStream) -> String {
	let mut buffer = vec![0u8; 8192];
	let mut read = 0;
	while read < buffer.len() {
		let n = match tokio::time::timeout(Duration::from_secs(5), stream.read(&mut buffer[read..]))
			.await
		{
			Ok(Ok(n)) => n,
			_ => 0,
		};
		if n == 0 {
			break;
		}
		read += n;
		if buffer[..read].windows(4).any(|w| w == b"\r\n\r\n") {
			break;
		}
	}
	String::from_utf8_lossy(&buffer[..read])
		.lines()
		.next()
		.unwrap_or_default()
		.to_string()
}

async fn respond(
	stream: &mut TcpStream,
	status: u16,
	content_type: &str,
	body: &str,
) -> std::io::Result<()> {
	let reason = if status == 200 { "OK" } else { "Not Found" };
	let response = format!(
		"HTTP/1.1 {status} {reason}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\nCache-Control: no-store\r\nReferrer-Policy: strict-origin\r\n\r\n{body}",
		body.len()
	);
	stream.write_all(response.as_bytes()).await?;
	stream.flush().await
}

/// The page that loads the Picker. It carries both languages because the
/// daemon does not know the UI language.
fn picker_page(config: &DrivePickerConfig, state: &str) -> String {
	// JSON string literals are valid JavaScript; `<` is escaped so a value can
	// never close the script element.
	let js = |value: &str| {
		serde_json::to_string(value)
			.unwrap_or_else(|_| "\"\"".to_string())
			.replace('<', "\\u003c")
	};
	let locale = config.locale.as_deref().unwrap_or("en");
	format!(
		r#"<!doctype html>
<html><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1">
<title>WingDrive - Google Drive</title>
<style>body{{font-family:system-ui,sans-serif;max-width:34rem;margin:4rem auto;padding:0 1rem;line-height:1.5}}</style>
</head><body>
<h1>WingDrive</h1>
<p id="status">Opening Google Drive... / Abrindo o Google Drive...</p>
<script>
const TOKEN = {token};
const KEY = {key};
const APP_ID = {app_id};
const STATE = {state};
const LOCALE = {locale};
const statusLine = document.getElementById("status");
function report(path) {{
  fetch(path + "?state=" + encodeURIComponent(STATE), {{ cache: "no-store" }}).catch(() => {{}});
}}
function show(en, pt) {{
  statusLine.textContent = en + " / " + pt;
}}
function picked(data) {{
  const action = data[google.picker.Response.ACTION];
  if (action === google.picker.Action.PICKED) {{
    const count = (data[google.picker.Response.DOCUMENTS] || []).length;
    fetch("{done}?state=" + encodeURIComponent(STATE) + "&count=" + count, {{ cache: "no-store" }}).catch(() => {{}});
    show(count + " item(s) added to WingDrive. You can close this tab.",
      count + " item(ns) adicionado(s) ao WingDrive. Você pode fechar esta aba.");
  }} else if (action === google.picker.Action.CANCEL) {{
    report("{cancel}");
    show("Nothing was added. You can close this tab.", "Nada foi adicionado. Você pode fechar esta aba.");
  }}
}}
function openPicker() {{
  const myDrive = new google.picker.DocsView(google.picker.ViewId.DOCS)
    .setIncludeFolders(true).setSelectFolderEnabled(true).setParent("root");
  const shared = new google.picker.DocsView(google.picker.ViewId.DOCS)
    .setIncludeFolders(true).setSelectFolderEnabled(true).setOwnedByMe(false);
  const drives = new google.picker.DocsView(google.picker.ViewId.DOCS)
    .setIncludeFolders(true).setSelectFolderEnabled(true).setEnableDrives(true);
  new google.picker.PickerBuilder()
    .setOAuthToken(TOKEN).setDeveloperKey(KEY).setAppId(APP_ID).setLocale(LOCALE)
    .setTitle("WingDrive")
    .enableFeature(google.picker.Feature.MULTISELECT_ENABLED)
    .enableFeature(google.picker.Feature.SUPPORT_DRIVES)
    .addView(myDrive).addView(shared).addView(drives)
    .setCallback(picked).build().setVisible(true);
}}
function failed() {{
  show("Google Drive could not be opened. Go back to WingDrive and try again.",
    "Não foi possível abrir o Google Drive. Volte ao WingDrive e tente de novo.");
}}
</script>
<script src="https://apis.google.com/js/api.js" onload="gapi.load('picker', {{ callback: openPicker, onerror: failed }})" onerror="failed()"></script>
</body></html>"#,
		token = js(&config.access_token),
		key = js(&config.api_key),
		app_id = js(&config.app_id),
		state = js(state),
		locale = js(locale),
		done = DONE_PATH,
		cancel = CANCEL_PATH,
	)
}

#[cfg(test)]
mod tests {
	use super::*;

	fn config() -> DrivePickerConfig {
		DrivePickerConfig {
			api_key: "key".to_string(),
			app_id: "123".to_string(),
			access_token: "token</script>".to_string(),
			locale: Some("pt-BR".to_string()),
		}
	}

	#[test]
	fn project_number_is_the_numeric_client_prefix() {
		assert_eq!(
			project_number("49100968698-abc.apps.googleusercontent.com"),
			Some("49100968698")
		);
		assert_eq!(project_number("abc-def.apps.googleusercontent.com"), None);
		assert_eq!(project_number(""), None);
	}

	#[test]
	fn requests_need_the_session_state() {
		assert_eq!(
			parse_request("GET /drive-picker?state=s HTTP/1.1", "s"),
			Some(PickerRequest::Page)
		);
		assert_eq!(
			parse_request("GET /drive-picker/done?state=s&count=3 HTTP/1.1", "s"),
			Some(PickerRequest::Done { count: 3 })
		);
		assert_eq!(
			parse_request("GET /drive-picker/cancel?state=s HTTP/1.1", "s"),
			Some(PickerRequest::Cancel)
		);
		assert_eq!(
			parse_request("GET /drive-picker?state=x HTTP/1.1", "s"),
			None
		);
		assert_eq!(parse_request("GET /drive-picker HTTP/1.1", "s"), None);
		assert_eq!(
			parse_request("GET /favicon.ico?state=s HTTP/1.1", "s"),
			None
		);
		assert_eq!(
			parse_request("POST /drive-picker?state=s HTTP/1.1", "s"),
			None
		);
	}

	#[test]
	fn page_escapes_values_out_of_the_script() {
		let page = picker_page(&config(), "s");
		assert!(page.contains(r#"const TOKEN = "token\u003c/script>";"#));
		assert!(page.contains(r#"const APP_ID = "123";"#));
		assert!(page.contains(r#"const LOCALE = "pt-BR";"#));
	}

	async fn get(url: &str) -> String {
		let url = url::Url::parse(url).unwrap();
		let mut stream = TcpStream::connect(("127.0.0.1", url.port().unwrap()))
			.await
			.unwrap();
		let target = match url.query() {
			Some(query) => format!("{}?{query}", url.path()),
			None => url.path().to_string(),
		};
		stream
			.write_all(format!("GET {target} HTTP/1.1\r\nHost: localhost\r\n\r\n").as_bytes())
			.await
			.unwrap();
		let mut response = String::new();
		stream.read_to_string(&mut response).await.unwrap();
		response
	}

	#[tokio::test]
	async fn server_serves_the_page_and_records_the_pick() {
		let server = DrivePickerServer::start(config(), Duration::from_secs(30))
			.await
			.unwrap();
		let page = get(server.url()).await;
		assert!(page.starts_with("HTTP/1.1 200"));
		assert!(page.contains("google.picker.PickerBuilder"));
		assert_eq!(server.state().await, DrivePickerState::Waiting);

		let done = server.url().replace(PAGE_PATH, DONE_PATH) + "&count=2";
		assert!(get(&done).await.starts_with("HTTP/1.1 200"));
		assert_eq!(server.state().await, DrivePickerState::Picked { count: 2 });
	}

	#[tokio::test]
	async fn server_expires_when_nobody_picks() {
		let server = DrivePickerServer::start(config(), Duration::from_millis(50))
			.await
			.unwrap();
		tokio::time::sleep(Duration::from_millis(150)).await;
		assert_eq!(server.state().await, DrivePickerState::Expired);
	}
}
