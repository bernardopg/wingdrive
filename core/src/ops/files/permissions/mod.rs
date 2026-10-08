//! # Unix permissions
//!
//! Lets the Inspector show and change a file's mode bits and group, the
//! ownership change an unprivileged user may make. Changing the owning user
//! needs root, so it is out of scope. Recursive changes follow `chmod -R
//! u=rwX` semantics: folders get the full mode while files only keep execute
//! bits if they were already executable, so making a tree readable does not
//! turn every document into a program. Symbolic links are never followed.
//!
//! ## Example
//! ```rust,ignore
//! let input = SetPermissionsInput { path: WingPath::local("/home/me/run.sh"), mode: 0o755, recursive: false };
//! SetPermissionsAction::from_input(input)?.execute(library, context).await?;
//! ```

use std::path::Path;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use specta::Type;

use crate::context::CoreContext;
use crate::domain::addressing::WingPath;
use crate::infra::action::{error::ActionError, LibraryAction, ValidationResult};
use crate::infra::query::{LibraryQuery, QueryError, QueryResult};

/// Input for reading a path's permissions
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PermissionsQuery {
	pub path: WingPath,
}

/// Mode, owner and group of a path
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PermissionsOutput {
	/// Permission bits, including setuid, setgid and sticky (`0o7777` mask)
	pub mode: u32,
	pub owner: String,
	pub group: String,
	pub is_dir: bool,
	/// The daemon user owns the path, so it may change mode and group
	pub is_owner: bool,
	/// Groups the path can be moved to: those the daemon user belongs to
	pub available_groups: Vec<String>,
}

/// Input for changing permission bits
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SetPermissionsInput {
	pub path: WingPath,
	/// New permission bits (`0o7777` mask)
	pub mode: u32,
	/// Apply to everything inside a folder too
	#[serde(default)]
	pub recursive: bool,
}

/// Input for changing the group
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SetGroupInput {
	pub path: WingPath,
	pub group: String,
}

