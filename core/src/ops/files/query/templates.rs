//! # Templates folder
//!
//! Nautilus and Dolphin offer the files in the user's templates folder under
//! "New Document". The folder comes from `XDG_TEMPLATES_DIR` in
//! `~/.config/user-dirs.dirs`, which localized systems point at names such as
//! `~/Modelos`, with `~/Templates` as the fallback. Setting it to the home
//! folder disables templates, as the XDG user-dirs spec describes, so that case
//! lists nothing. `files.createFromTemplate` copies the chosen entry.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{
	context::CoreContext,
	domain::addressing::WingPath,
	infra::query::{LibraryQuery, QueryResult},
};

/// One file the New menu can create a copy of.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct TemplateFile {
	/// File name with extension, as the copy will be named.
	pub name: String,
	/// Path of the template, for `files.createFromTemplate`.
	pub path: WingPath,
}

/// Input for listing templates; the folder is the user's, not a parameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
pub struct TemplatesInput {}

/// The user's templates, sorted by name.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct TemplatesOutput {
	/// Templates folder in use, or `None` when there is none.
	pub directory: Option<String>,
	pub templates: Vec<TemplateFile>,
}

/// Lists the files in the user's templates folder.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct TemplatesQuery {
	pub input: TemplatesInput,
}

impl LibraryQuery for TemplatesQuery {
	type Input = TemplatesInput;
	type Output = TemplatesOutput;

	fn from_input(input: Self::Input) -> QueryResult<Self> {
		Ok(Self { input })
	}

	async fn execute(
		self,
		_context: std::sync::Arc<CoreContext>,
		_session: crate::infra::api::SessionContext,
	) -> QueryResult<Self::Output> {
		let Some(directory) = templates_dir(dirs::template_dir(), dirs::home_dir()) else {
			return Ok(TemplatesOutput::default());
		};

		let templates = list_templates(&directory).await;
		Ok(TemplatesOutput {
			directory: Some(directory.to_string_lossy().into_owned()),
			templates,
		})
	}
}

/// The configured templates folder, or `~/Templates`, unless it is the home folder.
fn templates_dir(configured: Option<PathBuf>, home: Option<PathBuf>) -> Option<PathBuf> {
	let directory = configured.or_else(|| home.as_ref().map(|home| home.join("Templates")))?;
	if home.is_some_and(|home| home == directory) {
		return None;
	}
	Some(directory)
}

/// Visible regular files directly in `directory`, sorted by name ignoring case.
///
/// Links to files count, since users often link shared templates in. Folders
/// are skipped; Nautilus turns them into submenus, which the menu does not do yet.
async fn list_templates(directory: &Path) -> Vec<TemplateFile> {
	let Ok(mut entries) = tokio::fs::read_dir(directory).await else {
		return Vec::new();
	};

	let mut templates = Vec::new();
	while let Ok(Some(entry)) = entries.next_entry().await {
		let name = entry.file_name().to_string_lossy().into_owned();
		if name.starts_with('.') {
			continue;
		}
		let path = entry.path();
		if tokio::fs::metadata(&path).await.is_ok_and(|m| m.is_file()) {
			templates.push(TemplateFile {
				name,
				path: WingPath::local(path),
			});
		}
	}

	templates.sort_by_key(|template| template.name.to_lowercase());
	templates
}

crate::register_library_query!(TemplatesQuery, "files.templates");

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn uses_the_configured_folder_then_falls_back_to_templates() {
		let home = PathBuf::from("/home/me");

		assert_eq!(
			templates_dir(Some(home.join("Modelos")), Some(home.clone())),
			Some(home.join("Modelos"))
		);
		assert_eq!(
			templates_dir(None, Some(home.clone())),
			Some(home.join("Templates"))
		);
		assert_eq!(templates_dir(None, None), None);
	}

	#[test]
	fn a_folder_set_to_home_disables_templates() {
		let home = PathBuf::from("/home/me");
		assert_eq!(templates_dir(Some(home.clone()), Some(home)), None);
	}

	#[tokio::test]
	async fn lists_visible_files_sorted_by_name() {
		let dir = tempfile::tempdir().unwrap();
		std::fs::write(dir.path().join("script.sh"), "").unwrap();
		std::fs::write(dir.path().join("Letter.odt"), "").unwrap();
		std::fs::write(dir.path().join(".hidden"), "").unwrap();
		std::fs::create_dir(dir.path().join("Folder")).unwrap();

		let names: Vec<_> = list_templates(dir.path())
			.await
			.into_iter()
			.map(|template| template.name)
			.collect();
		assert_eq!(names, ["Letter.odt", "script.sh"]);
	}

	#[tokio::test]
	async fn a_missing_folder_lists_nothing() {
		let dir = tempfile::tempdir().unwrap();
		assert!(list_templates(&dir.path().join("missing")).await.is_empty());
	}
}
