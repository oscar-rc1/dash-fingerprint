use anyhow::Result;
use clap::ArgMatches;
use nalgebra::DVector;
use rayon::prelude::*;
use std::fs::File;

use crate::{pdtw, FingerprintDb, FINGERPRINT_DB_PATH};

pub fn match_fingerprints(matches: &ArgMatches) -> Result<()> {
	// Load queries

	let queries =
		matches.values_of_os("fingerprint")
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

		println!("- Query: {:?}", q.0);

		for (i, d) in distances[0..5].iter().enumerate() {
			println!("\t{}) {} - {}", i + 1, d.0, d.1);

			if matches.is_present("verbose") {
				for (r, d) in &d.2 {
					println!("\t\t- {}: {}", r, d);
				}
			}
		}
	}

	Ok(())
}