/// Result of a permission change
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PermissionsChanged {
	/// Entries changed, including the path itself
	pub changed: u32,
	/// Entries that could not be changed, with the reason
	pub failed: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetPermissionsAction {
	input: SetPermissionsInput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetGroupAction {
	input: SetGroupInput,
}

fn local_path(path: &WingPath) -> Option<&Path> {
	path.is_local().then(|| path.as_local_path()).flatten()
}

fn not_local() -> ActionError {
	ActionError::Validation {
		field: "path".to_string(),
		message: "Permissions can only be changed for files on this device".to_string(),
	}
}

#[cfg(unix)]
mod unix {
	use std::ffi::{CStr, CString};
	use std::os::unix::fs::{MetadataExt, PermissionsExt};
	use std::path::Path;

	use super::PermissionsChanged;

	fn buffer_size() -> usize {
		let size = unsafe { libc::sysconf(libc::_SC_GETPW_R_SIZE_MAX) };
		if size > 0 {
			size as usize
		} else {
			16 * 1024
		}
	}

	pub fn user_name(uid: u32) -> String {
		let mut buf = vec![0 as libc::c_char; buffer_size()];
		let mut pwd: libc::passwd = unsafe { std::mem::zeroed() };
		let mut result = std::ptr::null_mut();
		let rc =
			unsafe { libc::getpwuid_r(uid, &mut pwd, buf.as_mut_ptr(), buf.len(), &mut result) };
		if rc == 0 && !result.is_null() {
			unsafe { CStr::from_ptr(pwd.pw_name) }
				.to_string_lossy()
				.into_owned()
		} else {
			uid.to_string()
		}
	}

	pub fn group_name(gid: u32) -> String {
		let mut buf = vec![0 as libc::c_char; buffer_size().max(64 * 1024)];
		let mut grp: libc::group = unsafe { std::mem::zeroed() };
		let mut result = std::ptr::null_mut();
		let rc =
			unsafe { libc::getgrgid_r(gid, &mut grp, buf.as_mut_ptr(), buf.len(), &mut result) };
		if rc == 0 && !result.is_null() {
			unsafe { CStr::from_ptr(grp.gr_name) }
				.to_string_lossy()
				.into_owned()
		} else {
			gid.to_string()
		}
	}

	pub fn group_id(name: &str) -> Option<u32> {
		let name = CString::new(name).ok()?;
		let mut buf = vec![0 as libc::c_char; buffer_size().max(64 * 1024)];
		let mut grp: libc::group = unsafe { std::mem::zeroed() };
		let mut result = std::ptr::null_mut();
		let rc = unsafe {
			libc::getgrnam_r(
				name.as_ptr(),
				&mut grp,
				buf.as_mut_ptr(),
				buf.len(),
				&mut result,
			)
		};
		(rc == 0 && !result.is_null()).then_some(grp.gr_gid)
	}

	/// Primary and supplementary groups of the daemon process.
	pub fn own_groups() -> Vec<u32> {
		let count = unsafe { libc::getgroups(0, std::ptr::null_mut()) };
		let mut groups = vec![0 as libc::gid_t; count.max(0) as usize];
		let written = unsafe { libc::getgroups(groups.len() as libc::c_int, groups.as_mut_ptr()) };
		groups.truncate(written.max(0) as usize);
		let primary = unsafe { libc::getegid() };
		if !groups.contains(&primary) {
			groups.insert(0, primary);
		}
		groups
	}

	pub fn own_uid() -> u32 {
		unsafe { libc::geteuid() }
	}

	pub fn mode_and_ids(path: &Path) -> std::io::Result<(u32, u32, u32, bool)> {
		let meta = std::fs::symlink_metadata(path)?;
		Ok((meta.mode() & 0o7777, meta.uid(), meta.gid(), meta.is_dir()))
	}

	/// Mode for one entry of a recursive change, like `chmod -R` with `X`.
	pub fn recursive_mode(requested: u32, current: u32, is_dir: bool) -> u32 {
		if is_dir || current & 0o111 != 0 {
			requested
		} else {
			requested & !0o111
		}
	}

	pub fn set_mode(path: &Path, mode: u32, recursive: bool) -> PermissionsChanged {
		let mut report = PermissionsChanged {
			changed: 0,
			failed: Vec::new(),
		};
		let mut stack = vec![(path.to_path_buf(), true)];
		while let Some((entry, is_root)) = stack.pop() {
			let meta = match std::fs::symlink_metadata(&entry) {
				Ok(meta) => meta,
				Err(e) => {
					report.failed.push(format!("{}: {e}", entry.display()));
					continue;
				}
			};
			// chmod on a link would change its target; links have no mode of their own.
			if meta.file_type().is_symlink() {
				continue;
			}
			let new_mode = if is_root {
				mode
			} else {
				recursive_mode(mode, meta.mode() & 0o7777, meta.is_dir())
			};
			match std::fs::set_permissions(&entry, std::fs::Permissions::from_mode(new_mode)) {
				Ok(()) => report.changed += 1,
				Err(e) => report.failed.push(format!("{}: {e}", entry.display())),
			}
			if recursive && meta.is_dir() {
				match std::fs::read_dir(&entry) {
					Ok(children) => stack.extend(children.flatten().map(|c| (c.path(), false))),
					Err(e) => report.failed.push(format!("{}: {e}", entry.display())),
				}
			}
		}
		report
	}

	pub fn set_group(path: &Path, gid: u32) -> std::io::Result<()> {
		std::os::unix::fs::lchown(path, None, Some(gid))
	}
}

impl LibraryQuery for PermissionsQuery {
	type Input = PermissionsQuery;
	type Output = PermissionsOutput;

	fn from_input(input: Self::Input) -> QueryResult<Self> {
		Ok(input)
	}

	async fn execute(
		self,
		_context: Arc<CoreContext>,
		_session: crate::infra::api::SessionContext,
	) -> QueryResult<Self::Output> {
		let path = local_path(&self.path)
			.ok_or_else(|| {
				QueryError::InvalidInput(
					"Permissions are only available for files on this device".into(),
				)
			})?
			.to_path_buf();
		#[cfg(unix)]
		{
			tokio::task::spawn_blocking(move || {
				let (mode, uid, gid, is_dir) =
					unix::mode_and_ids(&path).map_err(|e| QueryError::Internal(e.to_string()))?;
				Ok(PermissionsOutput {
					mode,
					owner: unix::user_name(uid),
					group: unix::group_name(gid),
					is_dir,
					is_owner: uid == unix::own_uid(),
					available_groups: unix::own_groups()
						.into_iter()
						.map(unix::group_name)
						.collect(),
				})
			})
			.await
			.map_err(|e| QueryError::Internal(e.to_string()))?
		}
		#[cfg(not(unix))]
		{
			let _ = path;
			Err(QueryError::Internal(
				"Unix permissions are not available on this platform".into(),
			))
		}
	}
}

impl LibraryAction for SetPermissionsAction {
	type Input = SetPermissionsInput;
	type Output = PermissionsChanged;

	fn from_input(input: Self::Input) -> Result<Self, String> {
		Ok(Self { input })
	}

	async fn validate(
		&self,
		_library: &Arc<crate::library::Library>,
		_context: Arc<CoreContext>,
	) -> Result<ValidationResult, ActionError> {
		local_path(&self.input.path).ok_or_else(not_local)?;
		if self.input.mode > 0o7777 {
			return Err(ActionError::Validation {
				field: "mode".to_string(),
				message: format!("{:o} is not a valid mode", self.input.mode),
			});
		}
		Ok(ValidationResult::Success { metadata: None })
	}

	async fn execute(
		self,
		_library: Arc<crate::library::Library>,
		_context: Arc<CoreContext>,
	) -> Result<Self::Output, ActionError> {
		let path = local_path(&self.input.path)
			.ok_or_else(not_local)?
			.to_path_buf();
		#[cfg(unix)]
		{
			let (mode, recursive) = (self.input.mode, self.input.recursive);
			let report =
				tokio::task::spawn_blocking(move || unix::set_mode(&path, mode, recursive))
					.await
					.map_err(|e| ActionError::Internal(e.to_string()))?;
			if report.changed == 0 {
				return Err(ActionError::Internal(report.failed.join("; ")));
			}
			Ok(report)
		}
		#[cfg(not(unix))]
		{
			let _ = path;
			Err(ActionError::Internal(
				"Unix permissions are not available on this platform".into(),
			))
		}
	}

	fn action_kind(&self) -> &'static str {
		"files.setPermissions"
	}
}

impl LibraryAction for SetGroupAction {
	type Input = SetGroupInput;
	type Output = PermissionsChanged;

	fn from_input(input: Self::Input) -> Result<Self, String> {
		Ok(Self { input })
	}

	async fn validate(
		&self,
		_library: &Arc<crate::library::Library>,
		_context: Arc<CoreContext>,
	) -> Result<ValidationResult, ActionError> {
		local_path(&self.input.path).ok_or_else(not_local)?;
		Ok(ValidationResult::Success { metadata: None })
	}

	async fn execute(
		self,
		_library: Arc<crate::library::Library>,
		_context: Arc<CoreContext>,
	) -> Result<Self::Output, ActionError> {
		let path = local_path(&self.input.path)
			.ok_or_else(not_local)?
			.to_path_buf();
		#[cfg(unix)]
		{
			let gid = unix::group_id(&self.input.group).ok_or_else(|| ActionError::Validation {
				field: "group".to_string(),
				message: format!("Unknown group {}", self.input.group),
			})?;
			tokio::task::spawn_blocking(move || unix::set_group(&path, gid))
				.await
				.map_err(|e| ActionError::Internal(e.to_string()))?
				.map_err(|e| ActionError::Internal(format!("Failed to change group: {e}")))?;
			Ok(PermissionsChanged {
				changed: 1,
				failed: Vec::new(),
			})
		}
		#[cfg(not(unix))]
		{
			let _ = path;
			Err(ActionError::Internal(
				"Unix permissions are not available on this platform".into(),
			))
		}
	}

	fn action_kind(&self) -> &'static str {
		"files.setGroup"
	}
}

