use anyhow::{Context, Result};
use clap::{App, Arg};
use colored::Colorize;
use std::{fs::File, io::Write, path::Path};

const DASH_HOME : &str = "videos/dash";

fn main() {
	let matches = App::new("fingerprint-video")
		.version("0.1.0")
		.arg(Arg::from_usage("[video]... 'List of videos to be fingerprinted'"))
		.get_matches();

	let videos = match matches.occurrences_of("video") {
		0 => enumerate_videos().unwrap(),
		_ => matches.values_of("video").unwrap().map(|x| x.to_string()).collect(),
	};

	for v in videos {
		println!("- Processing {}", v);

		if let Err(e) = process_video(&v) {
			eprintln!("\t{} {:?}", "error:".bright_red().bold(), e);
			std::process::exit(1);
		}
	}
}

fn enumerate_videos() -> Result<Vec<String>> {
	let mut result = vec![];

	for entry in std::fs::read_dir(DASH_HOME)? {
		result.push(entry?.path().to_str().unwrap().to_string());
	}

	Ok(result)
}

fn process_video(path: &String) -> Result<()> {
	let name = Path::new(path).file_name().unwrap().to_str().unwrap();
	let output_dir = format!("{}/../../fingerprints", path);

	let _ = std::fs::create_dir_all(&output_dir);

	for sub_entry in std::fs::read_dir(path)? {
		let sub_entry = sub_entry?;
		let resolution = sub_entry.file_name().into_string().unwrap();

		if !sub_entry.metadata()?.is_dir() || resolution == "aac" {
			continue;
		}

		let vec_r = fingerprint_stream(&path, &resolution)?;
		let mut csv_file = File::create(format!("{}/{}_{}.csv", output_dir, name, resolution))?;

		for r in vec_r {
			write!(&mut csv_file, "{}\n", 1.0 / (1.0 + (-r).exp()))?;
		}
	}

	Ok(())
}

fn fingerprint_stream(path: &str, resolution: &str) -> Result<Vec<f64>> {
	let mut i = 1;
	let mut a_last = 0;
	let mut vec_r = vec![0f64];

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
			vec_r.push(((a - a_last) as f64) / (a_last as f64));
		}

		i += 1;
		a_last = a;
	}

	Ok(vec_r)
}
