mod helpers;

use std::sync::Arc;

use helpers::IndexingHarnessBuilder;
use wing_core::{
	domain::tag::{PrivacyLevel, RelationshipType, TagType},
	ops::tags::TagManager,
};

#[tokio::test]
async fn searches_aliases_and_respects_namespace_and_type_filters() -> anyhow::Result<()> {
	let harness = IndexingHarnessBuilder::new("tag-manager-search")
		.disable_watcher()
		.build()
		.await?;
	let manager = TagManager::new(Arc::new(harness.library.db().conn().clone()));
	let tag = manager
		.create_tag_entity_full(
			"Project".into(),
			Some("office".into()),
			None,
			None,
			None,
			vec!["uniquealias".into()],
			Some(TagType::Standard),
			None,
			None,
			None,
			false,
			None,
			None,
			None,
			harness.device_id,
		)
		.await?;

	let found = manager
		.search_tags(
			"uniquealias",
			Some("office"),
			Some(TagType::Standard),
			false,
		)
		.await?;
	assert_eq!(found.len(), 1);
	assert_eq!(found[0].id, tag.uuid);
	assert!(manager
		.search_tags("uniquealias", Some("elsewhere"), None, false)
		.await?
		.is_empty());
	assert!(manager
		.search_tags("uniquealias", Some("office"), Some(TagType::Privacy), false)
		.await?
		.is_empty());
	assert!(manager
		.search_tags("absent", None, None, false)
		.await?
		.is_empty());
	let mut archived = manager
		.create_tag("Archived".into(), Some("office".into()), harness.device_id)
		.await?;
	archived.privacy_level = PrivacyLevel::Archive;
	archived.display_name = Some("Old project".into());
	let updated = manager.update_tag(&archived).await?;
	assert_eq!(updated.display_name.as_deref(), Some("Old project"));
	assert_eq!(updated.privacy_level, PrivacyLevel::Archive);
	let visible = manager.search_tags("", Some("office"), None, false).await?;
	assert!(visible.iter().any(|item| item.id == tag.uuid));
	assert!(!visible.iter().any(|item| item.id == archived.id));
	let including_archived = manager.search_tags("", Some("office"), None, true).await?;
	assert!(including_archived.iter().any(|item| item.id == archived.id));
	harness.shutdown().await?;
	Ok(())
}

#[tokio::test]
async fn hierarchy_tracks_children_and_rejects_cycles() -> anyhow::Result<()> {
	let harness = IndexingHarnessBuilder::new("tag-manager-hierarchy")
		.disable_watcher()
		.build()
		.await?;
	let manager = TagManager::new(Arc::new(harness.library.db().conn().clone()));
	let parent = manager
		.create_tag("Parent".into(), None, harness.device_id)
		.await?;
	let child = manager
		.create_tag("Child".into(), None, harness.device_id)
		.await?;
	manager
		.create_relationship(parent.id, child.id, RelationshipType::ParentChild, None)
		.await?;
	assert!(manager
		.get_direct_children(parent.id)
		.await?
		.contains(&child.id));
	assert!(manager
		.get_ancestors(child.id)
		.await?
		.iter()
		.any(|tag| tag.id == parent.id));
	assert!(manager
		.create_relationship(child.id, parent.id, RelationshipType::ParentChild, None)
		.await
		.is_err());
	manager.delete_tag(child.id).await?;
	assert!(manager.get_direct_children(parent.id).await?.is_empty());
	assert!(manager.get_tags_by_ids(&[child.id]).await?.is_empty());
	assert!(manager.delete_tag(child.id).await.is_err());
	harness.shutdown().await?;
	Ok(())
}
