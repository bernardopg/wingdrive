//! # Desktop notifications
//!
//! WingDrive tells the user when a long job ends while its window is in the
//! background. A notification has to carry WingDrive's own name and logo, and
//! every desktop reads that identity from a different place:
//!
//! - Linux notification servers (GNOME Shell, Plasma, xfce4-notifyd, mako,
//!   dunst, swaync, Quickshell shells) take the app name from `app_name` and
//!   the app from the `desktop-entry` hint. Without the hint, servers that
//!   group by app fall back to a guess. Logos come from `app_icon` (a theme
//!   name) or, when the theme has no WingDrive icon, from `image-data`.
//! - Flatpak sandboxes cannot reach `org.freedesktop.Notifications` reliably,
//!   so they use `org.freedesktop.portal.Notification`, which derives the
//!   identity from the sandbox itself.
//! - Windows shows a toast only for an AppUserModelID that a Start Menu
//!   shortcut registers. The NSIS installer creates that shortcut with the
//!   bundle identifier, and `main` sets the same ID on the process.
//!
//! The functions that decide what to send are pure and tested here; only the
//! `send` functions talk to the desktop.
//!
//! ## Example
//! ```rust,ignore
//! let caps = Capabilities::parse(["body", "body-markup", "actions"]);
//! let call = build_notify_call(&request, &caps, &identity);
//! ```

use std::path::{Path, PathBuf};

use serde::Deserialize;

/// Name shown as the sender of every notification.
pub const APP_NAME: &str = "WingDrive";
/// Icon name the packages install into the hicolor theme.
pub const ICON_NAME: &str = "WingDrive";
/// Bundle identifier; also the Windows AppUserModelID.
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
pub const APP_ID: &str = "com.wingdrive.desktop";
/// Desktop file ids the packages install, in lookup order: the AUR package
/// ships `wingdrive.desktop`, the deb and AppImage ship `WingDrive.desktop`.
pub const DESKTOP_ENTRY_CANDIDATES: [&str; 2] = ["wingdrive", "WingDrive"];
/// Program name, which GTK turns into the Wayland app_id and X11 WM_CLASS.
/// It matches `StartupWMClass` in every desktop file WingDrive installs.
pub const PROGRAM_NAME: &str = "WingDrive";
/// Largest logo edge sent as `image-data`; bigger icons only cost bus traffic.
const MAX_IMAGE_EDGE: u32 = 128;
/// Action key the notification spec reserves for a click on the body.
pub const DEFAULT_ACTION: &str = "default";

/// What a notification reports, which sets its urgency and category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NotificationKind {
	/// A copy, move or other job finished.
	TransferComplete,
	/// A job failed and needs the user's attention.
	TransferError,
	/// Anything else worth a passing mention.
	Info,
}

impl NotificationKind {
	/// Urgency byte from the notification spec: 0 low, 1 normal, 2 critical.
	///
	/// Failures stay normal: critical notifications never expire on most
	/// servers, which is too loud for a failed copy.
	pub fn urgency(self) -> u8 {
		match self {
			Self::Info => 0,
			Self::TransferComplete | Self::TransferError => 1,
		}
	}

	/// Category from the notification spec, used by servers for sounds and rules.
	pub fn category(self) -> Option<&'static str> {
		match self {
			Self::TransferComplete => Some("transfer.complete"),
			Self::TransferError => Some("transfer.error"),
			Self::Info => None,
		}
	}

	/// Priority name for `org.freedesktop.portal.Notification`.
	pub fn portal_priority(self) -> &'static str {
		match self {
			Self::Info => "low",
			Self::TransferComplete | Self::TransferError => "normal",
		}
	}
}

/// A notification the frontend asks for, already translated.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationRequest {
	pub title: String,
	#[serde(default)]
	pub body: String,
	pub kind: NotificationKind,
	/// Label for the click action ("Show WingDrive"); servers that list
	/// actions as buttons show it, others only react to a click.
	#[serde(default)]
	pub open_label: Option<String>,
}

