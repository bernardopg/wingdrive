//! Events domain for monitoring event bus in real-time

mod args;

pub use args::*;

use crate::context::Context;
use anyhow::Result;
use chrono::{DateTime, Utc};
use std::collections::HashSet;
use wing_core::infra::daemon::types::EventFilter;
use wing_core::infra::event::Event;

/// Run events command
pub async fn run(ctx: &Context, cmd: EventsCmd) -> Result<()> {
	match cmd {
		EventsCmd::Monitor(args) => run_events_monitor(ctx, args).await,
	}
}

/// Monitor events in real-time
async fn run_events_monitor(ctx: &Context, args: EventsMonitorArgs) -> Result<()> {
	println!("Monitoring events - Press Ctrl+C to exit");

	// Parse event types filter
	let event_types_filter: Option<HashSet<String>> = args
		.event_type
		.as_ref()
		.map(|types| types.split(',').map(|s| s.trim().to_string()).collect());

	if let Some(ref types) = event_types_filter {
		println!(
			"Filtering by event types: {}",
			types.iter().cloned().collect::<Vec<_>>().join(", ")
		);
	}
	if let Some(ref lib_id) = args.library_id {
		println!("Filtering by library: {}", lib_id);
	}
	if let Some(ref job_id) = args.job_id {
		println!("Filtering by job: {}", job_id);
	}
	if let Some(ref device_id) = args.device_id {
		println!("Filtering by device: {}", device_id);
	}
	println!("═══════════════════════════════════════════════════════");

	// Create event filter for daemon
	let filter = EventFilter {
		library_id: args.library_id,
		job_id: args.job_id.clone(),
		device_id: args.device_id,
		resource_type: None,
		path_scope: None,
		include_descendants: None,
	};

	// Subscribe to all events (we'll filter by type client-side)
	let event_types = vec![];

	match ctx.core.subscribe_events(event_types, Some(filter)).await {
		Ok(mut event_stream) => {
			println!("Connected to event stream");
			println!();

			// Listen for events
			while let Some(event) = event_stream.recv().await {
				// Apply client-side event type filter
				if let Some(ref types_filter) = event_types_filter {
					let event_variant = event.variant_name();
					if !types_filter.contains(event_variant) {
						continue;
					}
				}

				// Display the event
				display_event(&event, &args);
			}
		}

		Err(e) => {
			println!("Failed to connect to event stream: {}", e);
			println!("Make sure the daemon is running");
		}
	}

	Ok(())
}

/// Display an event based on formatting preferences
fn display_event(event: &Event, args: &EventsMonitorArgs) {
	let timestamp = if args.timestamps {
		format!("[{}] ", Utc::now().format("%H:%M:%S%.3f"))
	} else {
		String::new()
	};

	let event_variant = event.variant_name();

	// Determine output format (new format flag takes precedence over legacy flags)
	let use_json_pretty = matches!(args.format, OutputFormat::JsonPretty) || args.pretty;
	let use_json = matches!(args.format, OutputFormat::Json) || use_json_pretty || args.verbose;

	if use_json {
		// JSON mode: show full JSON
		let json_str = if use_json_pretty {
			serde_json::to_string_pretty(&event).unwrap_or_else(|_| format!("{:?}", event))
		} else {
			serde_json::to_string(&event).unwrap_or_else(|_| format!("{:?}", event))
		};
		println!("{}{}: {}", timestamp, event_variant, json_str);
	} else {
		// Human-readable mode: show event type and key fields
		let summary = summarize_event(event);
		println!("{}{}: {}", timestamp, event_variant, summary);
	}
}

fn short_job_id(id: &str) -> String {
	id.chars().take(8).collect()
}

