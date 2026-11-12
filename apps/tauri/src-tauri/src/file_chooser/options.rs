//! Request options and results of `org.freedesktop.impl.portal.FileChooser`.
//!
//! The portal frontend checks the type of every documented option before it
//! calls the backend, so a value of the wrong type here means a broken caller
//! or frontend. It fails the request with response 2 instead of opening a
//! picker that ignores part of what the app asked for.

use std::collections::HashMap;
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use zbus::{OwnedVariant as OwnedValue, Variant as Value};

/// The portal method that opened the picker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PickerMode {
	Open,
	Save,
	SaveFiles,
}

/// One rule of a filter: a shell glob or a MIME type, as the spec numbers them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "pattern", rename_all = "snake_case")]
pub enum FilterRule {
	Glob(String),
	MimeType(String),
}

/// A named filter (`(sa(us))`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileFilter {
	pub name: String,
	pub rules: Vec<FilterRule>,
	/// Globs for the MIME rules, from the shared MIME database, so the picker
	/// matches names without its own MIME table.
	#[serde(default)]
	pub globs: Vec<String>,
}

impl FileFilter {
	fn from_wire((name, rules): (String, Vec<(u32, String)>)) -> Result<Self, OptionError> {
		let rules = rules
			.into_iter()
			.map(|(kind, pattern)| match kind {
				0 => Ok(FilterRule::Glob(pattern)),
				1 => Ok(FilterRule::MimeType(pattern)),
				other => Err(OptionError::FilterKind(other)),
			})
			.collect::<Result<_, _>>()?;
		Ok(Self {
			name,
			rules,
			globs: Vec::new(),
		})
	}

	fn to_wire(&self) -> (String, Vec<(u32, String)>) {
		let rules = self
			.rules
			.iter()
			.map(|rule| match rule {
				FilterRule::Glob(pattern) => (0, pattern.clone()),
				FilterRule::MimeType(pattern) => (1, pattern.clone()),
			})
			.collect();
		(self.name.clone(), rules)
	}
}

/// A combo box, or a check box when `options` is empty (`(ssa(ss)s)`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Choice {
	pub id: String,
	pub label: String,
	pub options: Vec<ChoiceOption>,
	pub initial: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChoiceOption {
	pub id: String,
	pub label: String,
}

/// The options of one request, with the spec's defaults applied.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PickerOptions {
	pub accept_label: Option<String>,
	pub modal: bool,
	pub multiple: bool,
	pub directory: bool,
	pub filters: Vec<FileFilter>,
	/// Index into `filters`; a current filter missing from the list is appended.
	pub current_filter: Option<usize>,
	pub choices: Vec<Choice>,
	pub current_folder: Option<PathBuf>,
	pub current_name: Option<String>,
	pub current_file: Option<PathBuf>,
	/// `SaveFiles`: names to create in the chosen folder.
	pub files: Vec<String>,
}

impl Default for PickerOptions {
	fn default() -> Self {
		Self {
			accept_label: None,
			modal: true,
			multiple: false,
			directory: false,
			filters: Vec::new(),
			current_filter: None,
			choices: Vec::new(),
			current_folder: None,
			current_name: None,
			current_file: None,
			files: Vec::new(),
		}
	}
}

impl PickerOptions {
	/// The folder the picker opens in.
	///
	/// A save of an existing file starts next to it, then the suggested
	/// folder, then `fallback` (the home folder).
	pub fn start_folder(&self, fallback: &Path) -> PathBuf {
		let parent = self
			.current_file
			.as_deref()
			.and_then(Path::parent)
			.filter(|parent| parent.is_dir());
		parent
			.or(self
				.current_folder
				.as_deref()
				.filter(|folder| folder.is_dir()))
			.unwrap_or(fallback)
			.to_path_buf()
	}

