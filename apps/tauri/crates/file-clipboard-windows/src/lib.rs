//! # Windows File Clipboard
//!
//! Explorer exchanges files through `CF_HDROP`: a `DROPFILES` header followed by
//! NUL-terminated UTF-16 paths and one more NUL. A cut is marked with the
//! `Preferred DropEffect` format, so pasting a cut from WingDrive into Explorer
//! moves the files, and the other way round.
//!
//! The encoding is plain byte handling outside `cfg(windows)`, so its tests run
//! on every platform. Only [`write`] and [`read`] call the Win32 clipboard.

/// Size of the `DROPFILES` header: `pFiles`, `pt.x`, `pt.y`, `fNC` and `fWide`.
const DROPFILES_SIZE: usize = 20;

/// `DROPEFFECT_COPY`.
pub const DROPEFFECT_COPY: u32 = 1;
/// `DROPEFFECT_MOVE`, which Explorer writes for a cut.
pub const DROPEFFECT_MOVE: u32 = 2;
/// `DROPEFFECT_LINK`. Explorer offers it together with a copy.
pub const DROPEFFECT_LINK: u32 = 4;

/// Encodes absolute paths as a wide `DROPFILES` block.
///
/// Refuses relative paths and paths with a NUL, because the receiver resolves
/// names against its own folder and splits the list on NUL.
pub fn encode_drop_files(paths: &[String]) -> Result<Vec<u8>, String> {
	let mut bytes = Vec::with_capacity(DROPFILES_SIZE + 2);
	bytes.extend_from_slice(&(DROPFILES_SIZE as u32).to_le_bytes());
	bytes.extend_from_slice(&0i32.to_le_bytes());
	bytes.extend_from_slice(&0i32.to_le_bytes());
	bytes.extend_from_slice(&0u32.to_le_bytes());
	bytes.extend_from_slice(&1u32.to_le_bytes());
	for path in paths {
		if !is_absolute(path) {
			return Err(format!("Clipboard path must be absolute: {path}"));
		}
		if path.contains('\0') {
			return Err("Clipboard path contains a NUL character".to_string());
		}
		for unit in path.encode_utf16() {
			bytes.extend_from_slice(&unit.to_le_bytes());
		}
		bytes.extend_from_slice(&[0, 0]);
	}
	bytes.extend_from_slice(&[0, 0]);
	Ok(bytes)
}

/// Decodes a `DROPFILES` block written by any program.
///
/// The block can be longer than its list, because `GlobalSize` rounds up, so
/// decoding stops at the first empty name.
pub fn decode_drop_files(bytes: &[u8]) -> Result<Vec<String>, String> {
	let header = bytes
		.get(..DROPFILES_SIZE)
		.ok_or("The file clipboard is truncated")?;
	let offset = read_u32(&header[0..4]) as usize;
	if offset < DROPFILES_SIZE {
		return Err("The file clipboard has an invalid header".to_string());
	}
	if read_u32(&header[16..20]) == 0 {
		return Err("The file clipboard uses ANSI file names".to_string());
	}
	let units: Vec<u16> = bytes
		.get(offset..)
		.ok_or("The file clipboard is truncated")?
		.as_chunks::<2>()
		.0
		.iter()
		.map(|&pair| u16::from_le_bytes(pair))
		.collect();

	let mut paths = Vec::new();
	for name in units.split(|&unit| unit == 0) {
		if name.is_empty() {
			break;
		}
		paths
			.push(String::from_utf16(name).map_err(|_| "Clipboard file name is not valid UTF-16")?);
	}
	Ok(paths)
}

/// `Preferred DropEffect` value for a copy or a cut, as Explorer writes it.
pub fn drop_effect(cut: bool) -> u32 {
	if cut {
		DROPEFFECT_MOVE
	} else {
		DROPEFFECT_COPY | DROPEFFECT_LINK
	}
}

