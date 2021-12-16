use clap::{App, AppSettings, Arg, SubCommand};
use colored::Colorize;
use nalgebra::DVector;
use std::collections::HashMap;

mod cmd_match;
mod cmd_network;
mod cmd_video;
mod pdtw;

pub type FingerprintDb = HashMap<String, HashMap<String, DVector<f64>>>;
pub const FINGERPRINT_DB_PATH : &str = "videos/fingerprints.bin";

fn main() {
	let matches = App::new("dash-fp")
		.version("0.1.0")
		.setting(AppSettings::SubcommandRequired)
		.subcommand(
			SubCommand::with_name("match")
				.about("Matches network fingerprints against the video database")
				.display_order(1)
				.arg(Arg::from_usage("--verify         'Check if the best result matches the filename'"))
				.arg(Arg::from_usage("-v, --verbose    'Show more details about the matches'"))
				.arg(Arg::from_usage("<fingerprint>... 'Files with network fingerprints'"))
		)
		.subcommand(
			SubCommand::with_name("network")
				.about("Obtains a fingerprint from network traffic")
				.display_order(2)
				.arg(Arg::from_usage("--video [url]               'Opens the given URL in VLC before fingerprinting'"))
				.arg(Arg::from_usage("-n, --num-samples [samples] 'Number of samples to obtain'").default_value("40"))
				.arg(Arg::from_usage("-l, --segment-length [time] 'Segment length, in seconds'").default_value("4"))
				.arg(Arg::from_usage("-e, --epsilon [throughput]  'Minimum data rate, in bytes/s'").default_value("100"))
				.arg(Arg::from_usage("<interface>                 'Network interface to be monitored'"))
				.arg(Arg::from_usage("<output>                    'Path to the output file'"))
		)
		.subcommand(
			SubCommand::with_name("video")
				.about("Obtains a fingerprint from a set of DASH segments")
				.display_order(3)
				.arg(Arg::from_usage("[video] 'Name of the video to be fingerprinted. If not is specified, the command \
				                               will generate the database.'"))
		)
		.get_matches();

	let result = match matches.subcommand() {
		("match",   Some(m)) => cmd_match::match_fingerprints(m),
		("network", Some(m)) => cmd_network::fingerprint_network(m),
		("video",   Some(m)) => cmd_video::fingerprint_videos(m),
		_ => unreachable!(),
	};

	if let Err(e) = result {
		eprintln!("{} {:?}", "error:".bright_red().bold(), e);
		std::process::exit(1);
	}
}
