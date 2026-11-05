use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipe {
	pub name: String,
	pub seed: Option<u64>,
	pub locations: Vec<RecipeLocation>,
	#[serde(default)]
	pub media: Option<RecipeMedia>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeLocation {
	pub path: PathBuf,
	pub structure: Structure,
	pub files: FileSpec,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Structure {
	pub depth: usize,
	pub fanout_per_dir: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSpec {
	pub total: usize,
	pub size_buckets: HashMap<String, SizeBucket>,
	#[serde(default)]
	pub duplicate_ratio: Option<f32>,
	#[serde(default)]
	pub media_ratio: Option<f32>,
	#[serde(default)]
	pub extensions: Option<Vec<String>>,
	#[serde(default)]
	pub content_gen: Option<ContentGen>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SizeBucket {
	pub range: [u64; 2],
	pub share: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeMedia {
	#[serde(default)]
	pub generate_thumbnails: Option<bool>,
	#[serde(default)]
	pub synthetic_video: Option<SynthVideo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynthVideo {
	pub enabled: bool,
	pub duration_s: Option<u32>,
	pub width: Option<u32>,
	pub height: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentMode {
	Zeros,
	Partial,
	Full,
}

impl Default for ContentMode {
	fn default() -> Self {
		ContentMode::Zeros
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentGen {
	#[serde(default)]
	pub mode: ContentMode,
	/// Size of each inner sample block; should match content hash SAMPLE_SIZE (default 10 KiB)
	#[serde(default = "default_sample_block_size")]
	pub sample_block_size: u64,
	/// Write magic headers based on file extension and registry definitions
	#[serde(default)]
	pub magic_headers: bool,
}

fn default_sample_block_size() -> u64 {
	10 * 1024
}

#[cfg(test)]
mod tests {
	use super::Recipe;

	/// Every recipe shipped in `recipes/` must parse, so a YAML parser change or a schema
	/// change that breaks one fails here instead of at `wing-bench mkdata`.
	#[test]
	fn shipped_recipes_parse() {
		let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("recipes");
		let mut parsed = 0;
		for entry in std::fs::read_dir(&dir).expect("recipes directory") {
			let path = entry.expect("recipe entry").path();
			if path.extension().and_then(|e| e.to_str()) != Some("yaml") {
				continue;
			}
			let text = std::fs::read_to_string(&path).expect("readable recipe");
			let recipe: Recipe = serde_saphyr::from_str(&text)
				.unwrap_or_else(|e| panic!("{} does not parse: {e}", path.display()));
			assert!(
				!recipe.locations.is_empty(),
				"{} has no locations",
				path.display()
			);
			parsed += 1;
		}
		assert!(parsed > 0, "no recipes found in {}", dir.display());
	}
}
