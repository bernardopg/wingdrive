use super::*;
use clap::Parser;
#[derive(Parser)]
struct Cli {
	#[command(flatten)]
	args: LocationImportArgs,
}
macro_rules! case {
	($name:ident, $path:literal, $new_name:expr, $skip:literal) => {
		#[test]
		fn $name() {
			let mut args = vec!["wing", $path];
			if let Some(name) = $new_name {
				args.extend(["--name", name]);
			}
			if $skip {
				args.push("--skip-existing");
			}
			let input: LocationImportInput = Cli::try_parse_from(args).unwrap().args.into();
			assert_eq!(input.import_path, PathBuf::from($path));
			assert_eq!(input.new_name.as_deref(), $new_name);
			assert_eq!(input.skip_existing, $skip);
		}
	};
}

case!(import_relative_none_0, "dump.sql", None::<&str>, false);
case!(import_relative_none_1, "dump.sql", None::<&str>, true);
case!(import_relative_name_0, "dump.sql", Some("new"), false);
case!(import_relative_name_1, "dump.sql", Some("new"), true);
case!(import_relative_unicode_0, "dump.sql", Some("café"), false);
case!(import_relative_unicode_1, "dump.sql", Some("café"), true);
case!(import_spaces_none_0, "my export.sql", None::<&str>, false);
case!(import_spaces_none_1, "my export.sql", None::<&str>, true);
case!(import_spaces_name_0, "my export.sql", Some("new"), false);
case!(import_spaces_name_1, "my export.sql", Some("new"), true);
case!(
	import_spaces_unicode_0,
	"my export.sql",
	Some("café"),
	false
);
case!(import_spaces_unicode_1, "my export.sql", Some("café"), true);
