use clap::Args;
use wing_core::{
	ops::volumes::{
		add_cloud::{CloudStorageConfig, VolumeAddCloudInput},
		remove_cloud::VolumeRemoveCloudInput,
	},
	volume::{backend::CloudServiceType, VolumeFingerprint},
};

#[derive(Args, Debug)]
pub struct VolumeAddCloudArgs {
	/// Display name for the cloud volume
	pub name: String,

	/// Cloud service type
	#[arg(long, value_enum)]
	pub service: CloudServiceArg,

	/// Bucket name (S3, GCS)
	#[arg(long)]
	pub bucket: Option<String>,

	/// Region (S3)
	#[arg(long)]
	pub region: Option<String>,

	/// Access key ID (S3, Azure)
	#[arg(long)]
	pub access_key_id: Option<String>,

	/// Secret access key (S3, Azure)
	#[arg(long)]
	pub secret_access_key: Option<String>,

	/// Custom endpoint (S3, Azure, GCS)
	#[arg(long)]
	pub endpoint: Option<String>,

	/// Root folder path or ID (Google Drive, OneDrive, Dropbox, GCS)
	#[arg(long)]
	pub root: Option<String>,

	/// OAuth access token (Google Drive, OneDrive, Dropbox)
	#[arg(long)]
	pub access_token: Option<String>,

	/// OAuth refresh token (Google Drive, OneDrive, Dropbox)
	#[arg(long)]
	pub refresh_token: Option<String>,

	/// OAuth client ID (Google Drive, OneDrive, Dropbox)
	#[arg(long)]
	pub client_id: Option<String>,

	/// OAuth client secret (Google Drive, OneDrive, Dropbox)
	#[arg(long)]
	pub client_secret: Option<String>,

	/// Container name (Azure Blob)
	#[arg(long)]
	pub container: Option<String>,

	/// Storage account name (Azure Blob)
	#[arg(long)]
	pub account_name: Option<String>,

	/// Storage account key (Azure Blob)
	#[arg(long)]
	pub account_key: Option<String>,

