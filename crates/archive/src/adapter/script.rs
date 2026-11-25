//! Script adapter runtime: spawn external processes, parse JSONL protocol.

use std::collections::HashMap;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::time::Instant;

use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;

use crate::adapter::{sandbox, Adapter, AdapterKind, SyncReport};
use crate::db::SourceDb;
use crate::error::{Error, Result};
use crate::safety::TrustTier;
use crate::schema::{DataTypeMeta, DataTypeSchema, SearchContract};

/// A parsed `adapter.toml` manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterManifest {
	pub adapter: AdapterMeta,
	#[serde(default)]
	pub data_type: Option<DataTypeMeta>,
	#[serde(default)]
	pub models: HashMap<String, toml::Value>,
	#[serde(default)]
	pub search: Option<SearchContract>,
	#[serde(skip)]
	raw_toml: String,
}

/// The `[adapter]` section of `adapter.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterMeta {
	pub id: String,
	pub name: String,
	#[serde(default)]
	pub description: String,
	#[serde(default = "default_version")]
	pub version: String,
	#[serde(default)]
	pub author: String,
	#[serde(default)]
	pub license: String,
	#[serde(default)]
	pub homepage: String,
	#[serde(default)]
	pub icon: Option<String>,
	#[serde(default)]
	pub min_wingdrive: Option<String>,
	#[serde(default)]
	pub trust_tier: Option<String>,
	/// Up to two lines for the store card. Contract 1 requires it.
	#[serde(default)]
	pub summary: Option<String>,
	/// Version of the stdin/stdout protocol the adapter speaks. Prototypes written before the
	/// store omit it and run as contract 0.
	#[serde(default)]
	pub contract: u32,
	#[serde(default)]
	pub platforms: Vec<String>,
	#[serde(default)]
	pub categories: Vec<String>,
	#[serde(default)]
	pub permissions: Permissions,
	pub runtime: RuntimeConfig,
	#[serde(default)]
	pub config: Vec<ConfigField>,
	/// Store texts and config labels in other languages, keyed by language code (`pt`).
	#[serde(default)]
	pub i18n: std::collections::BTreeMap<String, AdapterTranslation>,
}

/// The newest adapter contract this build understands.
pub const SUPPORTED_CONTRACT: u32 = 1;

/// What an adapter may touch, declared in `[adapter.permissions]` and shown before install.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Permissions {
	/// Paths the adapter reads; `~` stands for the user's home.
	#[serde(default)]
	pub read: Vec<String>,
	/// Hosts the adapter contacts. Empty means no network.
	#[serde(default)]
	pub network: Vec<String>,
	/// Secret keys the adapter receives from the secret store.
	#[serde(default)]
	pub secrets: Vec<String>,
	/// External programs the adapter runs.
	#[serde(default)]
	pub tools: Vec<String>,
}

/// How a contract 1 adapter is started.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RuntimeKind {
	/// `python3 <entry>`.
	Python3,
	/// A compiled program at `<entry>`, built by the registry's CI.
	Binary,
}

fn default_version() -> String {
	"0.1.0".to_string()
}

/// The `[adapter.runtime]` section.
///
/// Contract 1 adapters name a `kind` and an `entry` and run without a shell. Prototypes still give a
/// `command` line that runs through `sh -c`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
	#[serde(default)]
	pub kind: Option<RuntimeKind>,
	#[serde(default)]
	pub entry: Option<String>,
	#[serde(default)]
	pub command: Option<String>,
	#[serde(default)]
	pub watch_command: Option<String>,
	#[serde(default = "default_timeout")]
	pub timeout: u64,
	#[serde(default)]
	pub schedule: Option<String>,
	#[serde(default)]
	pub requires: Vec<String>,
	#[serde(default)]
	pub setup: Option<String>,
	#[serde(default)]
	pub env: Vec<String>,
	/// The adapter answers `"mode": "detect"` with suggested config values (contract 1).
	#[serde(default)]
	pub detect: bool,
}

fn default_timeout() -> u64 {
	300
}

/// `[adapter.i18n.<lang>]`: the adapter's texts in one language.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AdapterTranslation {
	#[serde(default)]
	pub summary: Option<String>,
	#[serde(default)]
	pub description: Option<String>,
	/// Labels of config fields, keyed by field key.
	#[serde(default)]
	pub config: std::collections::BTreeMap<String, ConfigFieldTranslation>,
}