/// Features a notification server announces through `GetCapabilities`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Capabilities {
	pub body: bool,
	pub body_markup: bool,
	pub actions: bool,
	pub icon_static: bool,
	pub persistence: bool,
}

impl Capabilities {
	/// Reads the capability strings a server returned.
	pub fn parse<I, S>(capabilities: I) -> Self
	where
		I: IntoIterator<Item = S>,
		S: AsRef<str>,
	{
		let mut caps = Self::default();
		for capability in capabilities {
			match capability.as_ref() {
				"body" => caps.body = true,
				"body-markup" => caps.body_markup = true,
				"actions" => caps.actions = true,
				"icon-static" => caps.icon_static = true,
				"persistence" => caps.persistence = true,
				_ => {}
			}
		}
		caps
	}

	/// What to assume when `GetCapabilities` fails: a body and nothing else,
	/// which every server renders.
	pub fn minimal() -> Self {
		Self {
			body: true,
			..Self::default()
		}
	}
}

/// Raw RGBA pixels for the `image-data` hint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RgbaImage {
	pub width: u32,
	pub height: u32,
	pub rgba: Vec<u8>,
}

impl RgbaImage {
	/// Shrinks the image so neither edge exceeds `max_edge`, sampling the
	/// nearest pixel; logos are simple enough that this stays legible.
	pub fn fit(&self, max_edge: u32) -> Self {
		let longest = self.width.max(self.height);
		if longest <= max_edge || self.rgba.len() != (self.width * self.height * 4) as usize {
			return self.clone();
		}
		let width = (self.width * max_edge / longest).max(1);
		let height = (self.height * max_edge / longest).max(1);
		let mut rgba = Vec::with_capacity((width * height * 4) as usize);
		for y in 0..height {
			let source_y = y * self.height / height;
			for x in 0..width {
				let source_x = x * self.width / width;
				let at = ((source_y * self.width + source_x) * 4) as usize;
				rgba.extend_from_slice(&self.rgba[at..at + 4]);
			}
		}
		Self {
			width,
			height,
			rgba,
		}
	}
}

/// Where the notification's logo comes from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogoSource {
	/// The icon theme has `WingDrive`; servers resolve it themselves.
	Themed(String),
	/// No theme icon, so the pixels travel with the notification.
	Pixels(RgbaImage),
	/// Neither is available; the server shows its generic icon.
	None,
}

/// How the running WingDrive identifies itself to the notification server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identity {
	/// Basename of the installed desktop file, without `.desktop`.
	pub desktop_entry: String,
	pub logo: LogoSource,
}

/// One hint value; the variants are the D-Bus types the spec uses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Hint {
	Str(String),
	Byte(u8),
	Image(RgbaImage),
}

/// Arguments of one `org.freedesktop.Notifications.Notify` call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotifyCall {
	pub app_name: String,
	pub app_icon: String,
	pub summary: String,
	pub body: String,
	pub actions: Vec<String>,
	pub hints: Vec<(&'static str, Hint)>,
	/// -1 lets the server pick its default timeout.
	pub expire_timeout: i32,
}

/// Escapes text for servers that render the body as markup.
///
/// File names often contain `&` or `<`; unescaped, a markup server drops the
/// body or shows it cut off.
pub fn escape_markup(text: &str) -> String {
	let mut escaped = String::with_capacity(text.len());
	for character in text.chars() {
		match character {
			'&' => escaped.push_str("&amp;"),
			'<' => escaped.push_str("&lt;"),
			'>' => escaped.push_str("&gt;"),
			_ => escaped.push(character),
		}
	}
	escaped
}

