use anyhow::Result;
use clap::ArgMatches;
use nalgebra::DVector;
use rayon::prelude::*;
use std::{ffi::OsString, fs::File, io::{BufRead, BufReader}};

use crate::pdtw;

const FINGERPRINT_HOME : &str = "videos/fingerprints";

pub fn match_fingerprints(matches: &ArgMatches) -> Result<()> {
	// Load queries

	let queries =
		matches.values_of_os("fingerprint")
			.unwrap()
			.map(|x| {
				let name = x.to_os_string();
				let vector = load_csv(&name)?;
				Ok((name, vector))
			})
			.collect::<Result<Vec<_>>>()?;

	// Load database

	let database =
		std::fs::read_dir(FINGERPRINT_HOME)?
			.map(|x| {
				let path = x?.path();
				let name = path.file_stem().map(|x| x.to_os_string());
				let ext = path.extension().map(|x| x.to_os_string());

				Ok((ext, name, path.into_os_string()))
			})
			.filter(|x| {
				if let Ok((Some(ext), Some(_), _)) = &x {
					ext.to_ascii_lowercase() == "csv"
				} else {
					false
				}
			})
			.map(|x| {
				match x {
					Ok(x) => Ok((x.1.unwrap(), load_csv(&x.2)?)),
					Err(e) => Err(e),
				}
			})
			.collect::<Result<Vec<_>>>()?;

	// Run DTW

	for q in queries {
		let mut distances =
			database.par_iter()
				.map(|(name, template)| {
					(name, pdtw::partial_dtw(&q.1, &template))
				})
				.collect::<Vec<_>>();

		distances.sort_unstable_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

		println!("- Query: {:?}", q.0);

		for (i, d) in distances[0..5].iter().enumerate() {
			println!("\t{}) {:?} - {}", i + 1, d.0, d.1);
		}
	}

	Ok(())
}

fn load_csv(path: &OsString) -> Result<DVector<f64>> {
	let reader = BufReader::new(File::open(path)?);

	let values =
		reader.lines()
			.map(|x| Ok(x?.parse::<f64>()?))
			.collect::<Result<Vec<_>>>()?;

	Ok(values.into())
}
