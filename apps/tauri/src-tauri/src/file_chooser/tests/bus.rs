//! The backend on a private session bus: a `dbus-daemon` this test starts, so
//! the user's bus and portal never see it. Skipped where `dbus-daemon` is missing.

use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::time::Duration;

use zbus::{ObjectPath, OwnedVariant as OwnedValue, Variant as Value};

use super::super::portal::{FileChooser, BUS_NAME, OBJECT_PATH};
use super::super::*;
use super::FakeHost;

const IFACE: &str = "org.freedesktop.impl.portal.FileChooser";

struct PrivateBus {
	daemon: Child,
	address: String,
}

impl PrivateBus {
	fn start() -> Option<Self> {
		let mut daemon = Command::new("dbus-daemon")
			.args(["--session", "--nofork", "--print-address=1"])
			.stdout(Stdio::piped())
			.stderr(Stdio::null())
			.spawn()
			.ok()?;
		let mut address = String::new();
		BufReader::new(daemon.stdout.take()?)
			.read_line(&mut address)
			.ok()?;
		Some(Self {
			daemon,
			address: address.trim().to_string(),
		})
	}

	async fn connect(&self) -> zbus::Connection {
		zbus::connection::Builder::address(self.address.as_str())
			.build()
			.await
			.unwrap()
	}
}

impl Drop for PrivateBus {
	fn drop(&mut self) {
		self.daemon.kill().ok();
		self.daemon.wait().ok();
	}
}

struct Backend {
	bus: PrivateBus,
	host: FakeHost,
	broker: Arc<Broker>,
	_server: zbus::Connection,
	client: zbus::Connection,
}

async fn backend() -> Option<Backend> {
	let Some(bus) = PrivateBus::start() else {
		eprintln!("dbus-daemon unavailable; skipping");
		return None;
	};
	let host = FakeHost::default();
	let broker = Broker::new(host.clone());
	let server = zbus::connection::Builder::address(bus.address.as_str())
		.serve_at(
			OBJECT_PATH,
			FileChooser::new(broker.clone(), std::env::temp_dir()),
		)
		.name(BUS_NAME)
		.build()
		.await
		.unwrap();
	let client = bus.connect().await;
	Some(Backend {
		bus,
		host,
		broker,
		_server: server,
		client,
	})
}

type Reply = (u32, HashMap<String, OwnedValue>);

/// Calls a FileChooser method the way `xdg-desktop-portal` does.
fn call(
	client: &zbus::Connection,
	method: &'static str,
	handle: &'static str,
	options: HashMap<&'static str, Value<'static>>,
) -> tokio::task::JoinHandle<Reply> {
	let client = client.clone();
	tokio::spawn(async move {
		let reply = client
			.call_method(
				Some(BUS_NAME),
				OBJECT_PATH,
				Some(IFACE),
				method,
				&(
					ObjectPath::try_from(handle).unwrap(),
					"org.example.App",
					"x11:1a",
					"Pick something",
					options,
				),
			)
			.await
			.unwrap();
		reply.body().deserialize::<Reply>().unwrap()
	})
}