/// Builds the `Notify` call for a server with `caps`.
pub fn build_notify_call(
	request: &NotificationRequest,
	caps: &Capabilities,
	identity: &Identity,
) -> NotifyCall {
	let body = if !caps.body {
		String::new()
	} else if caps.body_markup {
		escape_markup(&request.body)
	} else {
		request.body.clone()
	};

	let mut hints = vec![
		("desktop-entry", Hint::Str(identity.desktop_entry.clone())),
		("urgency", Hint::Byte(request.kind.urgency())),
	];
	if let Some(category) = request.kind.category() {
		hints.push(("category", Hint::Str(category.to_owned())));
	}

	let app_icon = match &identity.logo {
		LogoSource::Themed(name) => name.clone(),
		LogoSource::Pixels(image) => {
			hints.push(("image-data", Hint::Image(image.fit(MAX_IMAGE_EDGE))));
			String::new()
		}
		LogoSource::None => String::new(),
	};

	// Advertising an action the server cannot show makes some servers
	// (xfce4-notifyd, older dunst) render a dead button, so only ask when the
	// server says it supports actions.
	let actions = if caps.actions {
		let label = request
			.open_label
			.clone()
			.filter(|label| !label.trim().is_empty())
			.unwrap_or_else(|| APP_NAME.to_owned());
		vec![DEFAULT_ACTION.to_owned(), label]
	} else {
		Vec::new()
	};

	NotifyCall {
		app_name: APP_NAME.to_owned(),
		app_icon,
		summary: request.title.clone(),
		body,
		actions,
		hints,
		expire_timeout: -1,
	}
}

/// Portal notification fields, as `(key, value)` pairs of the `a{sv}` dict.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PortalValue {
	Str(String),
	/// A serialized themed `GIcon`: `("themed", <["name", ...]>)`.
	ThemedIcon(Vec<String>),
}

/// Builds the `AddNotification` dict for `org.freedesktop.portal.Notification`.
///
/// The portal knows the app from the sandbox, so no desktop entry is sent.
pub fn build_portal_notification(
	request: &NotificationRequest,
	icon_name: &str,
) -> Vec<(&'static str, PortalValue)> {
	let mut fields = vec![("title", PortalValue::Str(request.title.clone()))];
	if !request.body.is_empty() {
		fields.push(("body", PortalValue::Str(request.body.clone())));
	}
	fields.push((
		"priority",
		PortalValue::Str(request.kind.portal_priority().to_owned()),
	));
	fields.push((
		"icon",
		PortalValue::ThemedIcon(vec![icon_name.to_owned(), format!("{icon_name}-symbolic")]),
	));
	fields
}

/// Data directories in XDG lookup order: `$XDG_DATA_HOME`, then `$XDG_DATA_DIRS`.
pub fn xdg_data_dirs(
	data_home: Option<&str>,
	data_dirs: Option<&str>,
	home: Option<&Path>,
) -> Vec<PathBuf> {
	let mut dirs = Vec::new();
	match data_home.filter(|value| !value.is_empty()) {
		Some(value) => dirs.push(PathBuf::from(value)),
		None => {
			if let Some(home) = home {
				dirs.push(home.join(".local/share"));
			}
		}
	}
	let system = data_dirs
		.filter(|value| !value.is_empty())
		.unwrap_or("/usr/local/share:/usr/share");
	dirs.extend(
		system
			.split(':')
			.filter(|entry| !entry.is_empty())
			.map(PathBuf::from),
	);
	dirs
}

/// Picks the desktop file WingDrive was installed with.
///
/// The `desktop-entry` hint must name a file that exists, or servers that
/// look it up (GNOME Shell, Plasma, Quickshell shells) lose the app. With no
/// installed file (a development build) the deb/AppImage name is used, since
/// that is what the bundler generates.
pub fn resolve_desktop_entry(data_dirs: &[PathBuf], exists: impl Fn(&Path) -> bool) -> String {
	for candidate in DESKTOP_ENTRY_CANDIDATES {
		let file = format!("{candidate}.desktop");
		if data_dirs
			.iter()
			.any(|dir| exists(&dir.join("applications").join(&file)))
		{
			return candidate.to_owned();
		}
	}
	PROGRAM_NAME.to_owned()
}

