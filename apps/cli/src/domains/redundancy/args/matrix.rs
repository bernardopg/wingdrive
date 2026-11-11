use super::*;
use clap::Parser;
#[derive(Parser)]
struct Cli {
	#[command(flatten)]
	args: CompareArgs,
}
macro_rules! case {
	($name:ident, $mode:literal, $variant:ident, $limit:literal, $offset:literal) => {
		#[test]
		fn $name() {
			let a = Cli::try_parse_from([
				"wing",
				"550e8400-e29b-41d4-a716-446655440000",
				"550e8400-e29b-41d4-a716-446655440001",
				"--mode",
				$mode,
				"--limit",
				$limit,
				"--offset",
				$offset,
			])
			.unwrap()
			.args;
			assert!(matches!(a.mode, CompareMode::$variant));
			assert_eq!(a.limit, $limit.parse::<u32>().unwrap());
			assert_eq!(a.offset, $offset.parse::<u32>().unwrap());
			assert_ne!(a.volume_a, a.volume_b);
		}
	};
}

case!(compare_unique_a_0_0, "unique-a", UniqueA, "0", "0");
case!(compare_unique_a_0_25, "unique-a", UniqueA, "0", "25");
case!(compare_unique_a_50_0, "unique-a", UniqueA, "50", "0");
case!(compare_unique_a_50_25, "unique-a", UniqueA, "50", "25");
case!(compare_unique_b_0_0, "unique-b", UniqueB, "0", "0");
case!(compare_unique_b_0_25, "unique-b", UniqueB, "0", "25");
case!(compare_unique_b_50_0, "unique-b", UniqueB, "50", "0");
case!(compare_unique_b_50_25, "unique-b", UniqueB, "50", "25");
case!(compare_shared_0_0, "shared", Shared, "0", "0");
case!(compare_shared_0_25, "shared", Shared, "0", "25");
case!(compare_shared_50_0, "shared", Shared, "50", "0");
case!(compare_shared_50_25, "shared", Shared, "50", "25");
