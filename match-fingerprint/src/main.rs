use anyhow::Result;
use clap::{App, Arg, ArgMatches};
use colored::Colorize;
use nalgebra::{DMatrix, DVector, DVectorSlice};
use rayon::prelude::*;
use std::{ffi::OsString, fs::File, io::{BufRead, BufReader}};

const FINGERPRINT_HOME : &str = "videos/fingerprints";

fn main() {
	let matches = App::new("match-fingerprint")
		.version("0.1.0")
		.arg(Arg::from_usage("<fingerprint>... 'File with the network fingerprint to be matched against the video database'"))
		.get_matches();

	if let Err(e) = match_fingerprints(&matches) {
		eprintln!("{} {:?}", "error:".bright_red().bold(), e);
		std::process::exit(1);
	}
}

fn match_fingerprints(matches: &ArgMatches) -> Result<()> {
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
					(name, partial_dtw(&q.1, &template))
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

fn partial_dtw(query: &DVector<f64>, template: &DVector<f64>) -> f64 {
	let mut min_dist = f64::INFINITY;

	if template.nrows() >= query.nrows() {
		let num_seq = template.nrows() - query.nrows() + 1;

		let dist =
			(0..num_seq).into_par_iter()
				.map(|i| {
					let query_slice = query.rows(0, query.nrows());

					(1..(template.nrows()-i).min(2*query.nrows())).into_par_iter()
						.map(|j| {
							let template_slice = template.rows(i, j);
							partial_dtw_subsequence(&query_slice, &template_slice)
						})
						.reduce(|| f64::INFINITY, |a, b| a.min(b))
				})
				.reduce(|| f64::INFINITY, |a, b| a.min(b));

		if dist < min_dist {
			min_dist = dist;
		}
	}

	min_dist
}

fn partial_dtw_subsequence(query: &DVectorSlice<f64>, template: &DVectorSlice<f64>) -> f64 {
	let n = query.nrows();
	let m = template.nrows();

	let mut grid = DMatrix::zeros(n + 1, m + 1);

	for i in 1..=n {
		grid[(i,0)] = f64::INFINITY;
	}

	for i in 1..=m {
		grid[(0,i)] = f64::INFINITY;
	}

	for i in 1..=n {
		for j in 1..=m {
			let cost = (query[i-1] - template[j-1]).abs();

			if j > 2 {
				grid[(i,j)] = cost + grid[(i-1,j)].min(grid[(i-1,j-1)].min(grid[(i-1,j-2)]));
			} else {
				grid[(i,j)] = cost + grid[(i-1,j)].min(grid[(i-1,j-1)]);
			}
		}
	}

	grid[(n,m)] / (n as f64)
}