/// Whether some hicolor size directory under `data_dirs` holds `name`.
pub fn theme_has_icon(data_dirs: &[PathBuf], name: &str, exists: impl Fn(&Path) -> bool) -> bool {
	const SIZES: [&str; 9] = [
		"scalable",
		"512x512",
		"256x256",
		"256x256@2",
		"128x128",
		"64x64",
		"48x48",
		"32x32",
		"16x16",
	];
	data_dirs.iter().any(|dir| {
		SIZES.iter().any(|size| {
			let apps = dir.join("icons/hicolor").join(size).join("apps");
			exists(&apps.join(format!("{name}.png"))) || exists(&apps.join(format!("{name}.svg")))
		})
	})
}

#[cfg(target_os = "linux")]
mod linux {
	use std::collections::HashMap;
	use std::sync::Mutex;

	use tauri::AppHandle;
	use tokio::sync::OnceCell;
	use zbus::{Structure, Variant as Value};

	use super::*;

	const NOTIFICATIONS_NAME: &str = "org.freedesktop.Notifications";
	const NOTIFICATIONS_PATH: &str = "/org/freedesktop/Notifications";
	const PORTAL_NAME: &str = "org.freedesktop.portal.Desktop";
	const PORTAL_PATH: &str = "/org/freedesktop/portal/desktop";
	const PORTAL_INTERFACE: &str = "org.freedesktop.portal.Notification";

	/// Session bus connection, server capabilities and the ids WingDrive sent.
	#[derive(Default)]
	pub struct Notifier {
		session: OnceCell<Option<(zbus::Connection, Capabilities)>>,
		identity: OnceCell<Identity>,
		sent: Mutex<Vec<u32>>,
	}

	impl Notifier {
		async fn session(&self, app: &AppHandle) -> Option<&(zbus::Connection, Capabilities)> {
			self.session
				.get_or_init(|| async {
					let connection = match zbus::Connection::session().await {
						Ok(connection) => connection,
						Err(error) => {
							tracing::warn!(%error, "No session bus for notifications");
							return None;
						}
					};
					let caps = match notifications_proxy(&connection).await {
						Ok(proxy) => match proxy
							.call::<_, _, Vec<String>>("GetCapabilities", &())
							.await
						{
							Ok(list) => Capabilities::parse(list),
							Err(error) => {
								tracing::debug!(%error, "GetCapabilities failed; assuming body only");
								Capabilities::minimal()
							}
						},
						Err(_) => Capabilities::minimal(),
					};
					if caps.actions {
						listen_for_clicks(app.clone(), connection.clone());
					}
					Some((connection, caps))
				})
				.await
				.as_ref()
		}

		fn identity(&self, app: &AppHandle) -> &Identity {
			if let Some(identity) = self.identity.get() {
				return identity;
			}
			let identity = detect_identity(app);
			let _ = self.identity.set(identity);
			self.identity.get().expect("identity was just set")
		}

		pub fn owns(&self, id: u32) -> bool {
			self.sent.lock().unwrap().contains(&id)
		}

		pub async fn send(
			&self,
			app: &AppHandle,
			request: &NotificationRequest,
		) -> Result<(), String> {
			if is_sandboxed() {
				return send_portal(request).await;
			}
			let identity = self.identity(app).clone();
			let Some((connection, caps)) = self.session(app).await else {
				return Err("session bus unavailable".into());
			};
			let call = build_notify_call(request, caps, &identity);
			let proxy = notifications_proxy(connection)
				.await
				.map_err(|error| error.to_string())?;
			let hints: HashMap<&str, Value<'_>> = call
				.hints
				.iter()
				.map(|(key, hint)| (*key, hint_value(hint)))
				.collect();
			let actions: Vec<&str> = call.actions.iter().map(String::as_str).collect();
			let id: u32 = proxy
				.call(
					"Notify",
					&(
						call.app_name.as_str(),
						0u32,
						call.app_icon.as_str(),
						call.summary.as_str(),
						call.body.as_str(),
						actions,
						hints,
						call.expire_timeout,
					),
				)
				.await
				.map_err(|error| error.to_string())?;
			let mut sent = self.sent.lock().unwrap();
			sent.push(id);
			// Ids only matter until the user acts on recent notifications.
			if sent.len() > 64 {
				sent.remove(0);
			}
			Ok(())
		}
	}