/// A config field's label and help in one language.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConfigFieldTranslation {
	#[serde(default)]
	pub name: Option<String>,
	#[serde(default)]
	pub description: Option<String>,
}

/// A `[[adapter.config]]` field declaration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigField {
	pub key: String,
	pub name: String,
	#[serde(default)]
	pub description: String,
	#[serde(rename = "type", default = "default_config_type")]
	pub field_type: String,
	#[serde(default)]
	pub required: bool,
	#[serde(default)]
	pub secret: bool,
	#[serde(default)]
	pub default: Option<toml::Value>,
	#[serde(default)]
	pub options: Vec<String>,
	#[serde(default)]
	pub path_type: Option<String>,
}

fn default_config_type() -> String {
	"string".to_string()
}

impl AdapterManifest {
	/// Parse an `adapter.toml` file.
	///
	/// Refuses a contract this build does not understand and a runtime that cannot be started, so
	/// an adapter installed from the store either runs or never registers.
	pub fn parse(toml_str: &str) -> Result<Self> {
		let mut manifest: Self = toml::from_str(toml_str)
			.map_err(|e| Error::SchemaParse(format!("adapter.toml: {e}")))?;
		manifest.raw_toml = toml_str.to_string();
		manifest.check_runtime()?;
		Ok(manifest)
	}

	fn check_runtime(&self) -> Result<()> {
		let meta = &self.adapter;
		if meta.contract > SUPPORTED_CONTRACT {
			return Err(Error::SchemaParse(format!(
				"adapter.toml: {} needs adapter contract {}, this WingDrive supports up to {}",
				meta.id, meta.contract, SUPPORTED_CONTRACT
			)));
		}
		let runtime = &meta.runtime;
		if meta.contract >= 1 {
			let entry = runtime.entry.as_deref().unwrap_or_default();
			if runtime.kind.is_none() || entry.is_empty() {
				return Err(Error::SchemaParse(format!(
					"adapter.toml: {} uses contract 1 and needs runtime.kind and runtime.entry",
					meta.id
				)));
			}
			let path = Path::new(entry);
			if path.is_absolute()
				|| path
					.components()
					.any(|c| matches!(c, std::path::Component::ParentDir))
			{
				return Err(Error::SchemaParse(format!(
					"adapter.toml: {} runtime.entry must stay inside the adapter folder",
					meta.id
				)));
			}
		} else if runtime.command.as_deref().unwrap_or_default().is_empty() {
			return Err(Error::SchemaParse(format!(
				"adapter.toml: {} needs runtime.command (or contract = 1 with kind and entry)",
				meta.id
			)));
		}
		Ok(())
	}

	/// Parse from a file path.
	pub fn from_file(path: &Path) -> Result<Self> {
		let content = std::fs::read_to_string(path)?;
		Self::parse(&content)
	}

	/// Extract the embedded data type schema.
	pub fn extract_schema(&self) -> Result<DataTypeSchema> {
		let raw_table: toml::Table = toml::from_str(&self.raw_toml)
			.map_err(|e| Error::SchemaParse(format!("adapter.toml re-parse: {e}")))?;

		let mut schema_table = toml::Table::new();

		if let Some(dt) = raw_table.get("data_type") {
			schema_table.insert("data_type".to_string(), dt.clone());
		} else {
			return Err(Error::SchemaParse(
				"adapter.toml missing [data_type] section".into(),
			));
		}

		if let Some(models) = raw_table.get("models") {
			schema_table.insert("models".to_string(), models.clone());
		} else {
			return Err(Error::SchemaParse(
				"adapter.toml missing [models] section".into(),
			));
		}

		if let Some(search) = raw_table.get("search") {
			schema_table.insert("search".to_string(), search.clone());
		} else {
			return Err(Error::SchemaParse(
				"adapter.toml missing [search] section".into(),
			));
		}

		let schema_toml = toml::to_string_pretty(&schema_table)
			.map_err(|e| Error::SchemaParse(format!("schema rebuild: {e}")))?;

		crate::schema::parser::parse(&schema_toml)
	}

	/// List config fields marked as secret.
	pub fn secret_fields(&self) -> Vec<&ConfigField> {
		self.adapter.config.iter().filter(|f| f.secret).collect()
	}
}

