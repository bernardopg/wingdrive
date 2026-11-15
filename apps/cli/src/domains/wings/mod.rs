use crate::{context::Context, util::prelude::*};
use anyhow::Result;
use clap::Subcommand;
use comfy_table::{presets::UTF8_BORDERS_ONLY, Cell, Table};
use wing_core::ops::wings::{WingsListQuery, WingsListQueryInput};

#[derive(Debug, Clone, Subcommand)]
pub enum WingsCmd {
	/// List all wings
	List,
	/// Get a wing's layout
	Layout {
		/// Wing ID
		wing_id: String,
	},
}

pub async fn exec(cmd: WingsCmd, ctx: &Context) -> Result<()> {
	match cmd {
		WingsCmd::List => list_wings(ctx).await,
		WingsCmd::Layout { wing_id } => get_layout(ctx, wing_id).await,
	}
}

async fn list_wings(ctx: &Context) -> Result<()> {
	// Get current library ID
	let library_id = ctx.library_id.ok_or_else(|| {
		anyhow::anyhow!("No library selected. Run 'wing library list' to see available libraries")
	})?;

	println!("Library ID: {}", library_id);

	// Create query input
	let input = WingsListQueryInput;

	// Execute query through the client
	let response = ctx.core.query(&input, Some(library_id)).await?;

	println!(
		"\nRaw response: {}",
		serde_json::to_string_pretty(&response)?
	);

	let result: wing_core::ops::wings::WingsListOutput = serde_json::from_value(response)
		.map_err(|e| anyhow::anyhow!("Failed to parse response: {}", e))?;

	println!("\nQuery executed successfully!");
	println!("Found {} wings:", result.wings.len());

	if result.wings.is_empty() {
		println!("  (no wings found)");
	} else {
		let mut table = Table::new();
		table.load_style(UTF8_BORDERS_ONLY);
		table.set_header(vec!["ID", "Name", "Icon", "Color", "Order"]);

		for wing in result.wings {
			table.add_row(vec![
				Cell::new(&wing.id.to_string()[..8]),
				Cell::new(&wing.name),
				Cell::new(&wing.icon),
				Cell::new(&wing.color),
				Cell::new(wing.order),
			]);
		}

		println!("{table}");
	}

	Ok(())
}

async fn get_layout(ctx: &Context, wing_id: String) -> Result<()> {
	use uuid::Uuid;
	use wing_core::ops::wings::{WingLayoutQuery, WingLayoutQueryInput};

	let library_id = ctx
		.library_id
		.ok_or_else(|| anyhow::anyhow!("No library selected"))?;

	let wing_uuid = Uuid::parse_str(&wing_id)?;

	let input = WingLayoutQueryInput { wing_id: wing_uuid };
	let response: serde_json::Value = ctx.core.query(&input, Some(library_id)).await?;

	println!("\nRaw layout response:");
	println!("{}", serde_json::to_string_pretty(&response)?);

	Ok(())
}
