//! Exercises the thumbnail job with a real image in a temporary library.

mod helpers;

use std::time::Duration;

use helpers::IndexingHarnessBuilder;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use wing_core::{
	infra::{db::entities, job::output::JobOutput},
	location::IndexMode,
	ops::{
		media::thumbnail::{ThumbnailJob, ThumbnailJobConfig, ThumbnailVariantConfig},
		sidecar::{SidecarFormat, SidecarKind, SidecarVariant},
	},
};

#[tokio::test]
async fn generates_a_webp_and_reuses_the_existing_thumbnail() -> anyhow::Result<()> {
	let harness = IndexingHarnessBuilder::new("thumbnail-job").build().await?;
	let location = harness.create_test_location("images").await?;
	let source = location.path().join("sample.png");
	image::RgbImage::from_pixel(96, 48, image::Rgb([42, 99, 160])).save(&source)?;
	location
		.index("Thumbnail fixtures", IndexMode::Content)
		.await?;

	let db = harness.library.db().conn();
	let (entry, content) = entities::entry::Entity::find()
		.filter(entities::entry::Column::Name.eq("sample"))
		.filter(entities::entry::Column::Extension.eq("png"))
		.find_also_related(entities::content_identity::Entity)
		.one(db)
		.await?
		.expect("indexed image");
	let entry_id = entry.uuid.expect("image entry UUID");
	let content_id = content
		.expect("image content identity")
		.uuid
		.expect("content UUID");
	let config =
		ThumbnailJobConfig::with_variants(vec![ThumbnailVariantConfig::new(32, "fixture@1x", 81)]);

	let manager = harness
		.library
		.core_context()
		.get_sidecar_manager()
		.await
		.unwrap();
	let kind = SidecarKind::Thumb;
	let variant = SidecarVariant::new("fixture@1x");
	let format = SidecarFormat::Webp;
	let path = manager
		.compute_path(&harness.library.id(), &content_id, &kind, &variant, &format)
		.await?;
	assert!(!path.absolute_path.exists());

	for expected in [1, 0] {
		let job = harness
			.library
			.jobs()
			.dispatch(ThumbnailJob::for_entries(vec![entry_id], config.clone()))
			.await?;
		let JobOutput::ThumbnailGeneration {
			generated_count,
			error_count,
			total_size_bytes,
			..
		} = tokio::time::timeout(Duration::from_secs(30), job.wait()).await??
		else {
			panic!("unexpected thumbnail job output");
		};
		assert_eq!(error_count, 0);
		assert_eq!(generated_count, expected);
		if expected == 1 {
			assert_eq!(
				total_size_bytes,
				tokio::fs::metadata(&path.absolute_path).await?.len()
			);
		}
		let image = image::open(&path.absolute_path)?;
		assert_eq!((image.width(), image.height()), (32, 16));
	}
	let record = entities::sidecar::Entity::find()
		.filter(entities::sidecar::Column::ContentUuid.eq(content_id))
		.filter(entities::sidecar::Column::Variant.eq(variant.as_str()))
		.one(db)
		.await?
		.expect("thumbnail sidecar row");
	assert_eq!(record.status, "ready");
	assert_eq!(record.format, "webp");
	assert_eq!(record.rel_path, path.relative_path.to_string_lossy());

	harness.shutdown().await?;
	Ok(())
}