/// A single operation from the script's JSONL stdout.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SyncOperation {
	Upsert {
		upsert: String,
		external_id: String,
		fields: serde_json::Value,
	},
	Delete {
		delete: String,
		external_id: String,
	},
	Link {
		link: String,
		id: String,
		to: String,
		to_id: String,
	},
	Unlink {
		unlink: String,
		id: String,
		to: String,
		to_id: String,
	},
	Cursor {
		cursor: String,
	},
	Log {
		log: String,
		message: String,
	},
}

impl SyncOperation {
	/// Parse a single JSONL line into a SyncOperation.
	pub fn parse_line(line: &str) -> Result<Self> {
		let line = line.trim();
		if line.is_empty() {
			return Err(Error::AdapterSync("empty JSONL line".into()));
		}

		let value: serde_json::Value = serde_json::from_str(line)
			.map_err(|e| Error::AdapterSync(format!("invalid JSON: {e}")))?;

		let obj = value
			.as_object()
			.ok_or_else(|| Error::AdapterSync("JSONL line must be a JSON object".into()))?;

		if obj.contains_key("upsert") {
			Ok(SyncOperation::Upsert {
				upsert: obj["upsert"]
					.as_str()
					.ok_or_else(|| Error::AdapterSync("upsert field must be a string".into()))?
					.to_string(),
				external_id: obj
					.get("external_id")
					.and_then(|v| v.as_str())
					.ok_or_else(|| Error::AdapterSync("upsert missing external_id".into()))?
					.to_string(),
				fields: obj
					.get("fields")
					.cloned()
					.unwrap_or(serde_json::Value::Object(Default::default())),
			})
		} else if obj.contains_key("delete") {
			Ok(SyncOperation::Delete {
				delete: obj["delete"]
					.as_str()
					.ok_or_else(|| Error::AdapterSync("delete field must be a string".into()))?
					.to_string(),
				external_id: obj
					.get("external_id")
					.and_then(|v| v.as_str())
					.ok_or_else(|| Error::AdapterSync("delete missing external_id".into()))?
					.to_string(),
			})
		} else if obj.contains_key("link") {
			Ok(SyncOperation::Link {
				link: obj["link"]
					.as_str()
					.ok_or_else(|| Error::AdapterSync("link field must be a string".into()))?
					.to_string(),
				id: obj
					.get("id")
					.and_then(|v| v.as_str())
					.ok_or_else(|| Error::AdapterSync("link missing id".into()))?
					.to_string(),
				to: obj
					.get("to")
					.and_then(|v| v.as_str())
					.ok_or_else(|| Error::AdapterSync("link missing to".into()))?
					.to_string(),
				to_id: obj
					.get("to_id")
					.and_then(|v| v.as_str())
					.ok_or_else(|| Error::AdapterSync("link missing to_id".into()))?
					.to_string(),
			})
		} else if obj.contains_key("unlink") {
			Ok(SyncOperation::Unlink {
				unlink: obj["unlink"]
					.as_str()
					.ok_or_else(|| Error::AdapterSync("unlink field must be a string".into()))?
					.to_string(),
				id: obj
					.get("id")
					.and_then(|v| v.as_str())
					.ok_or_else(|| Error::AdapterSync("unlink missing id".into()))?
					.to_string(),
				to: obj
					.get("to")
					.and_then(|v| v.as_str())
					.ok_or_else(|| Error::AdapterSync("unlink missing to".into()))?
					.to_string(),
				to_id: obj
					.get("to_id")
					.and_then(|v| v.as_str())
					.ok_or_else(|| Error::AdapterSync("unlink missing to_id".into()))?
					.to_string(),
			})
		} else if obj.contains_key("cursor") {
			Ok(SyncOperation::Cursor {
				cursor: obj["cursor"]
					.as_str()
					.ok_or_else(|| Error::AdapterSync("cursor field must be a string".into()))?
					.to_string(),
			})
		} else if obj.contains_key("log") {
			Ok(SyncOperation::Log {
				log: obj["log"]
					.as_str()
					.ok_or_else(|| Error::AdapterSync("log field must be a string".into()))?
					.to_string(),
				message: obj
					.get("message")
					.and_then(|v| v.as_str())
					.unwrap_or("")
					.to_string(),
			})
		} else {
			Err(Error::AdapterSync(format!(
				"unknown JSONL operation: {}",
				serde_json::to_string(&value).unwrap_or_default()
			)))
		}
	}
}

