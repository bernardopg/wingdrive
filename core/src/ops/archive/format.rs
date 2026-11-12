//! Archive formats, detected from file names.

use std::path::Path;

use serde::{Deserialize, Serialize};
use specta::Type;

/// Archive formats WingDrive can read; all but 7-Zip can also be written.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum ArchiveFormat {
	Zip,
	Tar,
	TarGz,
	#[serde(rename = "tar_bz2")]
	TarBz2,
	TarXz,
	TarZst,
	SevenZip,
}

/// Suffixes longest first, so `.tar.gz` wins over `.gz`.
const SUFFIXES: &[(&str, ArchiveFormat)] = &[
	(".tar.gz", ArchiveFormat::TarGz),
	(".tar.bz2", ArchiveFormat::TarBz2),
	(".tar.xz", ArchiveFormat::TarXz),
	(".tar.zst", ArchiveFormat::TarZst),
	(".tgz", ArchiveFormat::TarGz),
	(".tbz2", ArchiveFormat::TarBz2),
	(".tbz", ArchiveFormat::TarBz2),
	(".txz", ArchiveFormat::TarXz),
	(".tzst", ArchiveFormat::TarZst),
	(".tar", ArchiveFormat::Tar),
	(".zip", ArchiveFormat::Zip),
	(".jar", ArchiveFormat::Zip),
	(".7z", ArchiveFormat::SevenZip),
];

impl ArchiveFormat {
	/// Format of an archive file, from its name.
	pub fn detect(path: &Path) -> Option<Self> {
		let name = path.file_name()?.to_str()?.to_lowercase();
		SUFFIXES
			.iter()
			.find(|(suffix, _)| name.ends_with(suffix) && name.len() > suffix.len())
			.map(|(_, format)| *format)
	}

	/// File name without the archive suffix: `photos.tar.gz` gives `photos`.
	pub fn stem(path: &Path) -> Option<String> {
		let name = path.file_name()?.to_str()?;
		let lower = name.to_lowercase();
		let (suffix, _) = SUFFIXES
			.iter()
			.find(|(suffix, _)| lower.ends_with(suffix) && lower.len() > suffix.len())?;
		Some(name[..name.len() - suffix.len()].to_string())
	}

	/// Extension used when creating an archive of this format.
	pub fn extension(self) -> &'static str {
		match self {
			Self::Zip => "zip",
			Self::Tar => "tar",
			Self::TarGz => "tar.gz",
			Self::TarBz2 => "tar.bz2",
			Self::TarXz => "tar.xz",
			Self::TarZst => "tar.zst",
			Self::SevenZip => "7z",
		}
	}

	pub fn can_write(self) -> bool {
		self != Self::SevenZip
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn detects_compound_suffixes_and_stems() {
		let cases = [
			("a.tar.gz", Some(ArchiveFormat::TarGz), Some("a")),
			("Photos.TGZ", Some(ArchiveFormat::TarGz), Some("Photos")),
			("b.tar", Some(ArchiveFormat::Tar), Some("b")),
			("c.zip", Some(ArchiveFormat::Zip), Some("c")),
			("d.7z", Some(ArchiveFormat::SevenZip), Some("d")),
			("e.gz", None, None),
			(".zip", None, None),
		];
		for (name, format, stem) in cases {
			assert_eq!(ArchiveFormat::detect(Path::new(name)), format, "{name}");
			assert_eq!(
				ArchiveFormat::stem(Path::new(name)).as_deref(),
				stem,
				"{name}"
			);
		}
	}

	macro_rules! format_case {
		($name:ident, $path:expr, $format:expr, $stem:expr) => {
			#[test]
			fn $name() {
				let path = Path::new($path);
				assert_eq!(ArchiveFormat::detect(path), $format);
				assert_eq!(ArchiveFormat::stem(path).as_deref(), $stem);
			}
		};
	}
	format_case!(
		detect_zip,
		"backup.zip",
		Some(ArchiveFormat::Zip),
		Some("backup")
	);
	format_case!(detect_jar, "app.jar", Some(ArchiveFormat::Zip), Some("app"));
	format_case!(
		detect_tar,
		"backup.tar",
		Some(ArchiveFormat::Tar),
		Some("backup")
	);
	format_case!(
		detect_tar_gz,
		"backup.tar.gz",
		Some(ArchiveFormat::TarGz),
		Some("backup")
	);
	format_case!(
		detect_tgz,
		"backup.tgz",
		Some(ArchiveFormat::TarGz),
		Some("backup")
	);
	format_case!(
		detect_tar_bz2,
		"backup.tar.bz2",
		Some(ArchiveFormat::TarBz2),
		Some("backup")
	);
	format_case!(
		detect_tbz2,
		"backup.tbz2",
		Some(ArchiveFormat::TarBz2),
		Some("backup")
	);
	format_case!(
		detect_tbz,
		"backup.tbz",
		Some(ArchiveFormat::TarBz2),
		Some("backup")
	);
	format_case!(
		detect_tar_xz,
		"backup.tar.xz",
		Some(ArchiveFormat::TarXz),
		Some("backup")
	);
	format_case!(
		detect_txz,
		"backup.txz",
		Some(ArchiveFormat::TarXz),
		Some("backup")
	);
	format_case!(
		detect_tar_zst,
		"backup.tar.zst",
		Some(ArchiveFormat::TarZst),
		Some("backup")
	);
	format_case!(
		detect_tzst,
		"backup.tzst",
		Some(ArchiveFormat::TarZst),
		Some("backup")
	);
	format_case!(
		detect_seven_zip,
		"backup.7z",
		Some(ArchiveFormat::SevenZip),
		Some("backup")
	);
	format_case!(
		detect_mixed_case,
		"Backup.TaR.Gz",
		Some(ArchiveFormat::TarGz),
		Some("Backup")
	);
	format_case!(
		detect_nested,
		"folder/backup.tar",
		Some(ArchiveFormat::Tar),
		Some("backup")
	);
	format_case!(
		detect_multiple_dots,
		"backup.2026.zip",
		Some(ArchiveFormat::Zip),
		Some("backup.2026")
	);
	format_case!(
		detect_unicode_stem,
		"日本語.zip",
		Some(ArchiveFormat::Zip),
		Some("日本語")
	);
	format_case!(
		detect_hidden_stem,
		".backup.zip",
		Some(ArchiveFormat::Zip),
		Some(".backup")
	);
	format_case!(reject_empty_path, "", None, None);
	format_case!(reject_suffix_only, ".tar.gz", None, None);
	format_case!(reject_unknown_suffix, "backup.rar", None, None);
	format_case!(reject_bare_gz, "backup.gz", None, None);
	format_case!(reject_trailing_extension, "backup.zip.txt", None, None);
	format_case!(reject_missing_dot, "backupzip", None, None);
	format_case!(reject_partial_suffix, "backup.tar.g", None, None);
}
