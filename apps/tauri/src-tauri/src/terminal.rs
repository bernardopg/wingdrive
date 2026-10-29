//! # Docked terminal
//!
//! Dolphin's F4 opens a terminal under the file view that follows the folder
//! you browse. WingDrive runs your login shell on a pseudo-terminal here and
//! streams its output to the webview, which draws it with xterm.js. Output
//! leaves as raw bytes over a Tauri channel: a multibyte character can span two
//! reads, and xterm.js decodes UTF-8 across chunks itself.
//!
//! Following a folder types ` cd '<path>'` into the shell after Ctrl+E and
//! Ctrl+U clear the prompt, as Dolphin does. It only types while the shell owns
//! the terminal's foreground: with `vim` or a build running, the keys would go
//! to that program, so the terminal stays where it is. The leading space keeps
//! the command out of the history in bash with `HISTCONTROL=ignorespace` and in
//! zsh with `HIST_IGNORE_SPACE`.
//!
//! ## Example
//! ```rust,ignore
//! let session = Session::spawn(command, size(80, 24), |bytes| sink(bytes), |code| done(code))?;
//! session.write(b"ls\r".to_vec())?;
//! session.follow(Path::new("/tmp"));
//! ```

use std::{
	collections::HashMap,
	io::{ErrorKind, Read, Write},
	path::Path,
	sync::{
		atomic::{AtomicBool, AtomicU32, Ordering},
		mpsc, Arc,
	},
};

use parking_lot::Mutex;
use portable_pty::{native_pty_system, ChildKiller, CommandBuilder, MasterPty, PtySize};
use tauri::ipc::{Channel, InvokeResponseBody};

/// Bytes read from the terminal per output message.
const READ_CHUNK: usize = 16 * 1024;

const CLOSED: &str = "The terminal is closed";

/// Open terminals, keyed by the id `terminal_open` returned, with the label of
/// the window that shows each one.
#[derive(Default)]
pub struct TerminalSessions {
	next_id: AtomicU32,
	sessions: Mutex<HashMap<u32, (String, Session)>>,
}

impl TerminalSessions {
	/// Hangs up the shells of a destroyed window, which can no longer close them.
	pub fn close_window(&self, label: &str) -> usize {
		let mut sessions = self.sessions.lock();
		let before = sessions.len();
		sessions.retain(|_, (window, _)| window != label);
		before - sessions.len()
	}
}

/// A shell running on a pseudo-terminal. Dropping it hangs up the shell.
struct Session {
	master: Box<dyn MasterPty + Send>,
	/// Keystrokes for the writer thread, so a shell that stops reading never
	/// blocks the caller.
	input: mpsc::Sender<Vec<u8>>,
	killer: Box<dyn ChildKiller + Send + Sync>,
	shell_pid: Option<u32>,
	/// Set once the shell is reaped; its pid may belong to another process then.
	exited: Arc<AtomicBool>,
}