crate::register_library_query!(PermissionsQuery, "files.permissions");
crate::register_library_action!(SetPermissionsAction, "files.setPermissions");
crate::register_library_action!(SetGroupAction, "files.setGroup");

#[cfg(all(test, unix))]
mod tests {
	use std::os::unix::fs::PermissionsExt;

	use super::unix::*;

	fn mode_of(path: &std::path::Path) -> u32 {
		std::fs::symlink_metadata(path)
			.unwrap()
			.permissions()
			.mode() & 0o7777
	}

	#[test]
	fn single_change_sets_the_exact_mode() {
		let dir = tempfile::tempdir().unwrap();
		let script = dir.path().join("run.sh");
		std::fs::write(&script, "#!/bin/sh\n").unwrap();
		std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o644)).unwrap();

		let report = set_mode(&script, 0o755, false);
		assert_eq!((report.changed, report.failed.len()), (1, 0));
		assert_eq!(mode_of(&script), 0o755);
	}

	#[test]
	fn recursive_change_keeps_documents_non_executable_and_skips_links() {
		let dir = tempfile::tempdir().unwrap();
		let root = dir.path().join("tree");
		std::fs::create_dir_all(root.join("sub")).unwrap();
		let doc = root.join("sub/doc.txt");
		let tool = root.join("tool");
		std::fs::write(&doc, "").unwrap();
		std::fs::write(&tool, "").unwrap();
		std::fs::set_permissions(&doc, std::fs::Permissions::from_mode(0o600)).unwrap();
		std::fs::set_permissions(&tool, std::fs::Permissions::from_mode(0o700)).unwrap();
		let outside = dir.path().join("outside");
		std::fs::write(&outside, "").unwrap();
		std::fs::set_permissions(&outside, std::fs::Permissions::from_mode(0o600)).unwrap();
		std::os::unix::fs::symlink(&outside, root.join("link")).unwrap();

		let report = set_mode(&root, 0o755, true);
		assert!(report.failed.is_empty(), "{:?}", report.failed);
		assert_eq!(mode_of(&root), 0o755);
		assert_eq!(mode_of(&root.join("sub")), 0o755);
		assert_eq!(mode_of(&doc), 0o644);
		assert_eq!(mode_of(&tool), 0o755);
		assert_eq!(mode_of(&outside), 0o600, "link target must not change");
	}

	#[test]
	fn names_resolve_for_the_current_user() {
		let uid = own_uid();
		assert!(!user_name(uid).is_empty());
		let groups = own_groups();
		assert!(!groups.is_empty());
		let name = group_name(groups[0]);
		assert_eq!(group_id(&name), Some(groups[0]));
	}
}
