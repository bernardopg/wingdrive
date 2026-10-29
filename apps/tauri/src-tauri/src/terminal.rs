//! # Docked terminal
//!
//! Dolphin's F4 opens a terminal under the file view that follows the folder
//! you browse. WingDrive runs your login shell on a pseudo-terminal here and
//! streams its output to the webview, which draws it with xterm.js. Output
//! leaves as raw bytes over a Tauri channel: a multibyte character can span two
//! reads, and xterm.js decodes UTF-8 across chunks itself.
//!
//! The terminal and the Explorer follow each other, as in Dolphin:
//!
//! - Navigating types a `cd` into the shell after clearing the prompt. It only
//!   types while the shell waits at its prompt: with `vim` or a build running,
//!   the keys would go to that program. The folder waits instead and is typed
//!   once the shell is back at its prompt.
//! - After each burst of output the shell's working folder is read again, and a
//!   change (you typed `cd`) is reported so the Explorer can go there.
//!
//! On Unix the command is ` cd '<path>'` after Ctrl+E and Ctrl+U. The leading
//! space keeps it out of the history in bash with `HISTCONTROL=ignorespace` and
//! in zsh with `HIST_IGNORE_SPACE`. On Windows the shell is `cmd`, so it is
//! `cd /d "<path>"` after Escape, which clears cmd's input line.
//!
//! ## Example
//! ```rust,ignore
//! let session = Session::spawn(command, &cwd, size(80, 24), sink, |folder| moved(folder), done)?;
//! session.write(b"ls\r".to_vec())?;
//! session.follow(Path::new("/tmp"));
//! ```

use std::{
	collections::HashMap,
	io::{ErrorKind, Read, Write},
	path::{Path, PathBuf},
	sync::{
		atomic::{AtomicBool, AtomicU32, Ordering},
		mpsc, Arc,
	},
	time::Duration,
};

use parking_lot::Mutex;
use portable_pty::{native_pty_system, ChildKiller, CommandBuilder, MasterPty, PtySize};
use tauri::ipc::{Channel, InvokeResponseBody};

/// Bytes read from the terminal per output message.
const READ_CHUNK: usize = 16 * 1024;

/// Quiet time after output before the shell's state is read again.
const SETTLE: Duration = Duration::from_millis(60);

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

/// State the session shares with the thread that watches the shell.
struct Shell {
	master: Mutex<Box<dyn MasterPty + Send>>,
	/// Keystrokes for the writer thread, so a shell that stops reading never
	/// blocks the caller.
	input: mpsc::Sender<Vec<u8>>,
	pid: Option<u32>,
	/// Folder to `cd` into once the shell is back at its prompt.
	pending: Mutex<Option<PathBuf>>,
	/// The shell's working folder when it was last read.
	cwd: Mutex<Option<PathBuf>>,
}

impl Shell {
	fn write(&self, data: Vec<u8>) -> Result<(), String> {
		self.input.send(data).map_err(|_| CLOSED.to_string())
	}

	/// Whether the shell waits at its prompt, rather than running a program.
	#[cfg(unix)]
	fn at_prompt(&self) -> bool {
		match (self.master.lock().process_group_leader(), self.pid) {
			(Some(leader), Some(pid)) => i64::from(leader) == i64::from(pid),
			_ => false,
		}
	}

	/// Windows consoles have no foreground process group; a shell with no
	/// child process is at its prompt.
	#[cfg(not(unix))]
	fn at_prompt(&self) -> bool {
		self.pid.is_some_and(|pid| !has_child_process(pid))
	}

	/// Types the pending `cd` if the shell is at its prompt, then returns the
	/// shell's folder when it changed since the last call.
	fn settle(&self) -> Option<PathBuf> {
		if !self.at_prompt() {
			return None;
		}
		if let Some(path) = self.pending.lock().take() {
			if let Some(command) = cd_command(&path, Syntax::native()) {
				let _ = self.write(command);
			}
			// The shell moves after it reads the command; the next output reports it.
			return None;
		}
		let cwd = self.pid.and_then(process_cwd)?;
		let mut last = self.cwd.lock();
		if last.as_ref() == Some(&cwd) {
			return None;
		}
		*last = Some(cwd.clone());
		Some(cwd)
	}
}

