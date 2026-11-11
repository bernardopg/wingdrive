use super::*;
use clap::Parser;
#[derive(Parser)]
struct Cli {
	#[command(flatten)]
	args: TagSearchArgs,
}
macro_rules! case {
	($name:ident, $namespace:literal, $limit:literal, $archived:literal) => {
		#[test]
		fn $name() {
			let mut args = vec![
				"wing",
				"query",
				"--namespace",
				$namespace,
				"--limit",
				$limit,
			];
			if $archived {
				args.push("--include-archived");
			}
			let input: SearchTagsInput = Cli::try_parse_from(args).unwrap().args.into();
			assert_eq!(input.query, "query");
			assert_eq!(input.namespace.as_deref(), Some($namespace));
			assert_eq!(input.limit, Some($limit.parse::<usize>().unwrap()));
			assert_eq!(input.include_archived, Some($archived));
			assert_eq!(input.resolve_ambiguous, Some(false));
		}
	};
}

case!(tag_work_0_0, "work", "0", false);
case!(tag_work_0_1, "work", "0", true);
case!(tag_work_1_0, "work", "1", false);
case!(tag_work_1_1, "work", "1", true);
case!(tag_work_50_0, "work", "50", false);
case!(tag_work_50_1, "work", "50", true);
case!(tag_work_1000_0, "work", "1000", false);
case!(tag_work_1000_1, "work", "1000", true);
case!(tag_personal_0_0, "personal", "0", false);
case!(tag_personal_0_1, "personal", "0", true);
case!(tag_personal_1_0, "personal", "1", false);
case!(tag_personal_1_1, "personal", "1", true);
case!(tag_personal_50_0, "personal", "50", false);
case!(tag_personal_50_1, "personal", "50", true);
case!(tag_personal_1000_0, "personal", "1000", false);
case!(tag_personal_1000_1, "personal", "1000", true);
case!(tag_unicode_0_0, "café", "0", false);
case!(tag_unicode_0_1, "café", "0", true);
case!(tag_unicode_1_0, "café", "1", false);
case!(tag_unicode_1_1, "café", "1", true);
case!(tag_unicode_50_0, "café", "50", false);
case!(tag_unicode_50_1, "café", "50", true);
case!(tag_unicode_1000_0, "café", "1000", false);
case!(tag_unicode_1000_1, "café", "1000", true);
case!(tag_hyphen_0_0, "my-project", "0", false);
case!(tag_hyphen_0_1, "my-project", "0", true);
case!(tag_hyphen_1_0, "my-project", "1", false);
case!(tag_hyphen_1_1, "my-project", "1", true);
case!(tag_hyphen_50_0, "my-project", "50", false);
case!(tag_hyphen_50_1, "my-project", "50", true);
case!(tag_hyphen_1000_0, "my-project", "1000", false);
case!(tag_hyphen_1000_1, "my-project", "1000", true);