	/// Path to service account JSON file (GCS)
	#[arg(long)]
	pub service_account: Option<String>,
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum CloudServiceArg {
	S3,
	GoogleDrive,
	Dropbox,
	OneDrive,
	GoogleCloudStorage,
	AzureBlob,
	BackblazeB2,
	Wasabi,
	DigitalOceanSpaces,
}

impl From<CloudServiceArg> for CloudServiceType {
	fn from(arg: CloudServiceArg) -> Self {
		match arg {
			CloudServiceArg::S3 => CloudServiceType::S3,
			CloudServiceArg::GoogleDrive => CloudServiceType::GoogleDrive,
			CloudServiceArg::Dropbox => CloudServiceType::Dropbox,
			CloudServiceArg::OneDrive => CloudServiceType::OneDrive,
			CloudServiceArg::GoogleCloudStorage => CloudServiceType::GoogleCloudStorage,
			CloudServiceArg::AzureBlob => CloudServiceType::AzureBlob,
			CloudServiceArg::BackblazeB2 => CloudServiceType::BackblazeB2,
			CloudServiceArg::Wasabi => CloudServiceType::Wasabi,
			CloudServiceArg::DigitalOceanSpaces => CloudServiceType::DigitalOceanSpaces,
		}
	}
}

impl VolumeAddCloudArgs {
	pub fn validate_and_build(self) -> Result<VolumeAddCloudInput, String> {
		let service = CloudServiceType::from(self.service.clone());

		let config = match self.service {
			CloudServiceArg::S3
			| CloudServiceArg::BackblazeB2
			| CloudServiceArg::Wasabi
			| CloudServiceArg::DigitalOceanSpaces => {
				let bucket = self
					.bucket
					.ok_or("--bucket is required for S3-compatible services")?;
				let region = self
					.region
					.ok_or("--region is required for S3-compatible services")?;
				let access_key_id = self
					.access_key_id
					.ok_or("--access-key-id is required for S3-compatible services")?;
				let secret_access_key = self
					.secret_access_key
					.ok_or("--secret-access-key is required for S3-compatible services")?;

				CloudStorageConfig::S3 {
					bucket,
					region,
					access_key_id,
					secret_access_key,
					endpoint: self.endpoint,
				}
			}
			CloudServiceArg::GoogleDrive => {
				return Err(
					"Google Drive connects through the browser: run `wing integrations connect google`"
						.to_string(),
				);
			}
			CloudServiceArg::OneDrive => {
				let access_token = self
					.access_token
					.ok_or("--access-token is required for OneDrive")?;
				let refresh_token = self
					.refresh_token
					.ok_or("--refresh-token is required for OneDrive")?;
				let client_id = self
					.client_id
					.ok_or("--client-id is required for OneDrive")?;
				let client_secret = self
					.client_secret
					.ok_or("--client-secret is required for OneDrive")?;

				CloudStorageConfig::OneDrive {
					root: self.root,
					access_token,
					refresh_token,
					client_id,
					client_secret,
				}
			}
			CloudServiceArg::Dropbox => {
				let refresh_token = self
					.refresh_token
					.ok_or("--refresh-token is required for Dropbox")?;
				let client_id = self
					.client_id
					.ok_or("--client-id is required for Dropbox")?;
				let client_secret = self
					.client_secret
					.ok_or("--client-secret is required for Dropbox")?;

				CloudStorageConfig::Dropbox {
					root: self.root,
					refresh_token,
					client_id,
					client_secret,
				}
			}
			CloudServiceArg::AzureBlob => {
				let container = self
					.container
					.ok_or("--container is required for Azure Blob")?;
				let account_name = self
					.account_name
					.ok_or("--account-name is required for Azure Blob")?;
				let account_key = self
					.account_key
					.ok_or("--account-key is required for Azure Blob")?;

				CloudStorageConfig::AzureBlob {
					container,
					endpoint: self.endpoint,
					account_name,
					account_key,
				}
			}
			CloudServiceArg::GoogleCloudStorage => {
				let bucket = self
					.bucket
					.ok_or("--bucket is required for Google Cloud Storage")?;
				let service_account_path = self
					.service_account
					.ok_or("--service-account is required for Google Cloud Storage")?;

				let credential = std::fs::read_to_string(&service_account_path).map_err(|e| {
					format!(
						"Failed to read service account file '{}': {}",
						service_account_path, e
					)
				})?;

				CloudStorageConfig::GoogleCloudStorage {
					bucket,
					root: self.root,
					endpoint: self.endpoint,
					credential,
				}
			}
		};

		Ok(VolumeAddCloudInput {
			service,
			display_name: self.name,
			config,
		})
	}
}

#[derive(Args, Debug)]
pub struct VolumeRemoveCloudArgs {
	/// Volume fingerprint (from volume list)
	pub fingerprint: String,

	/// Skip confirmation prompt
	#[arg(long, short = 'y', default_value_t = false)]
	pub yes: bool,
}

impl TryFrom<VolumeRemoveCloudArgs> for VolumeRemoveCloudInput {
	type Error = String;