/// A shell running on a pseudo-terminal. Dropping it hangs up the shell.
struct Session {
	shell: Arc<Shell>,
	killer: Box<dyn ChildKiller + Send + Sync>,
	/// Set once the shell is reaped; its pid may belong to another process then.
	exited: Arc<AtomicBool>,
}

impl Session {
	/// Starts `command` in `cwd` on a new pseudo-terminal.
	///
	/// `on_output` receives every chunk the terminal prints and returns false
	/// when nobody listens anymore, which hangs up the shell. `on_cwd` receives
	/// the shell's folder each time it changes. `on_exit` runs once with the
	/// shell's exit code after the terminal closes.
	fn spawn(
		mut command: CommandBuilder,
		cwd: &Path,
		size: PtySize,
		mut on_output: impl FnMut(&[u8]) -> bool + Send + 'static,
		on_cwd: impl Fn(&Path) + Send + 'static,
		on_exit: impl FnOnce(Option<u32>) + Send + 'static,
	) -> Result<Self, String> {
		// The shell reports its folder with symbolic links resolved; compare like with like.
		let cwd = cwd.canonicalize().unwrap_or_else(|_| cwd.to_path_buf());
		command.cwd(&cwd);
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
		let exited = Arc::new(AtomicBool::new(false));

		let (input, keystrokes) = mpsc::channel::<Vec<u8>>();
		let shell = Arc::new(Shell {
			master: Mutex::new(pair.master),
			input,
			pid: child.process_id(),
			pending: Mutex::new(None),
			cwd: Mutex::new(Some(cwd)),
		});

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

		// Reads the shell's state once its output goes quiet, so a long listing
		// costs one check rather than one per chunk.
		let (output_seen, quiet) = mpsc::channel::<()>();
		let watched = shell.clone();
		std::thread::Builder::new()
			.name("terminal-watch".into())
			.spawn(move || {
				while quiet.recv().is_ok() {
					loop {
						match quiet.recv_timeout(SETTLE) {
							Ok(()) => continue,
							Err(mpsc::RecvTimeoutError::Timeout) => break,
							Err(mpsc::RecvTimeoutError::Disconnected) => return,
						}
					}
					if let Some(cwd) = watched.settle() {
						on_cwd(&cwd);
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
							let _ = output_seen.send(());
						}
						Err(error) if error.kind() == ErrorKind::Interrupted => continue,
						// Linux reports EIO, not end of file, once the shell is gone.
						Err(_) => break,
					}
				}
				drop(output_seen);
				let code = child.wait().ok().map(|status| status.exit_code());
				reaped.store(true, Ordering::SeqCst);
				on_exit(code);
			})
			.map_err(|e| e.to_string())?;

		Ok(Self {
			shell,
			killer,
			exited,
		})
	}

	fn write(&self, data: Vec<u8>) -> Result<(), String> {
		self.shell.write(data)
	}

	fn resize(&self, size: PtySize) -> Result<(), String> {
		self.shell
			.master
			.lock()
			.resize(size)
			.map_err(|e| e.to_string())
	}

	/// Changes the shell to `path`: now when it waits at its prompt, or once it
	/// gets back there. Returns false for a path the shell cannot be sent to.
	fn follow(&self, path: &Path) -> bool {
		if !path.is_absolute() || !path.is_dir() {
			return false;
		}
		let Some(command) = cd_command(path, Syntax::native()) else {
			return false;
		};
		// The shell moves there either way, so its next report is not news.
		*self.shell.cwd.lock() = Some(path.canonicalize().unwrap_or_else(|_| path.to_path_buf()));
		if self.shell.at_prompt() {
			*self.shell.pending.lock() = None;
			self.write(command).is_ok()
		} else {
			*self.shell.pending.lock() = Some(path.to_path_buf());
			true
		}
	}
}

