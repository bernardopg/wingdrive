pub mod commands;
mod events;
mod session;

pub use commands::*;
pub use events::*;
pub use session::*;

#[cfg(test)]
mod tests;

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Runtime};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DragItem {
	pub kind: DragItemKind,
	pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum DragItemKind {
	File { path: String },
	FilePromise { name: String, mime_type: String },
	Text { content: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DragConfig {
	pub items: Vec<DragItem>,
	pub overlay_url: String,
	pub overlay_size: (f64, f64),
	pub allowed_operations: Vec<DragOperation>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DragOperation {
	Copy,
	Move,
	Link,
}

/// A session older than this is assumed lost (its end callback never came) and
/// no longer blocks a new drag.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
const STALE_SESSION: Duration = Duration::from_secs(30);

#[derive(Debug, Clone)]
pub struct DragCoordinator {
	state: Arc<RwLock<Option<(DragSession, Instant)>>>,
}

impl DragCoordinator {
	pub fn new() -> Self {
		Self {
			state: Arc::new(RwLock::new(None)),
		}
	}

	// Only the macOS native drag starts sessions.
	#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
	pub fn begin_drag<R: Runtime>(
		&self,
		app: &AppHandle<R>,
		config: DragConfig,
		source_window: String,
	) -> Result<(), String> {
		let mut state = self.state.write();

		// Check for stale state and clean it up
		if let Some((session, started_at)) = state.as_ref() {
			let elapsed = started_at.elapsed();
			if elapsed > STALE_SESSION {
				tracing::warn!(
					"Cleaning up stale drag session {} that started {:?} ago",
					session.id,
					elapsed
				);
				*state = None;
			} else {
				tracing::error!(
					"Drag operation already in progress: session_id={}, elapsed={:?}",
					session.id,
					elapsed
				);
				return Err("A drag operation is already in progress".to_string());
			}
		}

		let session = DragSession::new(config, source_window);
		tracing::info!("Starting drag session: session_id={}", session.id);
		*state = Some((session.clone(), Instant::now()));

		app.emit("drag:began", session.to_event()).ok();
		Ok(())
	}

	#[allow(dead_code)]
	pub fn update_position<R: Runtime>(&self, app: &AppHandle<R>, x: f64, y: f64) {
		if let Some((session, _)) = self.state.read().as_ref() {
			app.emit(
				"drag:moved",
				DragMoveEvent {
					session_id: session.id.clone(),
					x,
					y,
				},
			)
			.ok();
		}
	}

	#[allow(dead_code)]
	pub fn enter_window<R: Runtime>(&self, app: &AppHandle<R>, window_label: String) {
		if let Some((session, _)) = self.state.read().as_ref() {
			app.emit(
				"drag:entered",
				DragWindowEvent {
					session_id: session.id.clone(),
					window_label,
				},
			)
			.ok();
		}
	}

	#[allow(dead_code)]
	pub fn leave_window<R: Runtime>(&self, app: &AppHandle<R>, window_label: String) {
		if let Some((session, _)) = self.state.read().as_ref() {
			app.emit(
				"drag:left",
				DragWindowEvent {
					session_id: session.id.clone(),
					window_label,
				},
			)
			.ok();
		}
	}

	pub fn end_drag<R: Runtime>(&self, app: &AppHandle<R>, result: DragResult) {
		if let Some((session, started_at)) = self.state.write().take() {
			let elapsed = started_at.elapsed();
			tracing::info!(
				"Ending drag session: session_id={}, result={:?}, duration={:?}",
				session.id,
				result,
				elapsed
			);
			app.emit(
				"drag:ended",
				DragEndEvent {
					session_id: session.id,
					result,
				},
			)
			.ok();
		} else {
			tracing::warn!("end_drag called but no active session found");
		}
	}

	pub fn current_session(&self) -> Option<DragSession> {
		self.state
			.read()
			.as_ref()
			.map(|(session, _)| session.clone())
	}

	pub fn force_clear_state<R: Runtime>(&self, app: &AppHandle<R>) {
		if let Some((session, started_at)) = self.state.write().take() {
			tracing::warn!(
				"Force clearing drag state: session_id={}, elapsed={:?}",
				session.id,
				started_at.elapsed()
			);
			app.emit(
				"drag:ended",
				DragEndEvent {
					session_id: session.id,
					result: DragResult::Cancelled,
				},
			)
			.ok();
		}
	}
}

impl Default for DragCoordinator {
	fn default() -> Self {
		Self::new()
	}
}
