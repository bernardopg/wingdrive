//! Messages paired devices exchange about themselves.
//!
//! Pairing sends a device's description once. These messages keep that copy
//! current afterwards and tell a peer when its pairing was removed.

use crate::service::network::device::{DeviceInfo, DeviceType};
use serde::{Deserialize, Serialize};

/// Longest name, slug or version string accepted from a peer, in bytes.
const MAX_FIELD_BYTES: usize = 255;

/// Message about the sending device, carried by the messaging protocol.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceMessage {
	/// The sender's current description, sent every time it connects.
	Info(PeerDeviceInfo),
	/// Reply to [`DeviceMessage::Info`].
	InfoAck,
	/// The sender removed its pairing with the receiver.
	Unpaired,
	/// Reply to [`DeviceMessage::Unpaired`].
	UnpairedAck,
}

/// The part of a [`DeviceInfo`] its owner may change after pairing.
///
/// The device ID and network fingerprint are left out on purpose: the
/// receiver takes the sender's identity from the authenticated connection,
/// so a peer can only ever describe itself.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeerDeviceInfo {
	pub device_name: String,
	pub device_slug: String,
	pub device_type: DeviceType,
	pub os_version: String,
	pub app_version: String,
}

impl PeerDeviceInfo {
	/// Checks a description received from a peer before it is stored.
	pub fn validate(&self) -> Result<(), String> {
		let fields = [
			("device_name", &self.device_name),
			("device_slug", &self.device_slug),
			("os_version", &self.os_version),
			("app_version", &self.app_version),
		];
		for (field, value) in fields {
			if value.len() > MAX_FIELD_BYTES {
				return Err(format!("{field} is longer than {MAX_FIELD_BYTES} bytes"));
			}
			if value.chars().any(char::is_control) {
				return Err(format!("{field} contains control characters"));
			}
		}
		if self.device_name.trim().is_empty() {
			return Err("device_name is empty".to_string());
		}
		if let DeviceType::Other(kind) = &self.device_type {
			if kind.len() > MAX_FIELD_BYTES || kind.chars().any(char::is_control) {
				return Err("device_type is not a valid label".to_string());
			}
		}
		Ok(())
	}

	/// Writes this description into `info`. Returns whether anything changed.
	pub fn apply_to(&self, info: &mut DeviceInfo) -> bool {
		if *self == Self::from(&*info) {
			return false;
		}
		info.device_name = self.device_name.clone();
		info.device_slug = self.device_slug.clone();
		info.device_type = self.device_type.clone();
		info.os_version = self.os_version.clone();
		info.app_version = self.app_version.clone();
		true
	}
}

impl From<&DeviceInfo> for PeerDeviceInfo {
	fn from(info: &DeviceInfo) -> Self {
		Self {
			device_name: info.device_name.clone(),
			device_slug: info.device_slug.clone(),
			device_type: info.device_type.clone(),
			os_version: info.os_version.clone(),
			app_version: info.app_version.clone(),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn info() -> PeerDeviceInfo {
		PeerDeviceInfo {
			device_name: "motorola moto g(7)".to_string(),
			device_slug: "motorola-moto-g-7".to_string(),
			device_type: DeviceType::Mobile,
			os_version: "android".to_string(),
			app_version: "3.0.1".to_string(),
		}
	}

	#[test]
	fn accepts_a_normal_description() {
		assert_eq!(info().validate(), Ok(()));
	}

	#[test]
	fn rejects_empty_oversized_and_control_characters() {
		let mut blank = info();
		blank.device_name = "  ".to_string();
		assert!(blank.validate().is_err());

		let mut long = info();
		long.device_slug = "a".repeat(MAX_FIELD_BYTES + 1);
		assert!(long.validate().is_err());

		let mut control = info();
		control.device_name = "phone\nadmin".to_string();
		assert!(control.validate().is_err());

		let mut kind = info();
		kind.device_type = DeviceType::Other("x\u{0}".to_string());
		assert!(kind.validate().is_err());
	}

	#[test]
	fn wire_format_is_stable() {
		let json = serde_json::to_string(&DeviceMessage::Unpaired).unwrap();
		assert_eq!(json, "\"Unpaired\"");
		let json = serde_json::to_value(DeviceMessage::Info(info())).unwrap();
		assert_eq!(json["Info"]["device_type"], "Mobile");
		assert_eq!(json["Info"]["device_name"], "motorola moto g(7)");
	}
}