	async fn notifications_proxy(
		connection: &zbus::Connection,
	) -> zbus::Result<zbus::Proxy<'static>> {
		zbus::Proxy::new(
			connection,
			NOTIFICATIONS_NAME,
			NOTIFICATIONS_PATH,
			NOTIFICATIONS_NAME,
		)
		.await
	}

	fn hint_value(hint: &Hint) -> Value<'static> {
		match hint {
			Hint::Str(text) => Value::from(text.clone()),
			Hint::Byte(byte) => Value::U8(*byte),
			Hint::Image(image) => {
				// (iiibiiay): width, height, rowstride, has alpha, bits per
				// sample, channels, pixels.
				let structure: Structure<'static> = (
					image.width as i32,
					image.height as i32,
					(image.width * 4) as i32,
					true,
					8i32,
					4i32,
					image.rgba.clone(),
				)
					.into();
				Value::Structure(structure)
			}
		}
	}

	/// Opens the main window when the user clicks a WingDrive notification.
	fn listen_for_clicks(app: AppHandle, connection: zbus::Connection) {
		use stream::next;
		tauri::async_runtime::spawn(async move {
			let Ok(proxy) = notifications_proxy(&connection).await else {
				return;
			};
			let Ok(mut stream) = proxy.receive_signal("ActionInvoked").await else {
				return;
			};
			while let Some(message) = next(&mut stream).await {
				let Ok((id, action)) = message.body().deserialize::<(u32, String)>() else {
					continue;
				};
				let ours = tauri::Manager::try_state::<Notifier>(&app)
					.map(|notifier| notifier.owns(id))
					.unwrap_or(false);
				if ours && action == DEFAULT_ACTION {
					crate::launch::show_main_window(&app);
				}
			}
		});
	}

	/// `StreamExt::next` without pulling in a stream crate for one call.
	mod stream {
		use std::future::poll_fn;
		use std::pin::Pin;

		use zbus::export::futures_core::Stream;

		pub async fn next<S: Stream + Unpin>(stream: &mut S) -> Option<S::Item> {
			poll_fn(|context| Pin::new(&mut *stream).poll_next(context)).await
		}
	}

	pub fn is_sandboxed() -> bool {
		Path::new("/.flatpak-info").exists()
	}

	async fn send_portal(request: &NotificationRequest) -> Result<(), String> {
		let connection = zbus::Connection::session()
			.await
			.map_err(|error| error.to_string())?;
		let proxy = zbus::Proxy::new(&connection, PORTAL_NAME, PORTAL_PATH, PORTAL_INTERFACE)
			.await
			.map_err(|error| error.to_string())?;
		let fields: HashMap<&str, Value<'_>> = build_portal_notification(request, ICON_NAME)
			.into_iter()
			.map(|(key, value)| {
				let value = match value {
					PortalValue::Str(text) => Value::from(text),
					PortalValue::ThemedIcon(names) => {
						let icon: Structure<'static> = ("themed", Value::from(names)).into();
						Value::Structure(icon)
					}
				};
				(key, value)
			})
			.collect();
		let id = format!("wingdrive-{}", std::process::id());
		proxy
			.call::<_, _, ()>("AddNotification", &(id.as_str(), fields))
			.await
			.map_err(|error| error.to_string())
	}

	fn detect_identity(app: &AppHandle) -> Identity {
		let home = std::env::var_os("HOME").map(PathBuf::from);
		let data_home = std::env::var("XDG_DATA_HOME").ok();
		let data_dirs_env = std::env::var("XDG_DATA_DIRS").ok();
		let data_dirs = xdg_data_dirs(
			data_home.as_deref(),
			data_dirs_env.as_deref(),
			home.as_deref(),
		);
		let desktop_entry = resolve_desktop_entry(&data_dirs, Path::exists);
		let logo = if theme_has_icon(&data_dirs, ICON_NAME, Path::exists) {
			LogoSource::Themed(ICON_NAME.to_owned())
		} else {
			// An AppImage run without desktop integration has no theme icon;
			// send the window icon's pixels instead.
			match app.default_window_icon() {
				Some(icon) => LogoSource::Pixels(RgbaImage {
					width: icon.width(),
					height: icon.height(),
					rgba: icon.rgba().to_vec(),
				}),
				None => LogoSource::None,
			}
		};
		Identity {
			desktop_entry,
			logo,
		}
	}
}