/// The Python interpreter adapters run with: `python3` on Unix, `python` on Windows.
pub fn python_interpreter() -> &'static str {
	if cfg!(windows) {
		"python"
	} else {
		"python3"
	}
}

/// A script adapter loaded from a directory containing `adapter.toml`.
pub struct ScriptAdapter {
	dir: PathBuf,
	manifest: AdapterManifest,
	schema: DataTypeSchema,
	icon_svg: Option<String>,
}

impl ScriptAdapter {
	/// Load a script adapter from its directory.
	pub fn from_dir(dir: &Path) -> Result<Self> {
		let manifest_path = dir.join("adapter.toml");
		if !manifest_path.exists() {
			return Err(Error::AdapterNotFound(format!(
				"no adapter.toml in {}",
				dir.display()
			)));
		}

		let manifest = AdapterManifest::from_file(&manifest_path)?;
		let schema = manifest.extract_schema()?;

		let icon_path = dir.join("icon.svg");
		let icon_svg = if icon_path.exists() {
			std::fs::read_to_string(&icon_path).ok()
		} else {
			None
		};

		Ok(Self {
			dir: dir.to_path_buf(),
			manifest,
			schema,
			icon_svg,
		})
	}

	/// Get the manifest.
	pub fn manifest(&self) -> &AdapterManifest {
		&self.manifest
	}

	/// Get the extracted data type schema.
	pub fn schema(&self) -> &DataTypeSchema {
		&self.schema
	}

	/// Get the adapter kind.
	pub fn kind(&self) -> AdapterKind {
		AdapterKind::Script
	}

	/// The process that runs one sync or detection.
	///
	/// Contract 1 adapters start without a shell and without the daemon's environment, so tokens
	/// and paths the daemon holds never reach third-party code; they get a minimal `PATH`,
	/// `HOME`, locale and their own variables. On Linux they also run inside bubblewrap with only
	/// their declared paths (see [`super::sandbox`]). Prototypes keep the old `sh -c` behavior.
	fn command(&self, config: &serde_json::Value) -> Result<Command> {
		let runtime = &self.manifest.adapter.runtime;
		if self.manifest.adapter.contract < 1 {
			let mut command = Command::new("sh");
			command
				.arg("-c")
				.arg(runtime.command.as_deref().unwrap_or_default())
				.current_dir(&self.dir)
				.envs(self.build_env(config));
			return Ok(command);
		}

		let entry = runtime
			.entry
			.as_deref()
			.ok_or_else(|| Error::AdapterSync("adapter.runtime.entry is missing".into()))?;
		let dir = self.dir.canonicalize().unwrap_or_else(|_| self.dir.clone());
		let (program, args): (PathBuf, Vec<std::ffi::OsString>) = match runtime.kind {
			Some(RuntimeKind::Binary) => (dir.join(entry), Vec::new()),
			_ => (
				PathBuf::from(python_interpreter()),
				vec!["-I".into(), dir.join(entry).into_os_string()],
			),
		};

		let isolation = sandbox::isolation()?;
		let state_dir = self.state_dir(config)?;
		let mut command = if isolation == sandbox::Isolation::Bubblewrap {
			let home = std::env::var_os("HOME").map(PathBuf::from);
			let grant = sandbox::Grant {
				adapter_dir: dir.clone(),
				state_dir: state_dir.clone(),
				read: self
					.manifest
					.adapter
					.permissions
					.read
					.iter()
					.filter_map(|declared| sandbox::expand(declared, home.as_deref(), config))
					.collect(),
				network: !self.manifest.adapter.permissions.network.is_empty(),
				program,
			};
			let mut command = sandbox::wrap(&grant)?;
			command.args(&args);
			command
		} else {
			let mut command = Command::new(program);
			command.args(&args).current_dir(&dir);
			command
		};

		command.env_clear();
		for key in [
			"PATH",
			"HOME",
			"LANG",
			"LC_ALL",
			"TZ",
			"TMPDIR",
			"SYSTEMROOT",
			"USERPROFILE",
			"APPDATA",
			"LOCALAPPDATA",
		] {
			if let Ok(value) = std::env::var(key) {
				command.env(key, value);
			}
		}
		if isolation == sandbox::Isolation::Bubblewrap {
			// The sandbox has its own empty /tmp; the host's TMPDIR is not mounted.
			command.env("TMPDIR", "/tmp");
		}
		command.env("WINGDRIVE_ADAPTER_ISOLATION", isolation.as_str());
		command.envs(self.build_env(config));
		command.kill_on_drop(true);
		sandbox::limit_resources(&mut command, runtime.timeout.max(1));
		Ok(command)
	}