	/// The name a save suggests: `current_file`'s name wins over `current_name`.
	pub fn suggested_name(&self) -> Option<String> {
		self.current_file
			.as_deref()
			.and_then(Path::file_name)
			.map(|name| name.to_string_lossy().into_owned())
			.or_else(|| self.current_name.clone())
	}
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum OptionError {
	#[error("option {0} has the wrong type")]
	WrongType(&'static str),
	#[error("unknown filter rule type {0}")]
	FilterKind(u32),
	#[error("option {0} is not a valid path")]
	BadPath(&'static str),
	#[error("file name {0:?} is not a plain name")]
	BadFileName(String),
}

type Options = HashMap<String, OwnedValue>;

/// Reads `key` as `T`; missing keys are `None`, keys of another type an error.
fn get<T>(options: &Options, key: &'static str) -> Result<Option<T>, OptionError>
where
	T: TryFrom<Value<'static>>,
{
	let Some(value) = options.get(key) else {
		return Ok(None);
	};
	let value = value.try_clone().map_err(|_| OptionError::WrongType(key))?;
	T::try_from(Value::from(value))
		.map(Some)
		.map_err(|_| OptionError::WrongType(key))
}

/// Decodes a byte string path. The spec sends it NUL-terminated.
fn path_bytes(bytes: Vec<u8>, key: &'static str) -> Result<Option<PathBuf>, OptionError> {
	let bytes = bytes.strip_suffix(&[0]).unwrap_or(&bytes);
	if bytes.is_empty() {
		return Ok(None);
	}
	if bytes.contains(&0) {
		return Err(OptionError::BadPath(key));
	}
	let path = PathBuf::from(OsStr::from_bytes(bytes));
	match path.is_absolute() {
		true => Ok(Some(path)),
		false => Err(OptionError::BadPath(key)),
	}
}

/// A `SaveFiles` name must stay inside the chosen folder.
fn file_name(bytes: Vec<u8>) -> Result<String, OptionError> {
	let bytes = bytes.strip_suffix(&[0]).unwrap_or(&bytes);
	let name = String::from_utf8_lossy(bytes).into_owned();
	if name.is_empty() || name == "." || name == ".." || name.contains('/') || name.contains('\0') {
		return Err(OptionError::BadFileName(name));
	}
	Ok(name)
}

/// Parses the options vardict of `mode`. Options a mode does not define are
/// ignored, as unknown keys are.
pub fn parse(mode: PickerMode, options: &Options) -> Result<PickerOptions, OptionError> {
	let mut parsed = PickerOptions {
		accept_label: get(options, "accept_label")?,
		modal: get(options, "modal")?.unwrap_or(true),
		choices: get::<Vec<(String, String, Vec<(String, String)>, String)>>(options, "choices")?
			.unwrap_or_default()
			.into_iter()
			.map(|(id, label, options, initial)| Choice {
				id,
				label,
				options: options
					.into_iter()
					.map(|(id, label)| ChoiceOption { id, label })
					.collect(),
				initial,
			})
			.collect(),
		current_folder: match get::<Vec<u8>>(options, "current_folder")? {
			Some(bytes) => path_bytes(bytes, "current_folder")?,
			None => None,
		},
		..PickerOptions::default()
	};
	if mode != PickerMode::SaveFiles {
		parsed.filters = get::<Vec<(String, Vec<(u32, String)>)>>(options, "filters")?
			.unwrap_or_default()
			.into_iter()
			.map(FileFilter::from_wire)
			.collect::<Result<_, _>>()?;
		if let Some(current) = get::<(String, Vec<(u32, String)>)>(options, "current_filter")? {
			let current = FileFilter::from_wire(current)?;
			let index = match parsed.filters.iter().position(|filter| *filter == current) {
				Some(index) => index,
				None => {
					parsed.filters.push(current);
					parsed.filters.len() - 1
				}
			};
			parsed.current_filter = Some(index);
		}
	}
	match mode {
		PickerMode::Open => {
			parsed.multiple = get(options, "multiple")?.unwrap_or(false);
			parsed.directory = get(options, "directory")?.unwrap_or(false);
		}
		PickerMode::Save => {
			parsed.current_name = get(options, "current_name")?;
			parsed.current_file = match get::<Vec<u8>>(options, "current_file")? {
				Some(bytes) => path_bytes(bytes, "current_file")?,
				None => None,
			};
		}
		PickerMode::SaveFiles => {
			parsed.directory = true;
			parsed.files = get::<Vec<Vec<u8>>>(options, "files")?
				.unwrap_or_default()
				.into_iter()
				.map(file_name)
				.collect::<Result<_, _>>()?;
		}
	}
	Ok(parsed)
}

/// Turns a local absolute path into a `file://` URI, percent-encoding every
/// byte outside RFC 3986's unreserved set and `/`, so non-UTF-8 names survive.
pub fn file_uri(path: &Path) -> String {
	let mut uri = String::from("file://");
	for &byte in path.as_os_str().as_bytes() {
		match byte {
			b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' | b'/' => {
				uri.push(byte as char)
			}
			_ => uri.push_str(&format!("%{byte:02X}")),
		}
	}
	uri
}

/// What the user chose, as the picker sends it back.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Selection {
	/// `OpenFile`: the chosen files or folders.
	Open {
		paths: Vec<PathBuf>,
		#[serde(default)]
		choices: Vec<(String, String)>,
		#[serde(default)]
		current_filter: Option<usize>,
	},
	/// `SaveFile`: the target path; `overwrite` once the user confirmed it.
	Save {
		path: PathBuf,
		#[serde(default)]
		overwrite: bool,
		#[serde(default)]
		choices: Vec<(String, String)>,
		#[serde(default)]
		current_filter: Option<usize>,
	},
	/// `SaveFiles`: the folder that receives `files`.
	SaveFiles {
		folder: PathBuf,
		#[serde(default)]
		choices: Vec<(String, String)>,
	},
}

/// Why a selection cannot be returned yet.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SelectionError {
	/// The picker asks before replacing an existing file.
	#[error("{path} already exists")]
	Exists { path: String },
	#[error("{reason}")]
	Invalid { reason: String },
}

fn invalid(reason: impl Into<String>) -> SelectionError {
	SelectionError::Invalid {
		reason: reason.into(),
	}
}

/// A name not yet taken in `folder`: `name`, then `name (1).ext`, `name (2).ext`...
pub fn unique_name(folder: &Path, name: &str, taken: &[String]) -> String {
	let free = |candidate: &str| {
		!folder.join(candidate).exists() && !taken.iter().any(|used| used == candidate)
	};
	if free(name) {
		return name.to_string();
	}
	// A leading dot is a hidden file, not an extension.
	let (stem, extension) = match name.rfind('.') {
		Some(dot) if dot > 0 => name.split_at(dot),
		_ => (name, ""),
	};
	(1..)
		.map(|n| format!("{stem} ({n}){extension}"))
		.find(|candidate| free(candidate))
		.expect("an unbounded range always finds a free name")
}

/// Checks a selection against the request and builds the `results` vardict.
pub fn results(
	mode: PickerMode,
	options: &PickerOptions,
	selection: Selection,
) -> Result<HashMap<String, OwnedValue>, SelectionError> {
	let local = |path: &Path| {
		if path.is_absolute() {
			Ok(())
		} else {
			Err(invalid(format!("{} is not a local path", path.display())))
		}
	};
	let (uris, choices, current_filter) = match (mode, selection) {
		(
			PickerMode::Open,
			Selection::Open {
				paths,
				choices,
				current_filter,
			},
		) => {
			if paths.is_empty() {
				return Err(invalid("nothing selected"));
			}
			if paths.len() > 1 && !options.multiple {
				return Err(invalid("the app asked for one item"));
			}
			for path in &paths {
				local(path)?;
				let is_dir = path.is_dir();
				if options.directory && !is_dir {
					return Err(invalid(format!("{} is not a folder", path.display())));
				}
				if !options.directory && (is_dir || !path.exists()) {
					return Err(invalid(format!("{} is not a file", path.display())));
				}
			}
			let uris = paths.iter().map(|path| file_uri(path)).collect();
			(uris, choices, current_filter)
		}
		(
			PickerMode::Save,
			Selection::Save {
				path,
				overwrite,
				choices,
				current_filter,
			},
		) => {
			local(&path)?;
			if path.file_name().is_none() {
				return Err(invalid("the file needs a name"));
			}
			if path.is_dir() {
				return Err(invalid(format!("{} is a folder", path.display())));
			}
			if !path.parent().is_some_and(Path::is_dir) {
				return Err(invalid(format!(
					"{} has no existing parent folder",
					path.display()
				)));
			}
			if path.exists() && !overwrite {
				return Err(SelectionError::Exists {
					path: path.display().to_string(),
				});
			}
			(vec![file_uri(&path)], choices, current_filter)
		}
		(PickerMode::SaveFiles, Selection::SaveFiles { folder, choices }) => {
			local(&folder)?;
			if !folder.is_dir() {
				return Err(invalid(format!("{} is not a folder", folder.display())));
			}
			let mut names: Vec<String> = Vec::with_capacity(options.files.len());
			for name in &options.files {
				let unique = unique_name(&folder, name, &names);
				names.push(unique);
			}
			let uris = names
				.iter()
				.map(|name| file_uri(&folder.join(name)))
				.collect();
			(uris, choices, None)
		}
		_ => return Err(invalid("the selection does not match the request")),
	};

	let mut results = HashMap::new();
	results.insert("uris".to_string(), owned(Value::from(uris)));
	let choices: Vec<(String, String)> = choices
		.into_iter()
		.filter(|(id, _)| options.choices.iter().any(|choice| &choice.id == id))
		.collect();
	if !choices.is_empty() {
		results.insert("choices".to_string(), owned(Value::from(choices)));
	}
	if let Some(filter) = current_filter.and_then(|index| options.filters.get(index)) {
		results.insert(
			"current_filter".to_string(),
			owned(Value::from(filter.to_wire())),
		);
	}
	if mode == PickerMode::Open {
		results.insert("writable".to_string(), owned(Value::from(false)));
	}
	Ok(results)
}

fn owned(value: Value<'_>) -> OwnedValue {
	OwnedValue::try_from(value).expect("plain values hold no file descriptors")
}

/// Globs for a MIME type from shared-mime-info's `globs2` (`weight:type:glob`
/// lines). `image/*` matches every image type.
pub fn mime_globs(globs2: &str, mime: &str) -> Vec<String> {
	let mime = mime.trim().to_ascii_lowercase();
	let matches = |candidate: &str| match mime.strip_suffix("/*") {
		Some(group) => candidate
			.split_once('/')
			.is_some_and(|(media, _)| media == group),
		None => candidate == mime,
	};
	let mut globs: Vec<String> = Vec::new();
	for line in globs2.lines() {
		if line.starts_with('#') {
			continue;
		}
		let mut fields = line.splitn(4, ':');
		let (Some(_weight), Some(candidate), Some(glob)) =
			(fields.next(), fields.next(), fields.next())
		else {
			continue;
		};
		if matches(&candidate.to_ascii_lowercase()) && !globs.iter().any(|known| known == glob) {
			globs.push(glob.to_string());
		}
	}
	globs
}

/// Fills each filter's `globs` from the system MIME database.
pub fn resolve_mime_globs(filters: &mut [FileFilter], globs2: &str) {
	for filter in filters {
		filter.globs = filter
			.rules
			.iter()
			.flat_map(|rule| match rule {
				FilterRule::Glob(glob) => vec![glob.clone()],
				FilterRule::MimeType(mime) => mime_globs(globs2, mime),
			})
			.collect();
	}
}
