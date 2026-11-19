use super::super::{claim_name, OpenRequest, BUS_NAME, OBJECT_PATH};
use crate::test_support::PrivateBus;

#[tokio::test]
async fn private_bus_dispatches_folders_items_and_properties() {
	let Some(bus) = PrivateBus::start() else {
		eprintln!("dbus-daemon unavailable; skipping FileManager1 contract test");
		return;
	};
	let dir = tempfile::tempdir().unwrap();
	let path = dir.path().join("report space.txt");
	std::fs::write(&path, "report").unwrap();
	let uris = vec![
		tauri::Url::from_directory_path(dir.path())
			.unwrap()
			.to_string(),
		tauri::Url::from_file_path(&path).unwrap().to_string(),
	];
	let selected = OpenRequest {
		directory: dir.path().to_path_buf(),
		select: Some(path),
	};
	let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
	let _server = claim_name(bus.builder(), move |requests| {
		tx.send(requests).unwrap();
	})
	.await
	.unwrap();
	let client = bus.connect().await;
	let proxy = zbus::Proxy::new(&client, BUS_NAME, OBJECT_PATH, BUS_NAME)
		.await
		.unwrap();

	for method in ["ShowFolders", "ShowItems", "ShowItemProperties"] {
		proxy.call::<_, _, ()>(method, &(&uris, "")).await.unwrap();
		let requests = tokio::time::timeout(std::time::Duration::from_secs(2), rx.recv())
			.await
			.unwrap()
			.unwrap();
		let expected_first = if method == "ShowFolders" {
			OpenRequest {
				directory: dir.path().to_path_buf(),
				select: None,
			}
		} else {
			OpenRequest {
				directory: dir.path().parent().unwrap().to_path_buf(),
				select: Some(dir.path().to_path_buf()),
			}
		};
		assert_eq!(requests, [expected_first, selected.clone()]);
	}
}