/// True when a `Preferred DropEffect` value asks for a move.
pub fn is_cut(effect: u32) -> bool {
	effect & DROPEFFECT_MOVE != 0 && effect & DROPEFFECT_COPY == 0
}

/// A drive path such as `C:\dir` or a UNC path such as `\\server\share`.
fn is_absolute(path: &str) -> bool {
	let bytes = path.as_bytes();
	let drive = bytes.len() >= 3
		&& bytes[0].is_ascii_alphabetic()
		&& bytes[1] == b':'
		&& matches!(bytes[2], b'\\' | b'/');
	drive || path.starts_with(r"\\")
}

fn read_u32(bytes: &[u8]) -> u32 {
	u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
}

#[cfg(windows)]
mod win32 {
	use std::time::Duration;

	use windows_sys::Win32::{
		Foundation::{GlobalFree, HANDLE, HGLOBAL},
		System::{
			DataExchange::{
				CloseClipboard, EmptyClipboard, GetClipboardData, OpenClipboard,
				RegisterClipboardFormatW, SetClipboardData,
			},
			Memory::{GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock, GMEM_MOVEABLE},
			Ole::CF_HDROP,
		},
	};

	use super::{decode_drop_files, drop_effect, encode_drop_files, is_cut};

	/// Keeps the clipboard open and closes it on every return path.
	struct OpenClipboardGuard;

	impl OpenClipboardGuard {
		fn open() -> Result<Self, String> {
			// Another program can hold the clipboard for a few milliseconds.
			for _ in 0..10 {
				// SAFETY: no owner window; the guard closes the clipboard.
				if unsafe { OpenClipboard(0) } != 0 {
					return Ok(Self);
				}
				std::thread::sleep(Duration::from_millis(20));
			}
			Err("Another program is using the clipboard".to_string())
		}
	}

	impl Drop for OpenClipboardGuard {
		fn drop(&mut self) {
			// SAFETY: this guard opened the clipboard.
			unsafe { CloseClipboard() };
		}
	}

	fn preferred_drop_effect() -> u32 {
		let name: Vec<u16> = "Preferred DropEffect"
			.encode_utf16()
			.chain(Some(0))
			.collect();
		// SAFETY: `name` is NUL-terminated and outlives the call.
		unsafe { RegisterClipboardFormatW(name.as_ptr()) }
	}

	/// Places `bytes` on the open clipboard. On success the clipboard owns the memory.
	fn set_bytes(format: u32, bytes: &[u8]) -> Result<(), String> {
		// SAFETY: the block is allocated with `bytes.len()` bytes and freed on every failure.
		unsafe {
			let memory: HGLOBAL = GlobalAlloc(GMEM_MOVEABLE, bytes.len());
			if memory.is_null() {
				return Err("Could not allocate clipboard memory".to_string());
			}
			let target = GlobalLock(memory);
			if target.is_null() {
				GlobalFree(memory);
				return Err("Could not lock clipboard memory".to_string());
			}
			std::ptr::copy_nonoverlapping(bytes.as_ptr(), target.cast::<u8>(), bytes.len());
			GlobalUnlock(memory);
			if SetClipboardData(format, memory as HANDLE) == 0 {
				GlobalFree(memory);
				return Err("Could not write the file clipboard".to_string());
			}
		}
		Ok(())
	}

	/// Copies one format out of the open clipboard.
	fn get_bytes(format: u32) -> Option<Vec<u8>> {
		// SAFETY: the clipboard is open, and the block is read within `GlobalSize` while locked.
		unsafe {
			let handle = GetClipboardData(format);
			if handle == 0 {
				return None;
			}
			let memory = handle as HGLOBAL;
			let size = GlobalSize(memory);
			let source = GlobalLock(memory);
			if source.is_null() {
				return None;
			}
			let bytes = std::slice::from_raw_parts(source.cast::<u8>(), size).to_vec();
			GlobalUnlock(memory);
			Some(bytes)
		}
	}

