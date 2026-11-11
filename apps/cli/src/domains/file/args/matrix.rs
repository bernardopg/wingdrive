use super::*;
use clap::Parser;
#[derive(Parser)]
struct Cli {
	#[command(flatten)]
	args: FileCopyArgs,
}
macro_rules! case {
	($name:ident, $method:literal, $variant:ident, $overwrite:literal, $verify:literal, $move:literal, $dry:literal) => {
		#[test]
		fn $name() {
			let mut args = vec![
				"wing",
				"source",
				"--destination",
				"dest",
				"--method",
				$method,
			];
			if $overwrite {
				args.push("--overwrite");
			}
			if $verify {
				args.push("--verify-checksum");
			}
			if $move {
				args.push("--move-files");
			}
			if $dry {
				args.push("--dry-run");
			}
			let parsed = Cli::try_parse_from(args).unwrap().args;
			assert_eq!(parsed.dry_run, $dry);
			let input: FileCopyInput = parsed.into();
			assert_eq!(input.copy_method, CopyMethod::$variant);
			assert_eq!(input.overwrite, $overwrite);
			assert_eq!(input.verify_checksum, $verify);
			assert_eq!(input.move_files, $move);
			assert!(input.preserve_timestamps);
			assert!(input.validate().is_ok());
		}
	};
}

case!(copy_auto_0000, "auto", Auto, false, false, false, false);
case!(copy_auto_0001, "auto", Auto, false, false, false, true);
case!(copy_auto_0010, "auto", Auto, false, false, true, false);
case!(copy_auto_0011, "auto", Auto, false, false, true, true);
case!(copy_auto_0100, "auto", Auto, false, true, false, false);
case!(copy_auto_0101, "auto", Auto, false, true, false, true);
case!(copy_auto_0110, "auto", Auto, false, true, true, false);
case!(copy_auto_0111, "auto", Auto, false, true, true, true);
case!(copy_auto_1000, "auto", Auto, true, false, false, false);
case!(copy_auto_1001, "auto", Auto, true, false, false, true);
case!(copy_auto_1010, "auto", Auto, true, false, true, false);
case!(copy_auto_1011, "auto", Auto, true, false, true, true);
case!(copy_auto_1100, "auto", Auto, true, true, false, false);
case!(copy_auto_1101, "auto", Auto, true, true, false, true);
case!(copy_auto_1110, "auto", Auto, true, true, true, false);
case!(copy_auto_1111, "auto", Auto, true, true, true, true);
case!(
	copy_atomic_0000,
	"atomic",
	Atomic,
	false,
	false,
	false,
	false
);
case!(
	copy_atomic_0001,
	"atomic",
	Atomic,
	false,
	false,
	false,
	true
);
case!(
	copy_atomic_0010,
	"atomic",
	Atomic,
	false,
	false,
	true,
	false
);
case!(copy_atomic_0011, "atomic", Atomic, false, false, true, true);
case!(
	copy_atomic_0100,
	"atomic",
	Atomic,
	false,
	true,
	false,
	false
);
case!(copy_atomic_0101, "atomic", Atomic, false, true, false, true);
case!(copy_atomic_0110, "atomic", Atomic, false, true, true, false);
case!(copy_atomic_0111, "atomic", Atomic, false, true, true, true);
case!(
	copy_atomic_1000,
	"atomic",
	Atomic,
	true,
	false,
	false,
	false
);
case!(copy_atomic_1001, "atomic", Atomic, true, false, false, true);
case!(copy_atomic_1010, "atomic", Atomic, true, false, true, false);
case!(copy_atomic_1011, "atomic", Atomic, true, false, true, true);
case!(copy_atomic_1100, "atomic", Atomic, true, true, false, false);
case!(copy_atomic_1101, "atomic", Atomic, true, true, false, true);
case!(copy_atomic_1110, "atomic", Atomic, true, true, true, false);
case!(copy_atomic_1111, "atomic", Atomic, true, true, true, true);
case!(
	copy_streaming_0000,
	"streaming",
	Streaming,
	false,
	false,
	false,
	false
);
case!(
	copy_streaming_0001,
	"streaming",
	Streaming,
	false,
	false,
	false,
	true
);
case!(
	copy_streaming_0010,
	"streaming",
	Streaming,
	false,
	false,
	true,
	false
);
case!(
	copy_streaming_0011,
	"streaming",
	Streaming,
	false,
	false,
	true,
	true
);
case!(
	copy_streaming_0100,
	"streaming",
	Streaming,
	false,
	true,
	false,
	false
);
case!(
	copy_streaming_0101,
	"streaming",
	Streaming,
	false,
	true,
	false,
	true
);
case!(
	copy_streaming_0110,
	"streaming",
	Streaming,
	false,
	true,
	true,
	false
);
case!(
	copy_streaming_0111,
	"streaming",
	Streaming,
	false,
	true,
	true,
	true
);
case!(
	copy_streaming_1000,
	"streaming",
	Streaming,
	true,
	false,
	false,
	false
);
case!(
	copy_streaming_1001,
	"streaming",
	Streaming,
	true,
	false,
	false,
	true
);
case!(
	copy_streaming_1010,
	"streaming",
	Streaming,
	true,
	false,
	true,
	false
);
case!(
	copy_streaming_1011,
	"streaming",
	Streaming,
	true,
	false,
	true,
	true
);
case!(
	copy_streaming_1100,
	"streaming",
	Streaming,
	true,
	true,
	false,
	false
);
case!(
	copy_streaming_1101,
	"streaming",
	Streaming,
	true,
	true,
	false,
	true
);
case!(
	copy_streaming_1110,
	"streaming",
	Streaming,
	true,
	true,
	true,
	false
);
case!(
	copy_streaming_1111,
	"streaming",
	Streaming,
	true,
	true,
	true,
	true
);