impl Drop for Session {
	fn drop(&mut self) {
		if !self.exited.load(Ordering::SeqCst) {
			let _ = self.killer.kill();
		}
	}
}

/// How the shell expects a `cd` to be typed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Syntax {
	/// sh, bash, zsh and fish.
	Posix,
	/// Windows `cmd`, the shell `CommandBuilder::new_default_prog` starts there.
	Cmd,
}

impl Syntax {
	fn native() -> Self {
		if cfg!(windows) {
			Self::Cmd
		} else {
			Self::Posix
		}
	}
}

/// Keystrokes that clear the prompt and change to `path`, or `None` for a path
/// the shell could misread: one that is not UTF-8 or holds control characters,
/// and for `cmd` one with `"` or `%`, which quoting cannot protect.
fn cd_command(path: &Path, syntax: Syntax) -> Option<Vec<u8>> {
	let text = path.to_str()?;
	if text.chars().any(char::is_control) {
		return None;
	}
	let keys = match syntax {
		Syntax::Posix => format!("\x05\x15 cd '{}'\r", text.replace('\'', r"'\''")),
		Syntax::Cmd => {
			if text.contains(['"', '%']) {
				return None;
			}
			format!("\x1b cd /d \"{text}\"\r")
		}
	};
	Some(keys.into_bytes())
}

/// The working folder of process `pid`.
#[cfg(target_os = "linux")]
fn process_cwd(pid: u32) -> Option<PathBuf> {
	std::fs::read_link(format!("/proc/{pid}/cwd")).ok()
}

#[cfg(not(target_os = "linux"))]
fn process_cwd(pid: u32) -> Option<PathBuf> {
	use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

	let pid = Pid::from_u32(pid);
	let mut system = System::new();
	system.refresh_processes_specifics(
		ProcessesToUpdate::Some(&[pid]),
		false,
		ProcessRefreshKind::nothing().with_cwd(UpdateKind::Always),
	);
	system.process(pid)?.cwd().map(Path::to_path_buf)
}

/// Whether any running process has `pid` as its parent.
#[cfg(any(not(unix), test))]
fn has_child_process(pid: u32) -> bool {
	use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};

	let parent = Pid::from_u32(pid);
	let mut system = System::new();
	system.refresh_processes_specifics(ProcessesToUpdate::All, true, ProcessRefreshKind::nothing());
	system
		.processes()
		.values()
		.any(|process| process.parent() == Some(parent))
}

/// A terminal `terminal_open` started.
#[derive(Debug, serde::Serialize)]
pub struct OpenedTerminal {
	id: u32,
	/// This machine's name, which shells put in the OSC 7 folder reports they print.
	hostname: Option<String>,
}

/// Terminal size in character cells, as xterm.js reports it.
#[derive(Debug, Clone, Copy, serde::Deserialize)]
pub struct TerminalSize {
	cols: u16,
	rows: u16,
}

impl From<TerminalSize> for PtySize {
	fn from(TerminalSize { cols, rows }: TerminalSize) -> Self {
		size(cols, rows)
	}
}

fn size(cols: u16, rows: u16) -> PtySize {
	PtySize {
		rows: rows.max(1),
		cols: cols.max(1),
		pixel_width: 0,
		pixel_height: 0,
	}
}

