//! `wing adapters`: the adapter store from the command line.

use anyhow::Result;
use clap::{Args, Subcommand};

use crate::context::Context;
use crate::util::confirm::confirm_or_abort;
use crate::util::prelude::*;

use wing_core::ops::adapters::store::{
	InstallAdapterInput, InstallAdapterOutput, ListStoreInput, RefreshStoreInput,
	RemoveAdapterInput, RemoveAdapterOutput, StoreCard, StoreListing, StorePermissions,
};

#[derive(Subcommand, Debug)]
pub enum AdaptersCmd {
	/// Download the newest signed index and list every adapter in the store
	Browse,
	/// List adapters from the last downloaded index, without the network
	List,
	/// Installed adapters that have an update or a revoked version
	Outdated,
	/// Install an adapter from the store
	Install(InstallArgs),
	/// Update one adapter, or all of them with --all
	Update(UpdateArgs),
	/// Remove an installed adapter from every library
	Remove(RemoveArgs),
}

#[derive(Args, Debug)]
pub struct InstallArgs {
	/// Adapter id, as `wing adapters browse` shows it
	pub id: String,
	/// Accept the permissions the adapter asks for without asking
	#[arg(long, short = 'y')]
	pub yes: bool,
}

#[derive(Args, Debug)]
pub struct UpdateArgs {
	/// Adapter id; leave out with --all
	pub id: Option<String>,
	/// Update every adapter with an update
	#[arg(long)]
	pub all: bool,
	/// Accept new permissions without asking
	#[arg(long, short = 'y')]
	pub yes: bool,
}

#[derive(Args, Debug)]
pub struct RemoveArgs {
	pub id: String,
	/// Do not ask for confirmation
	#[arg(long, short = 'y')]
	pub yes: bool,
}

fn permissions_text(p: &StorePermissions) -> String {
	let mut parts = Vec::new();
	for (label, items) in [
		("reads", &p.read),
		("network", &p.network),
		("secrets", &p.secrets),
		("runs", &p.tools),
	] {
		if !items.is_empty() {
			parts.push(format!("{label}: {}", items.join(", ")));
		}
	}
	if parts.is_empty() {
		"no special permissions".to_string()
	} else {
		parts.join("; ")
	}
}

fn print_cards(listing: &StoreListing, only_outdated: bool) {
	let Some(sequence) = listing.sequence else {
		println!("No index yet. Run `wing adapters browse` to download it.");
		return;
	};
	let cards: Vec<&StoreCard> = listing
		.adapters
		.iter()
		.filter(|c| !only_outdated || c.update_available)
		.collect();
	if cards.is_empty() {
		println!(
			"{} (index {sequence})",
			if only_outdated {
				"Everything is up to date"
			} else {
				"The store has no adapters yet"
			}
		);
		return;
	}
	for card in cards {
		let state = match (
			&card.installed_version,
			card.update_available,
			&card.revoked_reason,
		) {
			(Some(v), _, Some(reason)) => format!("installed {v}, REVOKED: {reason}"),
			(Some(v), true, None) => format!("installed {v}, update to {}", card.version),
			(Some(v), false, None) => format!("installed {v}"),
			(None, _, _) => format!("available {}", card.version),
		};
		println!("{} ({}) [{}] {}", card.name, card.id, card.trust, state);
		println!("    {}", card.summary);
		println!("    {}", permissions_text(&card.permissions));
		if !card.supported_here {
			println!("    does not run on this system");
		} else if !card.runtime_available {
			println!("    needs Python 3, which is not installed");
		}
	}
	println!("Index {sequence}");
}

async fn install(ctx: &Context, card: &StoreCard, yes: bool) -> Result<InstallAdapterOutput> {
	let has_permissions = !permissions_text(&card.permissions).starts_with("no special");
	let asks = card.installed_version.is_none() || card.update_needs_consent;
	if has_permissions && asks {
		println!(
			"{} asks for: {}",
			card.name,
			permissions_text(&card.permissions)
		);
		confirm_or_abort("Allow these permissions?", yes)?;
	}
	let input = InstallAdapterInput {
		id: card.id.clone(),
		accept_permissions: has_permissions && asks,
	};
	let out: InstallAdapterOutput = execute_core_action!(ctx, input);
	Ok(out)
}

pub async fn run(ctx: &Context, cmd: AdaptersCmd) -> Result<()> {
	match cmd {
		AdaptersCmd::Browse => {
			let listing: StoreListing = execute_core_action!(ctx, RefreshStoreInput {});
			print_output!(ctx, &listing, |l: &StoreListing| print_cards(l, false));
		}
		AdaptersCmd::List => {
			let listing: StoreListing = execute_core_query!(ctx, ListStoreInput {});
			print_output!(ctx, &listing, |l: &StoreListing| print_cards(l, false));
		}
		AdaptersCmd::Outdated => {
			let listing: StoreListing = execute_core_action!(ctx, RefreshStoreInput {});
			print_output!(ctx, &listing, |l: &StoreListing| print_cards(l, true));
		}
		AdaptersCmd::Install(args) => {
			let listing: StoreListing = execute_core_action!(ctx, RefreshStoreInput {});
			let card = listing
				.adapters
				.iter()
				.find(|c| c.id == args.id)
				.ok_or_else(|| anyhow::anyhow!("{} is not in the adapter store", args.id))?;
			let out = install(ctx, card, args.yes).await?;
			print_output!(
				ctx,
				&out,
				|o: &InstallAdapterOutput| match &o.previous_version {
					Some(previous) => println!("Updated {} from {previous} to {}", o.id, o.version),
					None => println!("Installed {} {}", o.id, o.version),
				}
			);
		}
		AdaptersCmd::Update(args) => {
			let listing: StoreListing = execute_core_action!(ctx, RefreshStoreInput {});
			let targets: Vec<&StoreCard> = listing
				.adapters
				.iter()
				.filter(|c| c.update_available)
				.filter(|c| args.all || args.id.as_deref() == Some(c.id.as_str()))
				.collect();
			if !args.all && args.id.is_none() {
				anyhow::bail!("Name an adapter or pass --all");
			}
			if targets.is_empty() {
				println!("Nothing to update");
			}
			for card in targets {
				let out = install(ctx, card, args.yes).await?;
				println!(
					"Updated {} from {} to {}",
					out.id,
					out.previous_version.as_deref().unwrap_or("?"),
					out.version
				);
			}
		}
		AdaptersCmd::Remove(args) => {
			confirm_or_abort(&format!("Remove {} from every library?", args.id), args.yes)?;
			let out: RemoveAdapterOutput =
				execute_core_action!(ctx, RemoveAdapterInput { id: args.id });
			print_output!(ctx, &out, |o: &RemoveAdapterOutput| println!(
				"Removed {}",
				o.id
			));
		}
	}
	Ok(())
}
