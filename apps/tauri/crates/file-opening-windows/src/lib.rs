#![cfg(target_os = "windows")]

use file_opening::{FileOpener, OpenResult, OpenWithApp};
use std::path::Path;
use windows::core::*;
use windows::Win32::System::Com::*;
use windows::Win32::UI::Shell::*;
use windows::Win32::UI::WindowsAndMessaging::*;

// Thread-local COM initialization
thread_local! {
	static COM_INITIALIZED: std::cell::RefCell<bool> = const { std::cell::RefCell::new(false) };
}

fn ensure_com_initialized() {
	COM_INITIALIZED.with(|initialized| {
		if !*initialized.borrow() {
			unsafe {
				let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
			}
			*initialized.borrow_mut() = true;
		}
	});
}

/// The extension Windows associates a file with, such as ".txt", or None without one.
fn association_key(path: &Path) -> Option<String> {
	path.extension()
		.and_then(|extension| extension.to_str())
		.filter(|extension| !extension.is_empty())
		.map(|extension| format!(".{extension}"))
}

/// Copies a string the shell allocated with the COM allocator, then frees it.
///
/// `IAssocHandler::GetName` and `GetUIName` hand over their buffers; they leaked before.
unsafe fn take_co_string(value: PWSTR) -> std::result::Result<String, String> {
	let text = value.to_string().map_err(|e| e.to_string());
	CoTaskMemFree(Some(value.0 as *const _));
	text
}

/// One handler from `SHAssocEnumHandlers`.
struct Handler {
	/// Full path of the program, which `open_with_app` receives back as the app id.
	path: String,
	/// The name Windows shows in its own "Open with" menu.
	display_name: String,
	handler: IAssocHandler,
}

/// The recommended handlers for an extension, in the shell's order.
fn handlers_for(extension: &str) -> std::result::Result<Vec<Handler>, String> {
	let mut handlers = Vec::new();
	unsafe {
		let list = SHAssocEnumHandlers(&HSTRING::from(extension), ASSOC_FILTER_RECOMMENDED)
			.map_err(|e| e.to_string())?;
		loop {
			let mut fetched_handlers: [Option<IAssocHandler>; 1] = [None];
			let mut fetched = 0u32;
			if list
				.Next(&mut fetched_handlers, Some(&mut fetched))
				.is_err()
				|| fetched == 0
			{
				break;
			}
			let Some(handler) = fetched_handlers[0].take() else {
				continue;
			};
			let path = take_co_string(handler.GetName().map_err(|e| e.to_string())?)?;
			// A handler without a display name still opens files; show its program instead.
			let display_name = handler
				.GetUIName()
				.ok()
				.and_then(|name| take_co_string(name).ok())
				.filter(|name| !name.is_empty())
				.unwrap_or_else(|| program_name(&path));
			handlers.push(Handler {
				path,
				display_name,
				handler,
			});
		}
	}
	Ok(handlers)
}

/// "notepad" for "C:\Windows\system32\notepad.exe".
fn program_name(path: &str) -> String {
	Path::new(path)
		.file_stem()
		.map(|stem| stem.to_string_lossy().into_owned())
		.unwrap_or_else(|| path.to_string())
}

/// The program that opens `extension` on a double click, as a full path.
fn default_program(extension: &str) -> Option<String> {
	unsafe {
		let mut length = 0u32;
		// The first call measures the buffer, the second fills it.
		let _ = AssocQueryStringW(
			ASSOCF_NONE,
			ASSOCSTR_EXECUTABLE,
			&HSTRING::from(extension),
			w!("open"),
			PWSTR::null(),
			&mut length,
		);
		if length == 0 {
			return None;
		}
		let mut buffer = vec![0u16; length as usize];
		AssocQueryStringW(
			ASSOCF_NONE,
			ASSOCSTR_EXECUTABLE,
			&HSTRING::from(extension),
			w!("open"),
			PWSTR(buffer.as_mut_ptr()),
			&mut length,
		)
		.ok()
		.ok()?;
		let end = buffer
			.iter()
			.position(|&unit| unit == 0)
			.unwrap_or(buffer.len());
		Some(String::from_utf16_lossy(&buffer[..end]))
	}
}

/// Lists the apps for an extension: the default first, then the others by name.
fn list_apps_for_extension(extension: &str) -> std::result::Result<Vec<OpenWithApp>, String> {
	let default = default_program(extension);
	let mut apps: Vec<OpenWithApp> = handlers_for(extension)?
		.into_iter()
		.map(|handler| OpenWithApp {
			is_default: default
				.as_deref()
				.is_some_and(|program| program.eq_ignore_ascii_case(&handler.path)),
			id: handler.path,
			name: handler.display_name,
			icon: None,
		})
		.collect();
	order_apps(&mut apps);
	Ok(apps)
}

/// Default app first, then by name; the Explorer's "Open with" menu shows them in this order.
fn order_apps(apps: &mut [OpenWithApp]) {
	apps.sort_by(|a, b| {
		b.is_default
			.cmp(&a.is_default)
			.then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
	});
}

pub struct WindowsFileOpener;