	/// The folder a source's adapter may write to: `<source folder>/state`, created on demand.
	/// The source's database lives next to it and stays out of the adapter's reach.
	fn state_dir(&self, config: &serde_json::Value) -> Result<Option<PathBuf>> {
		let Some(data_dir) = config.get("_data_dir").and_then(|v| v.as_str()) else {
			return Ok(None);
		};
		let state = Path::new(data_dir).join("state");
		std::fs::create_dir_all(&state)?;
		Ok(Some(state.canonicalize().unwrap_or(state)))
	}

	/// Build the sanitized environment for the subprocess.
	fn build_env(&self, config: &serde_json::Value) -> HashMap<String, String> {
		let mut env = HashMap::new();

		env.insert(
			"WINGDRIVE_ADAPTER_ID".to_string(),
			self.manifest.adapter.id.clone(),
		);
		env.insert(
			"WINGDRIVE_ADAPTER_VERSION".to_string(),
			self.manifest.adapter.version.clone(),
		);

		if let Some(obj) = config.as_object() {
			for (key, value) in obj {
				let value = value
					.as_str()
					.map(|s| s.to_string())
					.unwrap_or_else(|| value.to_string());
				env.insert(format!("WINGDRIVE_CONFIG_{}", key.to_uppercase()), value);
			}
		}

		// Contract 1 adapters get a cleared environment; passing host variables through would
		// let a manifest read any secret the daemon has. They receive secrets as config fields.
		let passthrough: &[String] = if self.manifest.adapter.contract >= 1 {
			&[]
		} else {
			&self.manifest.adapter.runtime.env
		};
		for var_name in passthrough {
			if let Ok(value) = std::env::var(var_name) {
				env.insert(var_name.clone(), value);
			}
		}

		env
	}
}

/// How long a detect run may take before its suggestions are dropped.
const DETECT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(15);

/// Suggested values per config key, from a detect run.
pub type Suggestions = HashMap<String, Vec<String>>;

impl ScriptAdapter {
	/// Ask the adapter for config values it can find on this machine, such as a vault folder.
	///
	/// Only adapters that set `runtime.detect` are asked. The adapter reads
	/// `{"contract": 1, "mode": "detect", "config": {}, "cursor": null, "data_dir": null}` and prints
	/// `{"suggest": {"<key>": ["value", ...]}}` lines. Failures and timeouts give no suggestions.
	pub async fn detect_suggestions(&self) -> Suggestions {
		if self.manifest.adapter.contract < 1 || !self.manifest.adapter.runtime.detect {
			return Suggestions::new();
		}
		match tokio::time::timeout(DETECT_TIMEOUT, self.run_detect()).await {
			Ok(Ok(suggestions)) => suggestions,
			Ok(Err(error)) => {
				tracing::debug!(adapter = %self.manifest.adapter.id, %error, "detect failed");
				Suggestions::new()
			}
			Err(_) => {
				tracing::debug!(adapter = %self.manifest.adapter.id, "detect timed out");
				Suggestions::new()
			}
		}
	}