	/// Replaces the clipboard with `paths`, or clears it when `paths` is empty.
	pub fn write(paths: &[String], cut: bool) -> Result<(), String> {
		// Validate before taking the clipboard, so a bad path leaves it as it was.
		let files = encode_drop_files(paths)?;
		let _clipboard = OpenClipboardGuard::open()?;
		// SAFETY: the clipboard is open.
		if unsafe { EmptyClipboard() } == 0 {
			return Err("Could not own the file clipboard".to_string());
		}
		if paths.is_empty() {
			return Ok(());
		}
		set_bytes(CF_HDROP as u32, &files)?;
		set_bytes(preferred_drop_effect(), &drop_effect(cut).to_le_bytes())
	}

	/// Files on the clipboard and whether they were cut. Empty when it holds no files.
	pub fn read() -> Result<(Vec<String>, bool), String> {
		let _clipboard = OpenClipboardGuard::open()?;
		let Some(files) = get_bytes(CF_HDROP as u32) else {
			return Ok((Vec::new(), false));
		};
		let paths = decode_drop_files(&files)?;
		let cut = get_bytes(preferred_drop_effect())
			.and_then(|bytes| bytes.get(..4).map(super::read_u32))
			.is_some_and(is_cut);
		Ok((paths, cut))
	}
}

#[cfg(windows)]
pub use win32::{read, write};

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn paths_survive_a_round_trip() {
		let paths = vec![
			r"C:\Users\Ana\Área de Trabalho\a b#c.txt".to_string(),
			r"\\server\share\日本.pdf".to_string(),
			"D:/forward/slashes".to_string(),
		];
		assert_eq!(
			decode_drop_files(&encode_drop_files(&paths).unwrap()).unwrap(),
			paths
		);
		assert_eq!(
			decode_drop_files(&encode_drop_files(&[]).unwrap()).unwrap(),
			Vec::<String>::new()
		);
	}

	#[test]
	fn header_matches_the_dropfiles_layout() {
		let bytes = encode_drop_files(&[r"C:\a".to_string()]).unwrap();
		assert_eq!(&bytes[0..4], &20u32.to_le_bytes());
		assert_eq!(&bytes[16..20], &1u32.to_le_bytes());
		assert_eq!(
			&bytes[20..],
			&[b'C', 0, b':', 0, b'\\', 0, b'a', 0, 0, 0, 0, 0]
		);
	}

	#[test]
	fn refuses_paths_the_receiver_would_misread() {
		assert!(encode_drop_files(&["relative\\file".to_string()]).is_err());
		assert!(encode_drop_files(&["/tmp/unix".to_string()]).is_err());
		assert!(encode_drop_files(&["C:".to_string()]).is_err());
		assert!(encode_drop_files(&["C:\\a\0b".to_string()]).is_err());
	}

	#[test]
	fn decoding_ignores_padding_and_rejects_bad_blocks() {
		let mut bytes = encode_drop_files(&[r"C:\a".to_string()]).unwrap();
		bytes.extend_from_slice(&[0x41, 0, 0x42, 0]);
		assert_eq!(
			decode_drop_files(&bytes).unwrap(),
			vec![r"C:\a".to_string()]
		);

		assert!(decode_drop_files(&bytes[..10]).is_err());
		let mut ansi = bytes.clone();
		ansi[16..20].copy_from_slice(&0u32.to_le_bytes());
		assert!(decode_drop_files(&ansi).is_err());
		let mut inside_header = bytes;
		inside_header[0..4].copy_from_slice(&4u32.to_le_bytes());
		assert!(decode_drop_files(&inside_header).is_err());
	}

	#[test]
	fn drop_effect_matches_explorer() {
		assert!(is_cut(drop_effect(true)));
		assert!(!is_cut(drop_effect(false)));
		// Explorer writes 5 for a copy and 2 for a cut.
		assert!(!is_cut(5));
		assert!(is_cut(2));
		assert!(!is_cut(0));
	}
}
