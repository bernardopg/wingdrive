use std::{
	env::current_exe,
	path::{Path, PathBuf},
	sync::LazyLock,
};

use crate::{
	consts::{PDF_LANDSCAPE_RENDER_WIDTH, PDF_PORTRAIT_RENDER_WIDTH},
	Error, ImageHandler, Result,
};
use image::DynamicImage;
use pdfium_render::prelude::{PdfColor, PdfPageRenderRotation, PdfRenderConfig, Pdfium};
use tracing::error;

// This path must be relative to the running binary
#[cfg(windows)]
const BINDING_LOCATION: &str = ".";
#[cfg(unix)]
const BINDING_LOCATION: &str = if cfg!(target_os = "macos") {
	"../Frameworks/WingDrive.framework/Libraries"
} else {
	"../lib/wingdrive"
};

static PDFIUM_LIB: LazyLock<String> = LazyLock::new(|| {
	let lib_name = Pdfium::pdfium_platform_library_name();
	current_exe()
		.ok()
		.and_then(|exe_path| {
			exe_path.parent().and_then(|parent_path| {
				match parent_path
					.join(BINDING_LOCATION)
					.join(&lib_name)
					.canonicalize()
				{
					Ok(lib_path) => lib_path.to_str().map(ToOwned::to_owned),
					Err(err) => {
						error!("{err:#?}");
						None
					}
				}
			})
		})
		.unwrap_or_else(|| {
			#[allow(clippy::expect_used)]
			PathBuf::from(BINDING_LOCATION)
				.join(&lib_name)
				.to_str()
				.expect("We are converting valid strs to PathBuf then back, it should not fail")
				.to_owned()
		})
});

/// pdfium-render binds the library once per process: a second `bind_to_library` returns
/// `PdfiumLibraryBindingsAlreadyInitialized`, so binding per thumbnail failed every PDF after
/// the first. This instance binds on first use and serves every later thumbnail; a library
/// that fails to load stays unavailable until the daemon restarts.
static PDFIUM: LazyLock<std::result::Result<Pdfium, String>> = LazyLock::new(|| {
	Pdfium::bind_to_library(PDFIUM_LIB.as_str())
		.or_else(|err| {
			error!("{err:#?}");
			Pdfium::bind_to_system_library()
		})
		.map(Pdfium::new)
		.map_err(|err| format!("{err:?}"))
});

fn thumbnail_config(config: PdfRenderConfig) -> PdfRenderConfig {
	// From: https://github.com/ajrcarey/pdfium-render/blob/82c10b2d59b04a8413acd31892eb28822e60e06a/src/render_config.rs#L159
	config
		.rotate(PdfPageRenderRotation::None, false)
		.use_print_quality(false)
		.set_image_smoothing(false)
		.render_annotations(false)
		.render_form_data(false)
		// Required due to: https://github.com/ajrcarey/pdfium-render/issues/119
		.set_reverse_byte_order(false)
		.set_clear_color(PdfColor::new(255, 255, 255, 255))
		.clear_before_rendering(true)
}

static PORTRAIT_CONFIG: LazyLock<PdfRenderConfig> = LazyLock::new(|| {
	thumbnail_config(PdfRenderConfig::new().set_target_width(PDF_PORTRAIT_RENDER_WIDTH))
});

static LANDSCAPE_CONFIG: LazyLock<PdfRenderConfig> = LazyLock::new(|| {
	thumbnail_config(PdfRenderConfig::new().set_target_width(PDF_LANDSCAPE_RENDER_WIDTH))
});

pub struct PdfHandler {}

impl ImageHandler for PdfHandler {
	fn handle_image(&self, path: &Path) -> Result<DynamicImage> {
		let pdfium = PDFIUM
			.as_ref()
			.map_err(|err| Error::PdfiumUnavailable(err.clone()))?;

		let pdf = pdfium.load_pdf_from_file(path, None)?;
		let first_page = pdf.pages().first()?;

		let image = first_page
			.render_with_config(if first_page.is_portrait() {
				&PORTRAIT_CONFIG
			} else {
				&LANDSCAPE_CONFIG
			})?
			.as_image()?;

		Ok(image)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	/// A one-page PDF. pdfium rebuilds the missing cross-reference table.
	const PAGE: &[u8] = b"%PDF-1.4\n1 0 obj <</Type/Catalog/Pages 2 0 R>> endobj\n2 0 obj <</Type/Pages/Kids[3 0 R]/Count 1>> endobj\n3 0 obj <</Type/Page/Parent 2 0 R/MediaBox[0 0 200 300]>> endobj\ntrailer <</Root 1 0 R>>\n%%EOF\n";

	/// Every PDF gets a thumbnail, not only the first one the process renders. Runs where
	/// pdfium loads, for example with `LD_LIBRARY_PATH=apps/.deps/lib`.
	#[test]
	fn renders_more_than_one_pdf_per_process() {
		if let Err(err) = PDFIUM.as_ref() {
			eprintln!("skipping, pdfium is not available: {err}");
			return;
		}
		let dir = std::env::temp_dir().join(format!("wing-images-pdf-{}", std::process::id()));
		std::fs::create_dir_all(&dir).unwrap();
		for name in ["first.pdf", "second.pdf"] {
			let path = dir.join(name);
			std::fs::write(&path, PAGE).unwrap();
			let image = PdfHandler {}.handle_image(&path).unwrap();
			assert!(image.width() > 0, "{name} rendered empty");
		}
		std::fs::remove_dir_all(dir).unwrap();
	}
}
