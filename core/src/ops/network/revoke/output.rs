use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct DeviceRevokeOutput {
	pub revoked: bool,
	/// Whether the peer acknowledged the revocation and dropped the pairing
	/// on its side. `false` when it was offline or runs an older version.
	#[serde(default)]
	pub peer_notified: bool,
}