	async fn run_detect(&self) -> Result<Suggestions> {
		let input = serde_json::json!({
			"contract": self.manifest.adapter.contract,
			"mode": "detect",
			"config": {},
			"cursor": null,
			"data_dir": null,
		});
		let mut child = self
			.command(&serde_json::json!({}))?
			.stdin(std::process::Stdio::piped())
			.stdout(std::process::Stdio::piped())
			.stderr(std::process::Stdio::null())
			.kill_on_drop(true)
			.spawn()
			.map_err(|e| Error::AdapterSync(format!("failed to spawn adapter: {e}")))?;
		if let Some(mut stdin) = child.stdin.take() {
			stdin.write_all(input.to_string().as_bytes()).await?;
			stdin.shutdown().await?;
		}
		let stdout = child
			.stdout
			.take()
			.ok_or_else(|| Error::AdapterSync("failed to open stdout".into()))?;
		let mut suggestions = Suggestions::new();
		let mut lines = BufReader::new(stdout).lines();
		while let Some(line) = lines.next_line().await? {
			let Ok(value) = serde_json::from_str::<serde_json::Value>(&line) else {
				continue;
			};
			let Some(suggest) = value.get("suggest").and_then(|s| s.as_object()) else {
				continue;
			};
			for (key, values) in suggest {
				let entry = suggestions.entry(key.clone()).or_default();
				for value in values.as_array().into_iter().flatten() {
					if let Some(text) = value.as_str() {
						if !entry.iter().any(|v| v == text) {
							entry.push(text.to_string());
						}
					}
				}
			}
		}
		child.wait().await?;
		Ok(suggestions)
	}
}

/// The JSON a contract 1 adapter reads on stdin.
///
/// `config` holds the fields the user filled in; keys the engine injects (`_data_dir`) move to
/// their own fields. `cursor` is the last value the adapter emitted, or null on the first sync.
fn contract_input(
	contract: u32,
	config: &serde_json::Value,
	cursor: Option<String>,
) -> serde_json::Value {
	let mut user = serde_json::Map::new();
	let mut data_dir = serde_json::Value::Null;
	if let Some(fields) = config.as_object() {
		for (key, value) in fields {
			if key == "_data_dir" {
				data_dir = value
					.as_str()
					.map(|dir| {
						serde_json::Value::String(
							Path::new(dir).join("state").to_string_lossy().into_owned(),
						)
					})
					.unwrap_or(serde_json::Value::Null);
			} else if !key.starts_with('_') {
				user.insert(key.clone(), value.clone());
			}
		}
	}
	serde_json::json!({
		"contract": contract,
		"config": user,
		"cursor": cursor,
		"data_dir": data_dir,
	})
}

impl Adapter for ScriptAdapter {
	fn id(&self) -> &str {
		&self.manifest.adapter.id
	}

	fn name(&self) -> &str {
		&self.manifest.adapter.name
	}

	fn data_type(&self) -> &str {
		&self.schema.data_type.id
	}

	fn description(&self) -> &str {
		&self.manifest.adapter.description
	}

	fn version(&self) -> &str {
		&self.manifest.adapter.version
	}

	fn author(&self) -> &str {
		&self.manifest.adapter.author
	}

	fn icon_svg(&self) -> Option<&str> {
		self.icon_svg.as_deref()
	}

	fn schema(&self) -> Option<&DataTypeSchema> {
		Some(&self.schema)
	}

	fn config_fields(&self) -> &[ConfigField] {
		&self.manifest.adapter.config
	}

	fn translations(&self) -> Option<&std::collections::BTreeMap<String, AdapterTranslation>> {
		Some(&self.manifest.adapter.i18n)
	}

