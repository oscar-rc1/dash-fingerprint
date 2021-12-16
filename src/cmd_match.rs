use anyhow::Result;
use clap::ArgMatches;
use colored::Colorize;
use nalgebra::DVector;
use rayon::prelude::*;
use std::fs::File;

use crate::{pdtw, FingerprintDb, FINGERPRINT_DB_PATH};

pub fn match_fingerprints(matches: &ArgMatches) -> Result<()> {
	// Load queries

	let queries =
		matches.values_of("fingerprint")
			.unwrap()
			.map(|x| {
				let file = File::open(x)?;
				let query : DVector<f64> = ciborium::de::from_reader(file)?;

				Ok((x, query))
			})
			.collect::<Result<Vec<_>>>()?;

	// Load database

	let db_file = File::open(FINGERPRINT_DB_PATH)?;
	let database : FingerprintDb = ciborium::de::from_reader(db_file)?;

	// Run DTW

	let mut matches_ok = 0;
	let mut matches_fail = 0;

	for q in queries {
		let mut distances =
			database.par_iter()
				.map(|(name, streams)| {
					let mut streams =
						streams.par_iter()
							.map(|(resolution, template)| {
								(resolution, pdtw::partial_dtw(&q.1, &template))
							})
							.collect::<Vec<_>>();

					streams.sort_unstable_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

					(name, streams[0].1, streams)
				})
				.collect::<Vec<_>>();

		distances.sort_unstable_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

		if !matches.is_present("verify") {
			println!("- Query: {}", q.0);

			for (i, d) in distances[0..5].iter().enumerate() {
				println!("\t{}) {} - {}", i + 1, d.0, d.1);

				if matches.is_present("verbose") {
					for (r, d) in &d.2 {
						println!("\t\t- {}: {}", r, d);
					}
				}
			}
		} else {
			let best_match = &distances[0];

			if q.0 == best_match.0 || q.0.ends_with(&format!("/{}", best_match.0)) {
				matches_ok += 1;
				println!("- {} : {}", q.0, "Ok".green().bold());
			} else {
				matches_fail += 1;
				println!("- {} : {}", q.0, "Fail".bright_red().bold());

				if matches.is_present("verbose") {
					println!("\t- Got {} with distance {}", best_match.0, best_match.1);
				}
			}
		}
	}

	if matches.is_present("verify") {
		println!("\n{}  {}",
			format!("\u{2714} {}", matches_ok).green().bold(),
			format!("\u{274C} {}", matches_fail).bright_red().bold()
		);
	}

	Ok(())
}