impl Session {
	/// Starts `command` on a new pseudo-terminal.
	///
	/// `on_output` receives every chunk the terminal prints and returns false
	/// when nobody listens anymore, which hangs up the shell. `on_exit` runs
	/// once with the shell's exit code after the terminal closes.
	fn spawn(
		command: CommandBuilder,
		size: PtySize,
		mut on_output: impl FnMut(&[u8]) -> bool + Send + 'static,
		on_exit: impl FnOnce(Option<u32>) + Send + 'static,
	) -> Result<Self, String> {
		let pair = native_pty_system()
			.openpty(size)
			.map_err(|e| format!("Failed to open a terminal: {e}"))?;
		let mut child = pair
			.slave
			.spawn_command(command)
			.map_err(|e| format!("Failed to start the shell: {e}"))?;
		// The shell holds its own copy; ours would keep the terminal open after it exits.
		drop(pair.slave);

		let mut reader = pair.master.try_clone_reader().map_err(|e| e.to_string())?;
		let mut writer = pair.master.take_writer().map_err(|e| e.to_string())?;
		let killer = child.clone_killer();
		let mut output_killer = child.clone_killer();
		let shell_pid = child.process_id();
		let exited = Arc::new(AtomicBool::new(false));

		let (input, keystrokes) = mpsc::channel::<Vec<u8>>();
		std::thread::Builder::new()
			.name("terminal-input".into())
			.spawn(move || {
				for data in keystrokes {
					if writer
						.write_all(&data)
						.and_then(|()| writer.flush())
						.is_err()
					{
						break;
					}
				}
			})
			.map_err(|e| e.to_string())?;

		let reaped = exited.clone();
		std::thread::Builder::new()
			.name("terminal-output".into())
			.spawn(move || {
				let mut buffer = vec![0; READ_CHUNK];
				loop {
					match reader.read(&mut buffer) {
						Ok(0) => break,
						Ok(read) => {
							if !on_output(&buffer[..read]) {
								let _ = output_killer.kill();
								break;
							}
						}
						Err(error) if error.kind() == ErrorKind::Interrupted => continue,
						// Linux reports EIO, not end of file, once the shell is gone.
						Err(_) => break,
					}
				}
				let code = child.wait().ok().map(|status| status.exit_code());
				reaped.store(true, Ordering::SeqCst);
				on_exit(code);
			})
			.map_err(|e| e.to_string())?;

		Ok(Self {
			master: pair.master,
			input,
			killer,
			shell_pid,
			exited,
		})
	}

	fn write(&self, data: Vec<u8>) -> Result<(), String> {
		self.input.send(data).map_err(|_| CLOSED.to_string())
	}

	fn resize(&self, size: PtySize) -> Result<(), String> {
		self.master.resize(size).map_err(|e| e.to_string())
	}

	/// Changes the shell to `path` when it waits at its prompt. Returns whether
	/// it typed the command.
	fn follow(&self, path: &Path) -> bool {
		if !path.is_absolute() || !path.is_dir() || !self.shell_in_foreground() {
			return false;
		}
		match cd_command(path) {
			Some(command) => self.write(command).is_ok(),
			None => false,
		}
	}

	/// Whether the shell, not a program it started, owns the terminal.
	#[cfg(unix)]
	fn shell_in_foreground(&self) -> bool {
		match (self.master.process_group_leader(), self.shell_pid) {
			(Some(leader), Some(pid)) => i64::from(leader) == i64::from(pid),
			_ => false,
		}
	}

	/// Windows consoles have no foreground process group to check, and cmd and
	/// PowerShell quote paths differently, so the terminal does not follow there.
	#[cfg(not(unix))]
	fn shell_in_foreground(&self) -> bool {
		false
	}
}

impl Drop for Session {
	fn drop(&mut self) {
		if !self.exited.load(Ordering::SeqCst) {
			let _ = self.killer.kill();
		}
	}
}

/// Keystrokes that clear the prompt and change to `path`, or `None` for a path
/// the shell could misread: one that is not UTF-8 or holds control characters.
fn cd_command(path: &Path) -> Option<Vec<u8>> {
	let text = path.to_str()?;
	if text.chars().any(char::is_control) {
		return None;
	}
	let quoted = text.replace('\'', r"'\''");
	Some(format!("\x05\x15 cd '{quoted}'\r").into_bytes())
}

fn size(cols: u16, rows: u16) -> PtySize {
	PtySize {
		rows: rows.max(1),
		cols: cols.max(1),
		pixel_width: 0,
		pixel_height: 0,
	}
}

/// The user's login shell in `cwd`, with the host's environment.
fn shell_command(cwd: &Path) -> CommandBuilder {
	let mut command = CommandBuilder::new_default_prog();
	command.cwd(cwd);
	#[cfg(target_os = "linux")]
	for (key, value) in file_opening_linux::host_environment_overrides() {
		match value {
			Some(value) => command.env(key, value),
			None => command.env_remove(key),
		}
	}
	command.env("TERM", "xterm-256color");
	command.env("COLORTERM", "truecolor");
	command
}

