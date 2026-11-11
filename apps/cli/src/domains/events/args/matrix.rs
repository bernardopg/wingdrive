use super::*;
use clap::Parser;
#[derive(Parser)]
struct Cli {
	#[command(flatten)]
	args: EventsMonitorArgs,
}
macro_rules! case {
	($name:ident, $format:literal, $variant:ident, $time:literal, $verbose:literal, $pretty:literal) => {
		#[test]
		fn $name() {
			let mut args = vec!["wing", "--format", $format, "--event-type", "JobCompleted"];
			if $time {
				args.push("--timestamps");
			}
			if $verbose {
				args.push("--verbose");
			}
			if $pretty {
				args.push("--pretty");
			}
			let a = Cli::try_parse_from(args).unwrap().args;
			assert!(matches!(a.format, OutputFormat::$variant));
			assert_eq!(a.event_type.as_deref(), Some("JobCompleted"));
			assert_eq!(a.timestamps, $time);
			assert_eq!(a.verbose, $verbose);
			assert_eq!(a.pretty, $pretty);
		}
	};
}

case!(events_human_000, "human", Human, false, false, false);
case!(events_human_001, "human", Human, false, false, true);
case!(events_human_010, "human", Human, false, true, false);
case!(events_human_011, "human", Human, false, true, true);
case!(events_human_100, "human", Human, true, false, false);
case!(events_human_101, "human", Human, true, false, true);
case!(events_human_110, "human", Human, true, true, false);
case!(events_human_111, "human", Human, true, true, true);
case!(events_json_000, "json", Json, false, false, false);
case!(events_json_001, "json", Json, false, false, true);
case!(events_json_010, "json", Json, false, true, false);
case!(events_json_011, "json", Json, false, true, true);
case!(events_json_100, "json", Json, true, false, false);
case!(events_json_101, "json", Json, true, false, true);
case!(events_json_110, "json", Json, true, true, false);
case!(events_json_111, "json", Json, true, true, true);
case!(
	events_json_pretty_000,
	"json-pretty",
	JsonPretty,
	false,
	false,
	false
);
case!(
	events_json_pretty_001,
	"json-pretty",
	JsonPretty,
	false,
	false,
	true
);
case!(
	events_json_pretty_010,
	"json-pretty",
	JsonPretty,
	false,
	true,
	false
);
case!(
	events_json_pretty_011,
	"json-pretty",
	JsonPretty,
	false,
	true,
	true
);
case!(
	events_json_pretty_100,
	"json-pretty",
	JsonPretty,
	true,
	false,
	false
);
case!(
	events_json_pretty_101,
	"json-pretty",
	JsonPretty,
	true,
	false,
	true
);
case!(
	events_json_pretty_110,
	"json-pretty",
	JsonPretty,
	true,
	true,
	false
);
case!(
	events_json_pretty_111,
	"json-pretty",
	JsonPretty,
	true,
	true,
	true
);