impl FileOpener for WindowsFileOpener {
	fn get_apps_for_file(&self, path: &Path) -> std::result::Result<Vec<OpenWithApp>, String> {
		ensure_com_initialized();
		match association_key(path) {
			Some(extension) => list_apps_for_extension(&extension),
			None => Ok(vec![]),
		}
	}

	fn open_with_default(&self, path: &Path) -> std::result::Result<OpenResult, String> {
		ensure_com_initialized();

		let path_str = path.to_string_lossy();
		let h_path = HSTRING::from(&*path_str);

		unsafe {
			let result = ShellExecuteW(None, w!("open"), &h_path, None, None, SW_SHOWNORMAL);

			if result.0 as isize > 32 {
				Ok(OpenResult::Success)
			} else {
				Ok(OpenResult::PlatformError {
					message: format!("ShellExecute failed with code {}", result.0 as isize),
				})
			}
		}
	}

	fn open_with_app(&self, path: &Path, app_id: &str) -> std::result::Result<OpenResult, String> {
		ensure_com_initialized();

		let Some(extension) = association_key(path) else {
			return Ok(OpenResult::PlatformError {
				message: "File has no extension".to_string(),
			});
		};

		// The app id is the program path that get_apps_for_file returned.
		let Some(handler) = handlers_for(&extension)?
			.into_iter()
			.find(|handler| handler.path.eq_ignore_ascii_case(app_id))
		else {
			return Ok(OpenResult::AppNotFound {
				app_id: app_id.to_string(),
			});
		};

		if !path.exists() {
			return Ok(OpenResult::FileNotFound {
				path: path.to_string_lossy().into_owned(),
			});
		}

		unsafe {
			let h_path = HSTRING::from(&*path.to_string_lossy());
			let shell_item: IShellItem =
				SHCreateItemFromParsingName(&h_path, None).map_err(|e| e.to_string())?;
			let data_object: IDataObject = shell_item
				.BindToHandler(None, &BHID_DataObject)
				.map_err(|e| e.to_string())?;
			handler
				.handler
				.Invoke(&data_object)
				.map_err(|e| e.to_string())?;
		}
		Ok(OpenResult::Success)
	}
}

// These run on the Windows CI runner (scripts/ci-rust.sh tests this crate there). They query
// the registry only and never start a program.
#[cfg(test)]
mod tests {
	use super::*;
	use std::path::PathBuf;

	fn app(name: &str, is_default: bool) -> OpenWithApp {
		OpenWithApp {
			id: format!(r"C:\Apps\{name}.exe"),
			name: name.to_string(),
			icon: None,
			is_default,
		}
	}

	#[test]
	fn association_key_is_the_dotted_extension() {
		assert_eq!(
			association_key(Path::new(r"C:\a\notes.TXT")).as_deref(),
			Some(".TXT")
		);
		assert_eq!(association_key(Path::new(r"C:\a\Makefile")), None);
		assert_eq!(association_key(Path::new(r"C:\a\trailing.")), None);
	}

	#[test]
	fn default_app_comes_first_then_names_in_order() {
		let mut apps = vec![app("Zed", false), app("notepad", false), app("Word", true)];
		order_apps(&mut apps);
		let names: Vec<_> = apps.iter().map(|app| app.name.as_str()).collect();
		assert_eq!(names, ["Word", "notepad", "Zed"]);
	}

	#[test]
	fn program_name_drops_the_folder_and_extension() {
		assert_eq!(program_name(r"C:\Windows\system32\notepad.exe"), "notepad");
	}

	#[test]
	fn text_files_list_named_programs() {
		let apps = WindowsFileOpener
			.get_apps_for_file(&PathBuf::from(r"C:\missing\notes.txt"))
			.expect("listing .txt handlers");
		assert!(!apps.is_empty(), "every Windows install opens .txt files");
		for app in &apps {
			assert!(!app.name.is_empty(), "{app:?} has no name");
			// The id is the program path that open_with_app looks up again.
			assert!(Path::new(&app.id).is_absolute(), "{app:?} id is not a path");
		}
		assert!(apps.iter().filter(|app| app.is_default).count() <= 1);
	}

	#[test]
	fn files_without_an_extension_have_no_apps() {
		let apps = WindowsFileOpener
			.get_apps_for_file(&PathBuf::from(r"C:\missing\Makefile"))
			.unwrap();
		assert!(apps.is_empty());
	}

	#[test]
	fn unknown_app_is_reported_without_opening_anything() {
		let result = WindowsFileOpener
			.open_with_app(
				&PathBuf::from(r"C:\missing\notes.txt"),
				r"C:\no\such\program.exe",
			)
			.unwrap();
		assert!(
			matches!(result, OpenResult::AppNotFound { .. }),
			"{result:?}"
		);
	}

	#[test]
	fn known_app_on_a_missing_file_reports_the_file() {
		let path = PathBuf::from(r"C:\wingdrive-missing-folder\notes.txt");
		let Some(app) = WindowsFileOpener
			.get_apps_for_file(&path)
			.unwrap()
			.into_iter()
			.next()
		else {
			return;
		};
		let result = WindowsFileOpener.open_with_app(&path, &app.id).unwrap();
		assert!(
			matches!(result, OpenResult::FileNotFound { .. }),
			"{result:?}"
		);
	}
}