/// The user's login shell, with the host's environment.
fn shell_command() -> CommandBuilder {
	let mut command = CommandBuilder::new_default_prog();
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

/// Starts a shell in `cwd`. Output arrives on `output` as raw bytes, the
/// shell's new folder on `folder` each time you change it, and the exit code on
/// `exit` once the shell ends.
#[tauri::command]
pub fn terminal_open(
	window: tauri::Window,
	sessions: tauri::State<'_, TerminalSessions>,
	cwd: String,
	size: TerminalSize,
	output: Channel<InvokeResponseBody>,
	folder: Channel<String>,
	exit: Channel<Option<u32>>,
) -> Result<OpenedTerminal, String> {
	let cwd = Path::new(&cwd);
	if !cwd.is_dir() {
		return Err(format!("Not a folder: {}", cwd.display()));
	}
	let session = Session::spawn(
		shell_command(),
		cwd,
		size.into(),
		move |bytes| output.send(InvokeResponseBody::Raw(bytes.to_vec())).is_ok(),
		move |cwd| {
			if let Some(path) = cwd.to_str() {
				let _ = folder.send(path.to_string());
			}
		},
		move |code| {
			let _ = exit.send(code);
		},
	)?;
	let id = sessions.next_id.fetch_add(1, Ordering::Relaxed);
	sessions
		.sessions
		.lock()
		.insert(id, (window.label().to_string(), session));
	Ok(OpenedTerminal {
		id,
		hostname: sysinfo::System::host_name(),
	})
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
	size: TerminalSize,
) -> Result<(), String> {
	let sessions = sessions.sessions.lock();
	let (_, session) = sessions.get(&id).ok_or(CLOSED)?;
	session.resize(size.into())
}

/// Changes the shell to `path`, now or once it is back at its prompt; false for a
/// path it cannot be sent to.
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
	fn cd_command_quotes_the_path_for_posix_shells() {
		assert_eq!(
			cd_command(Path::new("/home/me/My Files"), Syntax::Posix).unwrap(),
			b"\x05\x15 cd '/home/me/My Files'\r"
		);
		assert_eq!(
			cd_command(Path::new("/tmp/it's"), Syntax::Posix).unwrap(),
			b"\x05\x15 cd '/tmp/it'\\''s'\r"
		);
	}

	#[test]
	fn cd_command_quotes_the_path_for_cmd() {
		assert_eq!(
			cd_command(Path::new(r"D:\My Files & more"), Syntax::Cmd).unwrap(),
			b"\x1b cd /d \"D:\\My Files & more\"\r"
		);
		assert!(cd_command(Path::new(r"C:\100%PATH%"), Syntax::Cmd).is_none());
		assert!(cd_command(Path::new("C:\\a\"b"), Syntax::Cmd).is_none());
	}

	#[test]
	fn cd_command_refuses_control_characters() {
		for syntax in [Syntax::Posix, Syntax::Cmd] {
			assert!(cd_command(Path::new("/tmp/a\nrm -rf ~"), syntax).is_none());
			assert!(cd_command(Path::new("/tmp/a\x1b[2J"), syntax).is_none());
		}
	}

	#[test]
	fn size_never_reaches_zero() {
		let size = size(0, 0);
		assert_eq!((size.cols, size.rows), (1, 1));
	}

	#[cfg(unix)]
	#[test]
	fn child_processes_are_seen() {
		let mut parent = std::process::Command::new("sh")
			.args(["-c", "sleep 30 & wait"])
			.spawn()
			.unwrap();
		assert!(wait_until(|| has_child_process(parent.id())));
		let _ = parent.kill();
		let _ = parent.wait();
		assert!(!has_child_process(std::process::id() + 1_000_000));
	}

	/// An interactive bash without the user's configuration, so the prompt and
	/// line editing are the same on every machine.
	#[cfg(unix)]
	fn bash() -> CommandBuilder {
		let mut command = CommandBuilder::new("bash");
		command.args(["--norc", "--noprofile", "-i"]);
		command.env("PS1", "$ ");
		command.env("TERM", "dumb");
		command
	}

	#[cfg(unix)]
	struct Bash {
		session: Session,
		printed: Receiver<Vec<u8>>,
		moved: Receiver<PathBuf>,
		exited: Receiver<Option<u32>>,
	}

	#[cfg(unix)]
	fn spawn_bash(cwd: &Path) -> Bash {
		let (output, printed) = mpsc::channel();
		let (folder, moved) = mpsc::channel();
		let (exit, exited) = mpsc::channel();
		let session = Session::spawn(
			bash(),
			cwd,
			size(200, 24),
			move |bytes| output.send(bytes.to_vec()).is_ok(),
			move |cwd| {
				let _ = folder.send(cwd.to_path_buf());
			},
			move |code| {
				let _ = exit.send(code);
			},
		)
		.unwrap();
		Bash {
			session,
			printed,
			moved,
			exited,
		}
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
	fn canonical(path: &Path) -> PathBuf {
		path.canonicalize().unwrap()
	}

	#[cfg(unix)]
	#[test]
	fn shell_starts_in_the_folder_and_follows_navigation() {
		let start = tempfile::tempdir().unwrap();
		let next = tempfile::tempdir().unwrap();
		let target = next.path().join("it's here");
		std::fs::create_dir(&target).unwrap();
		let bash = spawn_bash(start.path());
		let mut seen = String::new();

		bash.session.write(b"echo \"at:$PWD\"\r".to_vec()).unwrap();
		wait_for(
			&bash.printed,
			&mut seen,
			&format!("at:{}", canonical(start.path()).display()),
		);

		assert!(wait_until(|| bash.session.shell.at_prompt()));
		assert!(bash.session.follow(&target));
		bash.session.write(b"echo \"now:$PWD\"\r".to_vec()).unwrap();
		wait_for(
			&bash.printed,
			&mut seen,
			&format!("now:{}", canonical(&target).display()),
		);
		// Following is not a move the Explorer needs to hear about.
		assert!(bash.moved.recv_timeout(Duration::from_millis(400)).is_err());
	}

	#[cfg(unix)]
	#[test]
	fn busy_shell_follows_once_back_at_its_prompt() {
		let start = tempfile::tempdir().unwrap();
		let target = tempfile::tempdir().unwrap();
		let bash = spawn_bash(start.path());
		let mut seen = String::new();
		assert!(wait_until(|| bash.session.shell.at_prompt()));

		bash.session.write(b"sleep 1\r".to_vec()).unwrap();
		assert!(wait_until(|| !bash.session.shell.at_prompt()));
		assert!(bash.session.follow(target.path()));
		assert!(bash.session.shell.pending.lock().is_some());

		assert!(wait_until(|| bash.session.shell.pending.lock().is_none()));
		bash.session.write(b"echo \"now:$PWD\"\r".to_vec()).unwrap();
		wait_for(
			&bash.printed,
			&mut seen,
			&format!("now:{}", canonical(target.path()).display()),
		);
	}

	#[cfg(unix)]
	#[test]
	fn cd_typed_in_the_shell_is_reported() {
		let start = tempfile::tempdir().unwrap();
		let target = tempfile::tempdir().unwrap();
		let bash = spawn_bash(start.path());
		assert!(wait_until(|| bash.session.shell.at_prompt()));

		let command = format!("cd '{}'\r", target.path().display());
		bash.session.write(command.into_bytes()).unwrap();
		assert_eq!(
			bash.moved.recv_timeout(Duration::from_secs(10)).unwrap(),
			canonical(target.path())
		);
	}

	#[cfg(unix)]
	#[test]
	fn exit_reports_the_code_and_closing_hangs_up() {
		let start = tempfile::tempdir().unwrap();
		let bash = spawn_bash(start.path());
		bash.session.write(b"exit 3\r".to_vec()).unwrap();
		assert_eq!(
			bash.exited.recv_timeout(Duration::from_secs(10)).unwrap(),
			Some(3)
		);
		drop(bash.session);

		let bash = spawn_bash(start.path());
		drop(bash.session);
		assert!(bash.exited.recv_timeout(Duration::from_secs(10)).is_ok());
	}

	#[cfg(unix)]
	#[test]
	fn destroyed_window_closes_only_its_terminals() {
		let start = tempfile::tempdir().unwrap();
		let sessions = TerminalSessions::default();
		let closing = spawn_bash(start.path());
		let staying = spawn_bash(start.path());
		sessions
			.sessions
			.lock()
			.insert(0, ("window-1".into(), closing.session));
		sessions
			.sessions
			.lock()
			.insert(1, ("main".into(), staying.session));

		assert_eq!(sessions.close_window("window-1"), 1);
		assert!(closing.exited.recv_timeout(Duration::from_secs(10)).is_ok());
		assert!(staying
			.exited
			.recv_timeout(Duration::from_millis(200))
			.is_err());
		assert_eq!(sessions.sessions.lock().len(), 1);
	}
}