	fn try_from(args: VolumeRemoveCloudArgs) -> Result<Self, String> {
		let fingerprint = VolumeFingerprint::from_string(&args.fingerprint)
			.map_err(|e| format!("Invalid fingerprint: {}", e))?;

		Ok(Self { fingerprint })
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use clap::Parser;

	#[derive(Parser)]
	struct AddCli {
		#[command(flatten)]
		args: VolumeAddCloudArgs,
	}

	fn parse(args: &[&str]) -> VolumeAddCloudArgs {
		AddCli::try_parse_from(args).unwrap().args
	}

	#[test]
	fn s3_compatible_services_keep_provider_and_credentials() {
		for (service, expected) in [
			("s3", CloudServiceType::S3),
			("backblaze-b2", CloudServiceType::BackblazeB2),
			("wasabi", CloudServiceType::Wasabi),
			("digital-ocean-spaces", CloudServiceType::DigitalOceanSpaces),
		] {
			let input = parse(&[
				"wing",
				"My volume",
				"--service",
				service,
				"--bucket",
				"test-bucket",
				"--region",
				"test-region",
				"--access-key-id",
				"fake-id",
				"--secret-access-key",
				"fake-secret",
				"--endpoint",
				"https://example.invalid",
			])
			.validate_and_build()
			.unwrap();
			assert_eq!(input.service, expected);
			assert_eq!(input.display_name, "My volume");
			let CloudStorageConfig::S3 {
				bucket,
				region,
				access_key_id,
				secret_access_key,
				endpoint,
			} = input.config
			else {
				panic!("expected S3 configuration")
			};
			assert_eq!(
				(
					bucket.as_str(),
					region.as_str(),
					access_key_id.as_str(),
					secret_access_key.as_str()
				),
				("test-bucket", "test-region", "fake-id", "fake-secret")
			);
			assert_eq!(endpoint.as_deref(), Some("https://example.invalid"));
		}
	}

	#[test]
	fn s3_reports_each_missing_required_field() {
		let required = [
			("--bucket", "bucket"),
			("--region", "region"),
			("--access-key-id", "key"),
			("--secret-access-key", "secret"),
		];
		for (missing, _) in required {
			let mut args = vec!["wing", "name", "--service", "s3"];
			for (flag, value) in required {
				if flag != missing {
					args.extend([flag, value]);
				}
			}
			assert!(parse(&args)
				.validate_and_build()
				.unwrap_err()
				.contains(missing));
		}
	}

	#[test]
	fn google_drive_points_to_the_browser_sign_in() {
		let error = parse(&["wing", "name", "--service", "google-drive"])
			.validate_and_build()
			.unwrap_err();
		assert!(error.contains("wing integrations connect google"));
	}

	#[test]
	fn onedrive_requires_access_token_and_accepts_full_credentials() {
		assert!(parse(&["wing", "name", "--service", "one-drive"])
			.validate_and_build()
			.unwrap_err()
			.contains("--access-token"));
		let input = parse(&[
			"wing",
			"name",
			"--service",
			"one-drive",
			"--access-token",
			"access",
			"--refresh-token",
			"refresh",
			"--client-id",
			"id",
			"--client-secret",
			"secret",
		])
		.validate_and_build()
		.unwrap();
		assert!(matches!(
			input.config,
			CloudStorageConfig::OneDrive { root: None, .. }
		));
	}

	#[test]
	fn dropbox_uses_refresh_token_without_access_token() {
		let input = parse(&[
			"wing",
			"name",
			"--service",
			"dropbox",
			"--refresh-token",
			"refresh",
			"--client-id",
			"id",
			"--client-secret",
			"secret",
		])
		.validate_and_build()
		.unwrap();
		assert!(
			matches!(input.config, CloudStorageConfig::Dropbox { refresh_token, .. } if refresh_token == "refresh")
		);
	}

	#[test]
	fn azure_requires_account_and_preserves_endpoint() {
		assert!(parse(&[
			"wing",
			"name",
			"--service",
			"azure-blob",
			"--container",
			"test"
		])
		.validate_and_build()
		.unwrap_err()
		.contains("--account-name"));
		let input = parse(&[
			"wing",
			"name",
			"--service",
			"azure-blob",
			"--container",
			"test",
			"--account-name",
			"account",
			"--account-key",
			"fake",
			"--endpoint",
			"https://example.invalid",
		])
		.validate_and_build()
		.unwrap();
		assert!(
			matches!(input.config, CloudStorageConfig::AzureBlob { container, endpoint: Some(endpoint), .. } if container == "test" && endpoint == "https://example.invalid")
		);
	}

	#[test]
	fn google_cloud_reads_service_account_from_file() {
		let dir = tempfile::tempdir().unwrap();
		let path = dir.path().join("fake.json");
		std::fs::write(&path, "{\"test\":true}").unwrap();
		let input = parse(&[
			"wing",
			"name",
			"--service",
			"google-cloud-storage",
			"--bucket",
			"test",
			"--service-account",
			path.to_str().unwrap(),
			"--root",
			"folder",
		])
		.validate_and_build()
		.unwrap();
		assert!(
			matches!(input.config, CloudStorageConfig::GoogleCloudStorage { bucket, root: Some(root), credential, .. } if bucket == "test" && root == "folder" && credential == "{\"test\":true}")
		);
	}

	#[test]
	fn google_cloud_reports_unreadable_service_account() {
		let dir = tempfile::tempdir().unwrap();
		let path = dir.path().join("missing.json");
		let error = parse(&[
			"wing",
			"name",
			"--service",
			"google-cloud-storage",
			"--bucket",
			"test",
			"--service-account",
			path.to_str().unwrap(),
		])
		.validate_and_build()
		.unwrap_err();
		assert!(error.contains("Failed to read service account file"));
	}
}