/// Starts a shell in `cwd`. Output arrives on `output` as raw bytes and the
/// exit code on `exit` once the shell ends.
#[tauri::command]
pub fn terminal_open(
	window: tauri::Window,
	sessions: tauri::State<'_, TerminalSessions>,
	cwd: String,
	cols: u16,
	rows: u16,
	output: Channel<InvokeResponseBody>,
	exit: Channel<Option<u32>>,
) -> Result<u32, String> {
	let cwd = Path::new(&cwd);
	if !cwd.is_dir() {
		return Err(format!("Not a folder: {}", cwd.display()));
	}
	let session = Session::spawn(
		shell_command(cwd),
		size(cols, rows),
		move |bytes| output.send(InvokeResponseBody::Raw(bytes.to_vec())).is_ok(),
		move |code| {
			let _ = exit.send(code);
		},
	)?;
	let id = sessions.next_id.fetch_add(1, Ordering::Relaxed);
	sessions
		.sessions
		.lock()
		.insert(id, (window.label().to_string(), session));
	Ok(id)
}

/// Sends keystrokes or pasted text to the shell.
#[tauri::command]
pub fn terminal_write(
	sessions: tauri::State<'_, TerminalSessions>,
	id: u32,
	data: String,
) -> Result<(), String> {
	let sessions = sessions.sessions.lock();
	let (_, session) = sessions.get(&id).ok_or(CLOSED)?;
	session.write(data.into_bytes())
}

#[tauri::command]
pub fn terminal_resize(
	sessions: tauri::State<'_, TerminalSessions>,
	id: u32,
	cols: u16,
	rows: u16,
) -> Result<(), String> {
	let sessions = sessions.sessions.lock();
	let (_, session) = sessions.get(&id).ok_or(CLOSED)?;
	session.resize(size(cols, rows))
}

/// Changes the shell to `path` if it waits at its prompt; false when it is busy.
#[tauri::command]
pub fn terminal_follow(
	sessions: tauri::State<'_, TerminalSessions>,
	id: u32,
	path: String,
) -> Result<bool, String> {
	let sessions = sessions.sessions.lock();
	let (_, session) = sessions.get(&id).ok_or(CLOSED)?;
	Ok(session.follow(Path::new(&path)))
}

/// Hangs up the shell and forgets the terminal.
#[tauri::command]
pub fn terminal_close(sessions: tauri::State<'_, TerminalSessions>, id: u32) {
	sessions.sessions.lock().remove(&id);
}

#[cfg(test)]
mod tests {
	use std::{
		sync::mpsc::Receiver,
		time::{Duration, Instant},
	};

	use super::*;

	#[test]
	fn cd_command_quotes_the_path() {
		assert_eq!(
			cd_command(Path::new("/home/me/My Files")).unwrap(),
			b"\x05\x15 cd '/home/me/My Files'\r"
		);
		assert_eq!(
			cd_command(Path::new("/tmp/it's")).unwrap(),
			b"\x05\x15 cd '/tmp/it'\\''s'\r"
		);
	}

	#[test]
	fn cd_command_refuses_control_characters() {
		assert!(cd_command(Path::new("/tmp/a\nrm -rf ~")).is_none());
		assert!(cd_command(Path::new("/tmp/a\x1b[2J")).is_none());
	}

	#[test]
	fn size_never_reaches_zero() {
		let size = size(0, 0);
		assert_eq!((size.cols, size.rows), (1, 1));
	}

	/// An interactive bash without the user's configuration, so the prompt and
	/// line editing are the same on every machine.
	#[cfg(unix)]
	fn bash(cwd: &Path) -> CommandBuilder {
		let mut command = CommandBuilder::new("bash");
		command.args(["--norc", "--noprofile", "-i"]);
		command.cwd(cwd);
		command.env("PS1", "$ ");
		command.env("TERM", "dumb");
		command
	}