async fn shown(host: &FakeHost, count: usize) -> PickerRequest {
	for _ in 0..400 {
		if let Some(request) = host.shown.lock().unwrap().get(count - 1).cloned() {
			return request;
		}
		tokio::time::sleep(Duration::from_millis(5)).await;
	}
	panic!("the picker never opened");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn the_backend_owns_its_name_and_serves_the_interface() {
	let Some(backend) = backend().await else {
		return;
	};
	let dbus = zbus::fdo::DBusProxy::new(&backend.client).await.unwrap();
	assert!(dbus
		.name_has_owner(BUS_NAME.try_into().unwrap())
		.await
		.unwrap());
	let introspection = zbus::fdo::IntrospectableProxy::builder(&backend.client)
		.destination(BUS_NAME)
		.path(OBJECT_PATH)
		.build()
		.await
		.unwrap()
		.introspect()
		.await
		.unwrap();
	for name in [IFACE, "OpenFile", "SaveFile", "SaveFiles"] {
		assert!(introspection.contains(name), "{name}");
	}
	drop(backend.bus);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn open_file_returns_the_chosen_uris() {
	let Some(backend) = backend().await else {
		return;
	};
	let dir = tempfile::tempdir().unwrap();
	let file = dir.path().join("photo one.png");
	std::fs::write(&file, "x").unwrap();
	let mut folder = dir.path().as_os_str().as_encoded_bytes().to_vec();
	folder.push(0);
	let task = call(
		&backend.client,
		"OpenFile",
		"/org/freedesktop/portal/desktop/request/1_1/t1",
		HashMap::from([
			("multiple", Value::from(true)),
			("current_folder", Value::from(folder)),
			(
				"filters",
				Value::from(vec![(
					"Images".to_string(),
					vec![(0u32, "*.png".to_string())],
				)]),
			),
		]),
	);
	let request = shown(&backend.host, 1).await;
	assert_eq!(request.mode, PickerMode::Open);
	assert_eq!(request.app_id, "org.example.App");
	assert_eq!(request.parent_window, "x11:1a");
	assert_eq!(request.title, "Pick something");
	assert_eq!(request.start_folder, dir.path());
	assert!(request.options.multiple);
	assert_eq!(request.options.filters[0].globs, ["*.png"]);

	backend
		.broker
		.respond(
			&request.id,
			Selection::Open {
				paths: vec![file.clone()],
				choices: vec![],
				current_filter: Some(0),
			},
		)
		.unwrap();
	let (response, results) = task.await.unwrap();
	assert_eq!(response, 0);
	let uris = Vec::<String>::try_from(results["uris"].try_clone().unwrap()).unwrap();
	assert_eq!(uris, [options::file_uri(&file)]);
	assert!(uris[0].ends_with("photo%20one.png"));
	assert!(results.contains_key("current_filter"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn closing_the_request_cancels_the_picker() {
	let Some(backend) = backend().await else {
		return;
	};
	let handle = "/org/freedesktop/portal/desktop/request/1_1/t2";
	let task = call(
		&backend.client,
		"SaveFile",
		handle,
		HashMap::from([("current_name", Value::from("report.pdf"))]),
	);
	let request = shown(&backend.host, 1).await;
	assert_eq!(request.suggested_name.as_deref(), Some("report.pdf"));

	backend
		.client
		.call_method(
			Some(BUS_NAME),
			handle,
			Some("org.freedesktop.impl.portal.Request"),
			"Close",
			&(),
		)
		.await
		.unwrap();
	let (response, results) = task.await.unwrap();
	assert_eq!(response, 1);
	assert!(results.is_empty());
	assert_eq!(*backend.host.dismissed.lock().unwrap(), [request.id]);

	// The request object goes away with the call.
	let closed_again = backend
		.client
		.call_method(
			Some(BUS_NAME),
			handle,
			Some("org.freedesktop.impl.portal.Request"),
			"Close",
			&(),
		)
		.await;
	assert!(closed_again.is_err());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn save_files_and_parallel_requests_answer_independently() {
	let Some(backend) = backend().await else {
		return;
	};
	let dir = tempfile::tempdir().unwrap();
	std::fs::write(dir.path().join("a.txt"), "x").unwrap();
	let names: Vec<Vec<u8>> = vec![b"a.txt\0".to_vec(), b"b.txt\0".to_vec()];
	let first = call(
		&backend.client,
		"SaveFiles",
		"/org/freedesktop/portal/desktop/request/1_1/t3",
		HashMap::from([("files", Value::from(names))]),
	);
	let first_request = shown(&backend.host, 1).await;
	let second = call(
		&backend.client,
		"OpenFile",
		"/org/freedesktop/portal/desktop/request/1_1/t4",
		HashMap::new(),
	);
	let second_request = shown(&backend.host, 2).await;
	assert_ne!(first_request.id, second_request.id);

	backend.broker.cancel(&second_request.id);
	assert_eq!(second.await.unwrap().0, 1);

	backend
		.broker
		.respond(
			&first_request.id,
			Selection::SaveFiles {
				folder: dir.path().into(),
				choices: vec![],
			},
		)
		.unwrap();
	let (response, results) = first.await.unwrap();
	assert_eq!(response, 0);
	let uris = Vec::<String>::try_from(results["uris"].try_clone().unwrap()).unwrap();
	assert_eq!(
		uris,
		[
			options::file_uri(&dir.path().join("a (1).txt")),
			options::file_uri(&dir.path().join("b.txt")),
		]
	);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn invalid_options_and_a_failed_window_answer_two() {
	let Some(backend) = backend().await else {
		return;
	};
	let bad = call(
		&backend.client,
		"OpenFile",
		"/org/freedesktop/portal/desktop/request/1_1/t5",
		HashMap::from([("multiple", Value::from("yes"))]),
	);
	assert_eq!(bad.await.unwrap().0, 2);
	assert!(backend.host.shown.lock().unwrap().is_empty());

	let failing = FakeHost {
		fail: true,
		..FakeHost::default()
	};
	let broker = Broker::new(failing);
	let server = zbus::connection::Builder::address(backend.bus.address.as_str())
		.serve_at(OBJECT_PATH, FileChooser::new(broker, std::env::temp_dir()))
		.build()
		.await
		.unwrap();
	let reply = backend
		.client
		.call_method(
			server.unique_name().map(|name| name.to_string()).as_deref(),
			OBJECT_PATH,
			Some(IFACE),
			"OpenFile",
			&(
				ObjectPath::try_from("/org/freedesktop/portal/desktop/request/1_1/t6").unwrap(),
				"",
				"",
				"",
				HashMap::<&str, Value<'_>>::new(),
			),
		)
		.await
		.unwrap();
	assert_eq!(reply.body().deserialize::<Reply>().unwrap().0, 2);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn connect_serves_the_backend_on_the_given_bus() {
	let Some(bus) = PrivateBus::start() else {
		eprintln!("dbus-daemon unavailable; skipping");
		return;
	};
	let broker = Broker::new(FakeHost::default());
	let builder = zbus::connection::Builder::address(bus.address.as_str());
	let _server = super::super::portal::connect(builder, broker)
		.await
		.unwrap();
	let client = bus.connect().await;
	let dbus = zbus::fdo::DBusProxy::new(&client).await.unwrap();
	assert!(dbus
		.name_has_owner(BUS_NAME.try_into().unwrap())
		.await
		.unwrap());
}
