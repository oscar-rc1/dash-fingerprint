use anyhow::{bail, Context, Result};
use clap::{value_t, ArgMatches};
use std::{collections::HashMap, fs::File};

use crate::{FingerprintDb, FINGERPRINT_DB_PATH};

const DASH_PATH   : &str    = "videos/dash";
const RESOLUTIONS : &[&str] = &["480p", "720p", "1080p"];

pub fn fingerprint_videos(matches: &ArgMatches) -> Result<()> {
	if matches.occurrences_of("video") == 0 {
		build_database()
	} else {
		dump_single(value_t!(matches, "video", String)?)
	}
}

fn build_database() -> Result<()> {
	let mut database : FingerprintDb = HashMap::new();

	for entry in std::fs::read_dir(DASH_PATH)? {
		let entry = entry?;
		let path = entry.path().to_str().unwrap().to_string();
		let name = entry.file_name().to_str().unwrap().to_string();

		println!("- Fingerprinting {}", name);

		let fingerprint =
			fingerprint_dash(&path)?
				.into_iter()
				.map(|x| (x.0, x.1.into()))
				.collect::<HashMap<_,_>>();

		database.insert(name, fingerprint);
	}

	if database.len() == 0 {
		bail!("No videos found, nothing to do.");
	}

	let output = File::create(FINGERPRINT_DB_PATH)?;
	ciborium::ser::into_writer(&database, output)?;

	println!("\n- Database written to {}", FINGERPRINT_DB_PATH);
	Ok(())
}

fn dump_single(name: String) -> Result<()> {
	let path = format!("{}/{}", DASH_PATH, name);
	let fp = fingerprint_dash(&path)?;

	for i in 0..fp[0].1.len() {
		for j in 0..fp.len() {
			if j != 0 {
				print!(",");
			}

			print!("{}", fp[j].1[i]);
		}

		print!("\n");
	}

	Ok(())
}

fn fingerprint_dash(path: &str) -> Result<Vec<(String, Vec<f64>)>> {
	let mut result = vec![];
	let mut segment_count = None;

	for r in RESOLUTIONS {
		let fp = fingerprint_stream(path, &r)?;

		if let Some(segment_count) = &segment_count {
			if *segment_count != fp.len() {
				bail!("Segment count mismatch: expected {}, got {}", segment_count, fp.len());
			}
		} else {
			segment_count = Some(fp.len());
		}

		result.push((r.to_string(), fp));
	}

	Ok(result)
}

fn fingerprint_stream(path: &str, resolution: &str) -> Result<Vec<f64>> {
	let mut i = 1;
	let mut a_last = 0;
	let mut vec_r = vec![];

	let path_video = format!("{}/{}", path, resolution);
	let path_audio = format!("{}/aac", path);

	std::fs::metadata(&path_video).context("Missing video stream")?;
	std::fs::metadata(&path_audio).context("Missing audio stream")?;

	loop {
		let vid_meta = std::fs::metadata(format!("{}/segment_{}.m4s", path_video, i));
		let snd_meta = std::fs::metadata(format!("{}/segment_{}.m4s", path_audio, i));

		let a = match (vid_meta, snd_meta) {
			(Ok(vid_meta), Ok(snd_meta)) => (vid_meta.len() + snd_meta.len()) as i64,
			(Ok(vid_meta), Err(_)) => vid_meta.len() as i64,
			(Err(_), Ok(snd_meta)) => snd_meta.len() as i64,
			_ => break,
		};

		if i != 1 {
			let r = ((a - a_last) as f64) / (a_last as f64);
			let r_norm = 1.0 / (1.0 + (-r).exp());

			vec_r.push(r_norm);
		}

		i += 1;
		a_last = a;
	}

	Ok(vec_r)
}