	#[cfg(unix)]
	fn spawn_bash(cwd: &Path) -> (Session, Receiver<Vec<u8>>, Receiver<Option<u32>>) {
		let (output, printed) = mpsc::channel();
		let (exit, exited) = mpsc::channel();
		let session = Session::spawn(
			bash(cwd),
			size(200, 24),
			move |bytes| output.send(bytes.to_vec()).is_ok(),
			move |code| {
				let _ = exit.send(code);
			},
		)
		.unwrap();
		(session, printed, exited)
	}

	/// Waits until the terminal has printed `needle`, keeping what it read.
	#[cfg(unix)]
	fn wait_for(printed: &Receiver<Vec<u8>>, seen: &mut String, needle: &str) {
		let deadline = Instant::now() + Duration::from_secs(10);
		while !seen.contains(needle) {
			let left = deadline.saturating_duration_since(Instant::now());
			match printed.recv_timeout(left) {
				Ok(bytes) => seen.push_str(&String::from_utf8_lossy(&bytes)),
				Err(_) => panic!("terminal never printed {needle:?}; it printed {seen:?}"),
			}
		}
	}

	#[cfg(unix)]
	fn wait_until(condition: impl Fn() -> bool) -> bool {
		let deadline = Instant::now() + Duration::from_secs(10);
		while Instant::now() < deadline {
			if condition() {
				return true;
			}
			std::thread::sleep(Duration::from_millis(20));
		}
		false
	}

	#[cfg(unix)]
	#[test]
	fn shell_starts_in_the_folder_and_follows_navigation() {
		let start = tempfile::tempdir().unwrap();
		let next = tempfile::tempdir().unwrap();
		let target = next.path().join("it's here");
		std::fs::create_dir(&target).unwrap();
		let (session, printed, _exited) = spawn_bash(start.path());
		let mut seen = String::new();

		session.write(b"echo \"at:$PWD\"\r".to_vec()).unwrap();
		wait_for(
			&printed,
			&mut seen,
			&format!("at:{}", start.path().display()),
		);

		assert!(wait_until(|| session.shell_in_foreground()));
		assert!(session.follow(&target));
		session.write(b"echo \"now:$PWD\"\r".to_vec()).unwrap();
		wait_for(&printed, &mut seen, &format!("now:{}", target.display()));
	}

	#[cfg(unix)]
	#[test]
	fn busy_shell_does_not_follow() {
		let start = tempfile::tempdir().unwrap();
		let (session, _printed, _exited) = spawn_bash(start.path());
		assert!(wait_until(|| session.shell_in_foreground()));

		session.write(b"sleep 30\r".to_vec()).unwrap();
		assert!(wait_until(|| !session.shell_in_foreground()));
		assert!(!session.follow(Path::new("/")));
	}

	#[cfg(unix)]
	#[test]
	fn exit_reports_the_code_and_closing_hangs_up() {
		let start = tempfile::tempdir().unwrap();
		let (session, _printed, exited) = spawn_bash(start.path());
		session.write(b"exit 3\r".to_vec()).unwrap();
		assert_eq!(
			exited.recv_timeout(Duration::from_secs(10)).unwrap(),
			Some(3)
		);
		drop(session);

		let (session, _printed, exited) = spawn_bash(start.path());
		drop(session);
		assert!(exited.recv_timeout(Duration::from_secs(10)).is_ok());
	}

	#[cfg(unix)]
	#[test]
	fn destroyed_window_closes_only_its_terminals() {
		let start = tempfile::tempdir().unwrap();
		let sessions = TerminalSessions::default();
		let (closing, _printed, closing_exited) = spawn_bash(start.path());
		let (staying, _printed, staying_exited) = spawn_bash(start.path());
		sessions
			.sessions
			.lock()
			.insert(0, ("window-1".into(), closing));
		sessions.sessions.lock().insert(1, ("main".into(), staying));

		assert_eq!(sessions.close_window("window-1"), 1);
		assert!(closing_exited.recv_timeout(Duration::from_secs(10)).is_ok());
		assert!(staying_exited
			.recv_timeout(Duration::from_millis(200))
			.is_err());
		assert_eq!(sessions.sessions.lock().len(), 1);
	}
}
