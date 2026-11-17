//! The Windows scheduled task that starts the daemon at login.
//!
//! `schtasks /Create /XML` reads a Task Scheduler definition. The text is built here, apart from
//! the `schtasks` calls in `main.rs`, so the Linux test run checks it too.

use std::path::Path;

/// Name of the task; `SpacedriveDaemon` is the name before the rename, still removed on uninstall.
pub const TASK_NAME: &str = "WingDriveDaemon";

/// Escapes text for an XML element or attribute value.
///
/// A folder such as `C:\Users\Ana & Bia\WingDrive` made the whole definition invalid before.
fn xml_escape(value: &str) -> String {
	let mut escaped = String::with_capacity(value.len());
	for character in value.chars() {
		match character {
			'&' => escaped.push_str("&amp;"),
			'<' => escaped.push_str("&lt;"),
			'>' => escaped.push_str("&gt;"),
			'"' => escaped.push_str("&quot;"),
			'\'' => escaped.push_str("&apos;"),
			_ => escaped.push(character),
		}
	}
	escaped
}

/// Quotes one argument so the C runtime's `CommandLineToArgvW` rules give it back unchanged.
///
/// Backslashes count only before a quote: `C:\data\` written as `"C:\data\"` would end with an
/// escaped quote and swallow the rest of the line, so trailing backslashes are doubled.
fn windows_argument(value: &str) -> String {
	let mut quoted = String::from("\"");
	let mut backslashes = 0;
	for character in value.chars() {
		match character {
			'\\' => backslashes += 1,
			'"' => {
				quoted.extend(std::iter::repeat_n('\\', backslashes * 2 + 1));
				quoted.push('"');
				backslashes = 0;
			}
			_ => {
				quoted.extend(std::iter::repeat_n('\\', backslashes));
				quoted.push(character);
				backslashes = 0;
			}
		}
	}
	quoted.extend(std::iter::repeat_n('\\', backslashes * 2));
	quoted.push('"');
	quoted
}

/// The task definition that runs `daemon_path --data-dir data_dir` when the user logs in.
pub fn task_xml(daemon_path: &Path, data_dir: &Path) -> String {
	let command = xml_escape(&windows_argument(&daemon_path.to_string_lossy()));
	let arguments = xml_escape(&format!(
		"--data-dir {}",
		windows_argument(&data_dir.to_string_lossy())
	));
	format!(
		r#"<?xml version="1.0" encoding="UTF-16"?>
<Task version="1.2" xmlns="http://schemas.microsoft.com/windows/2004/02/mit/task">
  <RegistrationInfo>
    <Description>WingDrive Daemon Background Service</Description>
  </RegistrationInfo>
  <Triggers>
    <LogonTrigger>
      <Enabled>true</Enabled>
    </LogonTrigger>
  </Triggers>
  <Principals>
    <Principal>
      <LogonType>InteractiveToken</LogonType>
      <RunLevel>LeastPrivilege</RunLevel>
    </Principal>
  </Principals>
  <Settings>
    <MultipleInstancesPolicy>IgnoreNew</MultipleInstancesPolicy>
    <DisallowStartIfOnBatteries>false</DisallowStartIfOnBatteries>
    <StopIfGoingOnBatteries>false</StopIfGoingOnBatteries>
    <AllowHardTerminate>true</AllowHardTerminate>
    <StartWhenAvailable>true</StartWhenAvailable>
    <RunOnlyIfNetworkAvailable>false</RunOnlyIfNetworkAvailable>
    <AllowStartOnDemand>true</AllowStartOnDemand>
    <Enabled>true</Enabled>
    <Hidden>false</Hidden>
    <RunOnlyIfIdle>false</RunOnlyIfIdle>
    <WakeToRun>false</WakeToRun>
    <ExecutionTimeLimit>PT0S</ExecutionTimeLimit>
    <Priority>7</Priority>
  </Settings>
  <Actions>
    <Exec>
      <Command>{command}</Command>
      <Arguments>{arguments}</Arguments>
    </Exec>
  </Actions>
</Task>
"#
	)
}

/// Encodes the definition as UTF-16LE with a byte order mark.
///
/// The XML declares UTF-16, which is what Task Scheduler writes and expects. The file used to be
/// written as UTF-8 under that declaration, which an XML parser rejects.
pub fn task_file_bytes(xml: &str) -> Vec<u8> {
	let mut bytes = Vec::with_capacity(2 + xml.len() * 2);
	bytes.extend_from_slice(&[0xFF, 0xFE]);
	for unit in xml.encode_utf16() {
		bytes.extend_from_slice(&unit.to_le_bytes());
	}
	bytes
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::path::PathBuf;

	#[test]
	fn xml_special_characters_are_escaped() {
		assert_eq!(
			xml_escape(r#"a&b<c>d"e'f"#),
			"a&amp;b&lt;c&gt;d&quot;e&apos;f"
		);
	}

	#[test]
	fn arguments_survive_windows_command_line_parsing() {
		assert_eq!(
			windows_argument(r"C:\Program Files\WingDrive"),
			r#""C:\Program Files\WingDrive""#
		);
		// A trailing backslash must not escape the closing quote.
		assert_eq!(windows_argument(r"C:\data\"), r#""C:\data\\""#);
		// Embedded quotes are escaped, with the backslashes before them doubled.
		assert_eq!(windows_argument(r#"a\"b"#), r#""a\\\"b""#);
		assert_eq!(windows_argument("plain"), r#""plain""#);
	}

	#[test]
	fn task_runs_the_daemon_on_the_data_folder() {
		let xml = task_xml(
			&PathBuf::from(r"C:\Program Files\WingDrive\wing-daemon.exe"),
			&PathBuf::from(r"C:\Users\Ana & Bia\AppData\Roaming\WingDrive"),
		);
		assert!(xml.contains(
			r#"<Command>&quot;C:\Program Files\WingDrive\wing-daemon.exe&quot;</Command>"#
		));
		assert!(xml.contains(
			r#"<Arguments>--data-dir &quot;C:\Users\Ana &amp; Bia\AppData\Roaming\WingDrive&quot;</Arguments>"#
		));
		assert!(xml.contains("<LogonTrigger>"));
		// No raw ampersand is left: every one starts an entity.
		assert!(!xml.contains("& "));
	}

	#[test]
	fn task_file_is_utf16_with_a_byte_order_mark() {
		let bytes = task_file_bytes("<?xml é?>");
		assert_eq!(&bytes[..2], &[0xFF, 0xFE]);
		let (pairs, rest) = bytes[2..].as_chunks::<2>();
		assert!(rest.is_empty());
		let units: Vec<u16> = pairs.iter().map(|pair| u16::from_le_bytes(*pair)).collect();
		assert_eq!(String::from_utf16(&units).unwrap(), "<?xml é?>");
	}
}
