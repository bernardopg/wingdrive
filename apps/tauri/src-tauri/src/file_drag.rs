//! # Dragging Files Out to Other Apps
//!
//! Starts an OS drag carrying `text/uri-list`, so browsers, chat apps, editors
//! and other file managers receive the files. The in-app drag (dnd-kit) hands
//! over to this when the pointer leaves the window. The target is limited to
//! other apps: drops back into WingDrive go through its own drag and drop.

#[cfg(target_os = "linux")]
use gtk::{gdk, glib::SignalHandlerId, prelude::*, TargetEntry, TargetFlags, TargetList};
#[cfg(target_os = "linux")]
use std::{cell::RefCell, rc::Rc};

#[tauri::command]
pub async fn start_file_drag(
	window: tauri::WebviewWindow,
	paths: Vec<String>,
) -> Result<(), String> {
	if paths.is_empty() {
		return Err("Nothing to drag".to_string());
	}

	#[cfg(target_os = "linux")]
	{
		let uris = crate::clipboard::file_uris(&paths)?;
		let icon = if paths.len() == 1 && std::path::Path::new(&paths[0]).is_dir() {
			"folder"
		} else {
			"text-x-generic"
		};
		let (tx, rx) = tokio::sync::oneshot::channel();
		let target = window.clone();
		window
			.run_on_main_thread(move || {
				let _ = tx.send(begin_gtk_drag(&target, uris, icon));
			})
			.map_err(|e| format!("Failed to reach the main thread: {e}"))?;
		rx.await
			.map_err(|_| "The drag was dropped before it started".to_string())?
	}

	#[cfg(not(target_os = "linux"))]
	{
		let _ = window;
		Err("Dragging files to other apps is not available on this platform yet".to_string())
	}
}

#[cfg(target_os = "linux")]
fn begin_gtk_drag(
	window: &tauri::WebviewWindow,
	uris: Vec<String>,
	icon: &str,
) -> Result<(), String> {
	let gtk_window = window.gtk_window().map_err(|e| e.to_string())?;
	let targets = TargetList::new(&[TargetEntry::new("text/uri-list", TargetFlags::OTHER_APP, 0)]);

	// Both handlers go when the drag ends, dropped or cancelled.
	let handlers: Rc<RefCell<Vec<SignalHandlerId>>> = Rc::default();
	let data_get = gtk_window.connect_drag_data_get(move |_, _, data, _, _| {
		let refs: Vec<&str> = uris.iter().map(String::as_str).collect();
		if !data.set_uris(&refs) {
			tracing::warn!("Drop target rejected the file URI list");
		}
	});
	handlers.borrow_mut().push(data_get);
	let ended = handlers.clone();
	let drag_end = gtk_window.connect_drag_end(move |widget, _| {
		for id in ended.borrow_mut().drain(..) {
			widget.disconnect(id);
		}
	});
	handlers.borrow_mut().push(drag_end);

	let actions = gdk::DragAction::COPY | gdk::DragAction::MOVE | gdk::DragAction::LINK;
	match gtk_window.drag_begin_with_coordinates(&targets, actions, 1, None, -1, -1) {
		Some(context) => {
			context.drag_set_icon_name(icon, 0, 0);
			Ok(())
		}
		None => {
			for id in handlers.borrow_mut().drain(..) {
				gtk_window.disconnect(id);
			}
			Err("GTK refused to start the drag".to_string())
		}
	}
}