	fn detect<'a>(&'a self) -> std::pin::Pin<Box<dyn Future<Output = Suggestions> + Send + 'a>> {
		Box::pin(self.detect_suggestions())
	}

	fn trust_tier(&self) -> TrustTier {
		self.manifest
			.adapter
			.trust_tier
			.as_deref()
			.map(TrustTier::from_str_or_default)
			.unwrap_or(TrustTier::External)
	}

	fn sync<'a>(
		&'a self,
		db: &'a SourceDb,
		config: &'a serde_json::Value,
	) -> std::pin::Pin<Box<dyn Future<Output = Result<SyncReport>> + Send + 'a>> {
		Box::pin(async move {
			let start = Instant::now();
			let mut report = SyncReport {
				records_upserted: 0,
				records_deleted: 0,
				links_created: 0,
				links_removed: 0,
				duration_ms: 0,
				error: None,
			};

			let mut child = self
				.command(config)?
				.stdin(std::process::Stdio::piped())
				.stdout(std::process::Stdio::piped())
				.stderr(std::process::Stdio::piped())
				.spawn()
				.map_err(|e| Error::AdapterSync(format!("failed to spawn adapter: {e}")))?;

			let mut stdin = child
				.stdin
				.take()
				.ok_or_else(|| Error::AdapterSync("failed to open stdin".into()))?;
			let stdout = child
				.stdout
				.take()
				.ok_or_else(|| Error::AdapterSync("failed to open stdout".into()))?;
			let stderr = child
				.stderr
				.take()
				.ok_or_else(|| Error::AdapterSync("failed to open stderr".into()))?;
			// Drain stderr so a chatty adapter never blocks on a full pipe, and keep the last
			// line to explain a failed run.
			let adapter_id = self.manifest.adapter.id.clone();
			let stderr_task = tokio::spawn(async move {
				let mut lines = BufReader::new(stderr).lines();
				let mut last = None;
				while let Ok(Some(line)) = lines.next_line().await {
					if line.trim().is_empty() {
						continue;
					}
					tracing::debug!(adapter = %adapter_id, "{line}");
					last = Some(line);
				}
				last
			});

			let input = if self.manifest.adapter.contract >= 1 {
				contract_input(
					self.manifest.adapter.contract,
					config,
					db.get_cursor("default").await?,
				)
			} else {
				config.clone()
			};
			let config_json = serde_json::to_string(&input)
				.map_err(|e| Error::AdapterSync(format!("failed to serialize config: {e}")))?;

			tokio::spawn(async move {
				let _ = stdin.write_all(config_json.as_bytes()).await;
				let _ = stdin.shutdown().await;
			});

			let limit =
				std::time::Duration::from_secs(self.manifest.adapter.runtime.timeout.max(1));
			let process = async {
				let mut reader = BufReader::new(stdout).lines();
				while let Some(line) = reader.next_line().await? {
					match SyncOperation::parse_line(&line) {
						Ok(op) => match op {
							SyncOperation::Upsert {
								upsert: model,
								external_id,
								fields,
							} => {
								if let Err(e) = db.upsert(&model, &external_id, &fields).await {
									tracing::warn!(error = %e, "upsert failed");
								} else {
									report.records_upserted += 1;
								}
							}
							SyncOperation::Delete {
								delete: model,
								external_id,
							} => {
								if let Err(e) = db.delete(&model, &external_id).await {
									tracing::warn!(error = %e, "delete failed");
								} else {
									report.records_deleted += 1;
								}
							}
							SyncOperation::Link {
								link: model_a,
								id: ext_id_a,
								to: model_b,
								to_id: ext_id_b,
							} => {
								if let Err(e) =
									db.link(&model_a, &ext_id_a, &model_b, &ext_id_b).await
								{
									tracing::warn!(error = %e, "link failed");
								} else {
									report.links_created += 1;
								}
							}
							SyncOperation::Unlink {
								unlink: model_a,
								id: ext_id_a,
								to: model_b,
								to_id: ext_id_b,
							} => {
								if let Err(e) =
									db.unlink(&model_a, &ext_id_a, &model_b, &ext_id_b).await
								{
									tracing::warn!(error = %e, "unlink failed");
								} else {
									report.links_removed += 1;
								}
							}
							SyncOperation::Cursor { cursor } => {
								if let Err(e) = db.set_cursor("default", &cursor).await {
									tracing::warn!(error = %e, "cursor update failed");
								}
							}
							SyncOperation::Log {
								log: level,
								message,
							} => match level.as_str() {
								"info" => tracing::info!("{message}"),
								"warn" => tracing::warn!("{message}"),
								"error" => tracing::error!("{message}"),
								_ => tracing::debug!("{message}"),
							},
						},
						Err(e) => {
							tracing::warn!(line = %line, error = %e, "failed to parse JSONL line");
						}
					}
				}
				Ok::<(), Error>(())
			};
			let timed_out = match tokio::time::timeout(limit, process).await {
				Ok(result) => {
					result?;
					false
				}
				Err(_) => {
					let _ = child.start_kill();
					true
				}
			};

			let status = child.wait().await?;
			report.duration_ms = start.elapsed().as_millis() as u64;

			let last_stderr = stderr_task.await.ok().flatten();
			if timed_out {
				report.error = Some(format!(
					"adapter stopped after {} s (runtime.timeout)",
					limit.as_secs()
				));
			} else if !status.success() {
				report.error = Some(match last_stderr {
					Some(line) => format!("adapter exited with status: {status}: {line}"),
					None => format!("adapter exited with status: {status}"),
				});
			}

			Ok(report)
		})
	}
}