#[cfg(target_os = "linux")]
pub use linux::Notifier;

/// Platforms without a native sender keep the command callable.
///
/// Braced rather than a unit struct, so `main` builds it with `default()` on
/// every platform without clippy's `default_constructed_unit_structs`.
#[cfg(not(target_os = "linux"))]
#[derive(Default)]
pub struct Notifier {}

#[cfg(target_os = "windows")]
impl Notifier {
	pub async fn send(
		&self,
		app: &tauri::AppHandle,
		request: &NotificationRequest,
	) -> Result<(), String> {
		use tauri::Manager;
		use tauri_winrt_notification::{IconCrop, Toast};

		// The toast's logo: the bundled 128 px icon next to the executable.
		let logo = app
			.path()
			.resource_dir()
			.ok()
			.map(|dir| dir.join("icons").join("128x128.png"))
			.filter(|path| path.is_file());
		let request = request.clone();
		tauri::async_runtime::spawn_blocking(move || {
			let mut toast = Toast::new(APP_ID)
				.title(&request.title)
				.text1(&request.body);
			if let Some(logo) = logo.as_deref() {
				toast = toast.icon(logo, IconCrop::Square, APP_NAME);
			}
			toast.show().map_err(|error| error.to_string())
		})
		.await
		.map_err(|error| error.to_string())?
	}
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
impl Notifier {
	pub async fn send(
		&self,
		_app: &tauri::AppHandle,
		_request: &NotificationRequest,
	) -> Result<(), String> {
		// macOS: notifications need a signed bundle; nothing is sent yet.
		Ok(())
	}
}

/// Shows a desktop notification on behalf of the frontend.
///
/// Failure is logged and swallowed: a missing notification server must not
/// turn a finished copy into an error.
#[tauri::command]
pub async fn show_desktop_notification(
	app: tauri::AppHandle,
	notifier: tauri::State<'_, Notifier>,
	request: NotificationRequest,
) -> Result<(), String> {
	if let Err(error) = notifier.send(&app, &request).await {
		tracing::warn!(%error, "Desktop notification not shown");
	}
	Ok(())
}

/// Applies the process identity the desktop matches against the desktop file.
///
/// Call before GTK starts. Without it GTK derives the program name from
/// argv[0], so `/usr/bin/wingdrive` produced WM_CLASS `Wingdrive` while the
/// desktop file says `StartupWMClass=WingDrive`, and docks showed a generic
/// window instead of WingDrive.
pub fn apply_process_identity() {
	#[cfg(target_os = "linux")]
	{
		gtk::glib::set_prgname(Some(PROGRAM_NAME));
		gtk::glib::set_application_name(APP_NAME);
	}
	#[cfg(target_os = "windows")]
	{
		use windows_sys::Win32::UI::Shell::SetCurrentProcessExplicitAppUserModelID;
		let id: Vec<u16> = APP_ID.encode_utf16().chain(std::iter::once(0)).collect();
		// Matches the AppUserModelID the installer gives the Start Menu
		// shortcut, so the taskbar groups and toasts show WingDrive.
		unsafe {
			SetCurrentProcessExplicitAppUserModelID(id.as_ptr());
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn request(kind: NotificationKind) -> NotificationRequest {
		NotificationRequest {
			title: "Copy finished".into(),
			body: "Copied <a> & b.txt".into(),
			kind,
			open_label: Some("Show WingDrive".into()),
		}
	}

	fn themed_identity() -> Identity {
		Identity {
			desktop_entry: "wingdrive".into(),
			logo: LogoSource::Themed(ICON_NAME.into()),
		}
	}

	fn hint<'a>(call: &'a NotifyCall, key: &str) -> Option<&'a Hint> {
		call.hints
			.iter()
			.find(|(name, _)| *name == key)
			.map(|(_, hint)| hint)
	}

	#[test]
	fn parses_known_capabilities_and_ignores_the_rest() {
		let caps =
			Capabilities::parse(["body", "actions", "x-vendor", "persistence", "icon-static"]);
		assert!(caps.body && caps.actions && caps.persistence && caps.icon_static);
		assert!(!caps.body_markup);
	}

	#[test]
	fn call_carries_name_desktop_entry_icon_urgency_and_category() {
		let caps = Capabilities::parse(["body", "body-markup", "actions"]);
		let call = build_notify_call(
			&request(NotificationKind::TransferComplete),
			&caps,
			&themed_identity(),
		);
		assert_eq!(call.app_name, "WingDrive");
		assert_eq!(call.app_icon, "WingDrive");
		assert_eq!(
			hint(&call, "desktop-entry"),
			Some(&Hint::Str("wingdrive".into()))
		);
		assert_eq!(hint(&call, "urgency"), Some(&Hint::Byte(1)));
		assert_eq!(
			hint(&call, "category"),
			Some(&Hint::Str("transfer.complete".into()))
		);
		assert_eq!(hint(&call, "image-data"), None);
		assert_eq!(call.expire_timeout, -1);
	}

	#[test]
	fn body_is_escaped_only_for_markup_servers() {
		let markup = Capabilities::parse(["body", "body-markup"]);
		let plain = Capabilities::parse(["body"]);
		let request = request(NotificationKind::TransferComplete);
		assert_eq!(
			build_notify_call(&request, &markup, &themed_identity()).body,
			"Copied &lt;a&gt; &amp; b.txt"
		);
		assert_eq!(
			build_notify_call(&request, &plain, &themed_identity()).body,
			"Copied <a> & b.txt"
		);
	}

	#[test]
	fn servers_without_body_get_the_summary_only() {
		let call = build_notify_call(
			&request(NotificationKind::Info),
			&Capabilities::default(),
			&themed_identity(),
		);
		assert_eq!(call.body, "");
		assert_eq!(call.summary, "Copy finished");
	}

	#[test]
	fn actions_are_sent_only_when_announced() {
		let with = Capabilities::parse(["body", "actions"]);
		let without = Capabilities::parse(["body"]);
		let request = request(NotificationKind::TransferError);
		assert_eq!(
			build_notify_call(&request, &with, &themed_identity()).actions,
			vec!["default".to_owned(), "Show WingDrive".to_owned()]
		);
		assert!(build_notify_call(&request, &without, &themed_identity())
			.actions
			.is_empty());
	}

	#[test]
	fn blank_action_label_falls_back_to_the_app_name() {
		let mut request = request(NotificationKind::TransferComplete);
		request.open_label = Some("  ".into());
		let call = build_notify_call(
			&request,
			&Capabilities::parse(["actions"]),
			&themed_identity(),
		);
		assert_eq!(
			call.actions,
			vec!["default".to_owned(), "WingDrive".to_owned()]
		);
	}

	#[test]
	fn missing_theme_icon_sends_pixels_capped_at_128() {
		let identity = Identity {
			desktop_entry: "WingDrive".into(),
			logo: LogoSource::Pixels(RgbaImage {
				width: 512,
				height: 512,
				rgba: vec![255; 512 * 512 * 4],
			}),
		};
		let call = build_notify_call(
			&request(NotificationKind::Info),
			&Capabilities::minimal(),
			&identity,
		);
		assert_eq!(call.app_icon, "");
		match hint(&call, "image-data") {
			Some(Hint::Image(image)) => {
				assert_eq!((image.width, image.height), (128, 128));
				assert_eq!(image.rgba.len(), 128 * 128 * 4);
			}
			other => panic!("expected image-data, got {other:?}"),
		}
		assert_eq!(hint(&call, "urgency"), Some(&Hint::Byte(0)));
		assert_eq!(hint(&call, "category"), None);
	}

	#[test]
	fn fit_keeps_aspect_ratio_and_small_images() {
		let wide = RgbaImage {
			width: 256,
			height: 64,
			rgba: vec![0; 256 * 64 * 4],
		};
		let fitted = wide.fit(128);
		assert_eq!((fitted.width, fitted.height), (128, 32));
		let small = RgbaImage {
			width: 32,
			height: 32,
			rgba: vec![1; 32 * 32 * 4],
		};
		assert_eq!(small.fit(128), small);
	}

	#[test]
	fn portal_notification_names_the_themed_icon() {
		let fields =
			build_portal_notification(&request(NotificationKind::TransferError), ICON_NAME);
		assert!(fields.contains(&("title", PortalValue::Str("Copy finished".into()))));
		assert!(fields.contains(&("priority", PortalValue::Str("normal".into()))));
		assert!(fields.contains(&(
			"icon",
			PortalValue::ThemedIcon(vec!["WingDrive".into(), "WingDrive-symbolic".into()])
		)));
	}

	#[test]
	fn data_dirs_follow_xdg_defaults() {
		let home = Path::new("/home/u");
		assert_eq!(
			xdg_data_dirs(None, None, Some(home)),
			vec![
				PathBuf::from("/home/u/.local/share"),
				PathBuf::from("/usr/local/share"),
				PathBuf::from("/usr/share"),
			]
		);
		assert_eq!(
			xdg_data_dirs(Some("/x"), Some("/a::/b"), Some(home)),
			vec![
				PathBuf::from("/x"),
				PathBuf::from("/a"),
				PathBuf::from("/b")
			]
		);
	}

	#[test]
	fn desktop_entry_matches_the_installed_file() {
		let dirs = vec![
			PathBuf::from("/home/u/.local/share"),
			PathBuf::from("/usr/share"),
		];
		let aur = |path: &Path| path == Path::new("/usr/share/applications/wingdrive.desktop");
		let deb = |path: &Path| path == Path::new("/usr/share/applications/WingDrive.desktop");
		assert_eq!(resolve_desktop_entry(&dirs, aur), "wingdrive");
		assert_eq!(resolve_desktop_entry(&dirs, deb), "WingDrive");
		assert_eq!(resolve_desktop_entry(&dirs, |_| false), "WingDrive");
	}

	#[test]
	fn theme_icon_is_found_in_any_hicolor_size() {
		let dirs = vec![PathBuf::from("/usr/share")];
		let installed = |path: &Path| {
			path == Path::new("/usr/share/icons/hicolor/256x256@2/apps/WingDrive.png")
		};
		assert!(theme_has_icon(&dirs, ICON_NAME, installed));
		assert!(!theme_has_icon(&dirs, ICON_NAME, |_| false));
	}

	#[test]
	fn identity_matches_the_bundle_config() {
		let config: serde_json::Value =
			serde_json::from_str(include_str!("../tauri.conf.json")).unwrap();
		// The installer registers the Start Menu shortcut under the identifier.
		assert_eq!(config["identifier"], APP_ID);
		assert_eq!(config["productName"], APP_NAME);
		// The bundler names the hicolor icons and StartupWMClass after the binary.
		assert!(include_str!("../Cargo.toml").contains(&format!("name = \"{PROGRAM_NAME}\"")));
		assert_eq!(ICON_NAME, PROGRAM_NAME);
		let template = include_str!("../wingdrive.desktop");
		assert!(template.contains("StartupWMClass={{exec}}"));
		assert!(template.contains("Icon={{icon}}"));
	}

	#[test]
	fn request_deserializes_from_the_frontend_shape() {
		let request: NotificationRequest = serde_json::from_str(
			r#"{"title":"Done","body":"x","kind":"transferError","openLabel":"Open"}"#,
		)
		.unwrap();
		assert_eq!(request.kind, NotificationKind::TransferError);
		assert_eq!(request.open_label.as_deref(), Some("Open"));
	}
}