/// Create a human-readable summary of an event
fn summarize_event(event: &Event) -> String {
	match event {
		// Core lifecycle
		Event::CoreStarted => "Core started".to_string(),
		Event::CoreShutdown => "Core shutting down".to_string(),

		// Cache invalidation
		Event::Refresh => "Cache refresh triggered".to_string(),

		// Library events
		Event::LibraryCreated { name, id, .. } => {
			format!("Library '{}' created ({})", name, id)
		}
		Event::LibraryOpened { name, id, .. } => {
			format!("Library '{}' opened ({})", name, id)
		}
		Event::LibraryClosed { name, id } => {
			format!("Library '{}' closed ({})", name, id)
		}
		Event::LibraryDeleted {
			name,
			id,
			deleted_data,
		} => {
			format!(
				"Library '{}' deleted ({}, data: {})",
				name, id, deleted_data
			)
		}
		Event::LibraryLoadFailed {
			id,
			path,
			error,
			error_type,
		} => {
			if let Some(lib_id) = id {
				format!(
					"Failed to load library {} at {:?}: {} ({})",
					lib_id, path, error, error_type
				)
			} else {
				format!(
					"Failed to load library at {:?}: {} ({})",
					path, error, error_type
				)
			}
		}
		Event::LibraryStatisticsUpdated { library_id, .. } => {
			format!("Statistics updated for library {}", library_id)
		}

		// Entry events
		Event::EntryCreated {
			entry_id,
			library_id,
		} => {
			format!("Entry {} created in library {}", entry_id, library_id)
		}
		Event::EntryModified {
			entry_id,
			library_id,
		} => {
			format!("Entry {} modified in library {}", entry_id, library_id)
		}
		Event::EntryDeleted {
			entry_id,
			library_id,
		} => {
			format!("Entry {} deleted in library {}", entry_id, library_id)
		}
		Event::EntryMoved {
			entry_id,
			old_path,
			new_path,
			..
		} => {
			format!(
				"Entry {} moved from '{}' to '{}'",
				entry_id, old_path, new_path
			)
		}

		// Filesystem events
		Event::FsRawChange { library_id, kind } => {
			format!("Filesystem change in library {}: {:?}", library_id, kind)
		}

		// Volume events
		Event::VolumeAdded(vol) => {
			format!("Volume added: {} ({})", vol.name, vol.fingerprint.0)
		}
		Event::VolumeRemoved { fingerprint } => {
			format!("Volume removed: {}", fingerprint.0)
		}
		Event::VolumeUpdated { fingerprint, .. } => {
			format!("Volume updated: {}", fingerprint.0)
		}
		Event::VolumeSpeedTested {
			fingerprint,
			read_speed_mbps,
			write_speed_mbps,
		} => {
			format!(
				"Volume {} speed: read {}MB/s, write {}MB/s",
				fingerprint.0, read_speed_mbps, write_speed_mbps
			)
		}
		Event::VolumeMountChanged {
			fingerprint,
			is_mounted,
		} => {
			format!(
				"Volume {} mount changed: {}",
				fingerprint.0,
				if *is_mounted { "mounted" } else { "unmounted" }
			)
		}
		Event::VolumeError { fingerprint, error } => {
			format!("Volume {} error: {}", fingerprint.0, error)
		}

		// Job events
		Event::JobQueued {
			job_id, job_type, ..
		} => {
			format!("Job queued: {} ({})", job_type, short_job_id(job_id))
		}
		Event::JobStarted {
			job_id, job_type, ..
		} => {
			format!("Job started: {} ({})", job_type, short_job_id(job_id))
		}
		Event::JobProgress {
			job_id,
			job_type,
			progress,
			message,
			..
		} => {
			let msg = message
				.as_ref()
				.map(|m| format!(" - {}", m))
				.unwrap_or_default();
			format!(
				"Job progress: {} ({}) - {:.1}%{}",
				job_type,
				short_job_id(job_id),
				progress,
				msg
			)
		}
		Event::JobCompleted {
			job_id,
			job_type,
			output,
			..
		} => {
			format!(
				"Job completed: {} ({}) - {:?}",
				job_type,
				short_job_id(job_id),
				output
			)
		}
		Event::JobFailed {
			job_id,
			job_type,
			error,
			..
		} => {
			format!(
				"Job failed: {} ({}) - {}",
				job_type,
				short_job_id(job_id),
				error
			)
		}
		Event::JobCancelled {
			job_id, job_type, ..
		} => {
			format!("Job cancelled: {} ({})", job_type, short_job_id(job_id))
		}
		Event::JobPaused { job_id, .. } => {
			format!("Job paused: {}", short_job_id(job_id))
		}
		Event::JobResumed { job_id, .. } => {
			format!("Job resumed: {}", short_job_id(job_id))
		}

		// Indexing events
		Event::IndexingStarted { location_id } => {
			format!("Indexing started for location {}", location_id)
		}
		Event::IndexingProgress {
			location_id,
			processed,
			total,
		} => {
			let total_str = total.map(|t| format!("/{}", t)).unwrap_or_default();
			format!(
				"Indexing location {}: {}{} processed",
				location_id, processed, total_str
			)
		}
		Event::IndexingCompleted {
			location_id,
			total_files,
			total_dirs,
		} => {
			format!(
				"Indexing completed for location {}: {} files, {} dirs",
				location_id, total_files, total_dirs
			)
		}
		Event::IndexingFailed { location_id, error } => {
			format!("Indexing failed for location {}: {}", location_id, error)
		}

		// Device events
		Event::DeviceConnected {
			device_id,
			device_name,
		} => {
			format!("Device connected: {} ({})", device_name, device_id)
		}
		Event::DeviceDisconnected { device_id } => {
			format!("Device disconnected: {}", device_id)
		}

		// Resource events
		Event::ResourceChanged {
			resource_type,
			resource,
			..
		} => {
			if let Some(id) = resource.get("id") {
				format!("Resource changed: {} ({})", resource_type, id)
			} else {
				format!("Resource changed: {}", resource_type)
			}
		}
		Event::ResourceChangedBatch {
			resource_type,
			resources,
			..
		} => {
			if let Some(arr) = resources.as_array() {
				format!("Resources changed: {} ({} items)", resource_type, arr.len())
			} else {
				format!("Resources changed: {} (batch)", resource_type)
			}
		}
		Event::ResourceDeleted {
			resource_type,
			resource_id,
		} => {
			format!("Resource deleted: {} ({})", resource_type, resource_id)
		}

		// Legacy location events
		Event::LocationAdded {
			location_id, path, ..
		} => {
			format!("Location added: {} at {}", location_id, path.display())
		}
		Event::LocationRemoved { location_id, .. } => {
			format!("Location removed: {}", location_id)
		}
		Event::FilesIndexed {
			location_id, count, ..
		} => {
			format!("Files indexed: {} files at location {}", count, location_id)
		}
		Event::ThumbnailsGenerated { count, .. } => {
			format!("Thumbnails generated: {}", count)
		}
		Event::FileOperationCompleted {
			operation,
			affected_files,
			..
		} => {
			format!(
				"File operation completed: {:?} ({} files)",
				operation, affected_files
			)
		}
		Event::FilesModified { paths, .. } => {
			format!("Files modified: {} files", paths.len())
		}

		// Sync events
		Event::SyncStateChanged {
			previous_state,
			new_state,
			..
		} => {
			format!("Sync state changed: {} → {}", previous_state, new_state)
		}
		Event::SyncActivity {
			peer_device_id,
			count,
			..
		} => {
			format!(
				"Sync activity: {} changes from peer {}",
				count, peer_device_id
			)
		}
		Event::SyncConnectionChanged {
			peer_name,
			connected,
			..
		} => {
			format!(
				"Peer {} {}",
				peer_name,
				if *connected {
					"connected"
				} else {
					"disconnected"
				}
			)
		}
		Event::SyncError { message, .. } => {
			format!("Sync error: {}", message)
		}

		// Proxy pairing events
		Event::ProxyPairingConfirmationRequired {
			vouchee_device_name,
			voucher_device_name,
			..
		} => {
			format!(
				"Proxy pairing confirmation required: {} vouched by {}",
				vouchee_device_name, voucher_device_name
			)
		}
		Event::ProxyPairingVouchingReady {
			vouchee_device_id, ..
		} => {
			format!(
				"Proxy pairing vouching ready for device {}",
				vouchee_device_id
			)
		}

		// Custom events
		Event::Custom { event_type, data } => {
			format!("Custom event: {} - {:?}", event_type, data)
		}

		// Config events
		Event::ConfigChanged { field } => {
			format!("Configuration changed: {}", field)
		}
		Event::WingdropOffered { offer } => format!(
			"Wingdrop offer {} from {}: {} files, {} bytes",
			offer.id, offer.from_device_name, offer.file_count, offer.total_bytes
		),
		Event::WingdropUpdated { drop_id, status } => {
			format!("Wingdrop {}: {:?}", drop_id, status)
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use uuid::Uuid;

	#[test]
	fn short_job_id_is_rendered_without_panicking() {
		let event = Event::JobPaused {
			job_id: "é".into(),
			device_id: Uuid::nil(),
		};
		assert!(summarize_event(&event).contains('é'));
	}

	#[test]
	fn config_change_summary_includes_changed_field() {
		let event = Event::ConfigChanged {
			field: "language".into(),
		};
		assert!(summarize_event(&event).contains("language"));
	}

	#[test]
	fn lifecycle_and_entry_summaries_include_identity() {
		let library = Uuid::new_v4();
		let entry = Uuid::new_v4();
		assert_eq!(summarize_event(&Event::CoreStarted), "Core started");
		assert_eq!(summarize_event(&Event::CoreShutdown), "Core shutting down");
		assert_eq!(summarize_event(&Event::Refresh), "Cache refresh triggered");
		assert!(summarize_event(&Event::EntryCreated {
			library_id: library,
			entry_id: entry
		})
		.contains(&entry.to_string()));
		assert!(summarize_event(&Event::EntryModified {
			library_id: library,
			entry_id: entry
		})
		.contains(&library.to_string()));
		assert!(summarize_event(&Event::EntryDeleted {
			library_id: library,
			entry_id: entry
		})
		.contains(&entry.to_string()));
		assert!(summarize_event(&Event::EntryMoved {
			library_id: library,
			entry_id: entry,
			old_path: "before".into(),
			new_path: "after".into()
		})
		.contains("after"));
	}

	#[test]
	fn indexing_and_device_events_include_progress_and_errors() {
		let id = Uuid::new_v4();
		assert!(
			summarize_event(&Event::IndexingStarted { location_id: id }).contains(&id.to_string())
		);
		assert!(summarize_event(&Event::IndexingProgress {
			location_id: id,
			processed: 3,
			total: Some(10)
		})
		.contains("3/10"));
		assert!(summarize_event(&Event::IndexingProgress {
			location_id: id,
			processed: 3,
			total: None
		})
		.contains("3 processed"));
		assert!(summarize_event(&Event::IndexingCompleted {
			location_id: id,
			total_files: 4,
			total_dirs: 2
		})
		.contains("4 files, 2 dirs"));
		assert!(summarize_event(&Event::IndexingFailed {
			location_id: id,
			error: "failure".into()
		})
		.contains("failure"));
		assert!(summarize_event(&Event::DeviceConnected {
			device_id: id,
			device_name: "remote".into()
		})
		.contains("remote"));
		assert!(
			summarize_event(&Event::DeviceDisconnected { device_id: id }).contains(&id.to_string())
		);
	}

	#[test]
	fn job_events_render_short_ids_and_messages() {
		let device = Uuid::nil();
		let id = "abc".to_string();
		let events = [
			Event::JobQueued {
				job_id: id.clone(),
				job_type: "copy".into(),
				device_id: device,
			},
			Event::JobStarted {
				job_id: id.clone(),
				job_type: "copy".into(),
				device_id: device,
			},
			Event::JobProgress {
				job_id: id.clone(),
				job_type: "copy".into(),
				device_id: device,
				progress: 50.0,
				message: Some("moving".into()),
				generic_progress: None,
			},
			Event::JobFailed {
				job_id: id.clone(),
				job_type: "copy".into(),
				device_id: device,
				error: "failure".into(),
			},
			Event::JobCancelled {
				job_id: id.clone(),
				job_type: "copy".into(),
				device_id: device,
			},
			Event::JobResumed {
				job_id: id,
				device_id: device,
			},
		];
		for event in events {
			assert!(summarize_event(&event).contains("abc"));
		}
	}

	#[test]
	fn library_events_render_names_and_failure_details() {
		let id = Uuid::new_v4();
		let events = [
			Event::LibraryCreated {
				id,
				name: "work".into(),
				path: "fixture".into(),
				source: Default::default(),
			},
			Event::LibraryOpened {
				id,
				name: "work".into(),
				path: "fixture".into(),
			},
			Event::LibraryClosed {
				id,
				name: "work".into(),
			},
			Event::LibraryDeleted {
				id,
				name: "work".into(),
				deleted_data: false,
			},
		];
		for event in events {
			assert!(summarize_event(&event).contains("work"));
		}
		let fail = Event::LibraryLoadFailed {
			id: Some(id),
			path: "fixture".into(),
			error: "broken".into(),
			error_type: "config".into(),
		};
		assert!(summarize_event(&fail).contains("broken"));
	}

	#[test]
	fn display_supports_human_and_json_formats() {
		let event = Event::CoreStarted;
		for format in [
			OutputFormat::Human,
			OutputFormat::Json,
			OutputFormat::JsonPretty,
		] {
			let args = EventsMonitorArgs {
				event_type: None,
				library_id: None,
				job_id: None,
				device_id: None,
				timestamps: true,
				format,
				verbose: false,
				pretty: false,
			};
			display_event(&event, &args);
		}
	}

	#[test]
	fn resource_events_include_identifiers_and_batch_counts() {
		let id = Uuid::new_v4();
		let changed = Event::ResourceChanged {
			resource_type: "file".into(),
			resource: serde_json::json!({"id":id}),
			metadata: None,
		};
		assert!(summarize_event(&changed).contains(&id.to_string()));
		let batch = Event::ResourceChangedBatch {
			resource_type: "file".into(),
			resources: serde_json::json!([1, 2]),
			metadata: None,
		};
		assert!(summarize_event(&batch).contains("2 items"));
		let deleted = Event::ResourceDeleted {
			resource_type: "file".into(),
			resource_id: id,
		};
		assert!(summarize_event(&deleted).contains(&id.to_string()));
	}

	#[test]
	fn sync_events_summarize_state_connections_and_errors() {
		let id = Uuid::new_v4();
		let state = Event::SyncStateChanged {
			library_id: id,
			previous_state: "paused".into(),
			new_state: "ready".into(),
			timestamp: "now".into(),
		};
		assert!(summarize_event(&state).contains("paused → ready"));
		let activity = Event::SyncActivity {
			library_id: id,
			peer_device_id: id,
			activity_type: wing_core::infra::event::SyncActivityType::ChangesApplied { changes: 3 },
			model_type: None,
			count: 3,
			timestamp: "now".into(),
		};
		assert!(summarize_event(&activity).contains("3 changes"));
		for (connected, expected) in [(true, "connected"), (false, "disconnected")] {
			let connection = Event::SyncConnectionChanged {
				library_id: id,
				peer_device_id: id,
				peer_name: "peer".into(),
				connected,
				timestamp: "now".into(),
			};
			assert!(summarize_event(&connection).contains(expected));
		}
		let error = Event::SyncError {
			library_id: id,
			peer_device_id: None,
			error_type: "fixture".into(),
			message: "failure".into(),
			timestamp: "now".into(),
		};
		assert!(summarize_event(&error).contains("failure"));
	}

	#[test]
	fn legacy_location_events_report_counts_and_paths() {
		let id = Uuid::new_v4();
		let events = [
			(
				Event::LocationAdded {
					library_id: id,
					location_id: id,
					path: "fixture".into(),
				},
				"fixture",
			),
			(
				Event::LocationRemoved {
					library_id: id,
					location_id: id,
				},
				"removed",
			),
			(
				Event::FilesIndexed {
					library_id: id,
					location_id: id,
					count: 5,
				},
				"5 files",
			),
			(
				Event::ThumbnailsGenerated {
					library_id: id,
					count: 3,
				},
				"3",
			),
			(
				Event::FilesModified {
					library_id: id,
					paths: vec!["fixture".into()],
				},
				"1 files",
			),
		];
		for (event, expected) in events {
			assert!(summarize_event(&event).contains(expected));
		}
	}
}
