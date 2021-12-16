use anyhow::{bail, Result};
use clap::{value_t, ArgMatches};
use nalgebra::DVector;
use std::{fs::File, time::Duration};

pub fn fingerprint_network(matches: &ArgMatches) -> Result<()> {
	// Extract and validate parameters

	let output = value_t!(matches, "output", String)?;
	let interface = value_t!(matches, "interface", String)?;
	let num_samples = value_t!(matches, "num-samples", usize)?;
	let segment_length = value_t!(matches, "segment-length", i64)?;
	let epsilon = value_t!(matches, "epsilon", i64)?;

	if std::fs::metadata(format!("/sys/class/net/{}/statistics/rx_bytes", interface)).is_err() {
		bail!("Invalid network interface: {}", interface);
	}

	if num_samples < 1 {
		bail!("The number of samples must be at least 1");
	}

	if segment_length < 1 {
		bail!("The segment length must be at least 1 second");
	}

	if epsilon < 0 {
		bail!("The minimum throughput must be non-negative");
	}

	// Run fingerprint process

	let mut d_tr = DVector::zeros(num_samples);
	let mut rx_last = get_rx_bytes(&interface).unwrap();
	let mut p_last = 0;
	let mut p = 0;
	let mut i = 0;
	let mut t = 0;

	loop {
		let rx = get_rx_bytes(&interface).unwrap();
		let rx_rate = rx - rx_last;

		if rx_rate >= epsilon {
			p += rx_rate;

			if t >= segment_length {
				if i != 0 {
					let d = ((p - p_last) as f64) / (p_last as f64);
					d_tr[i] = 1.0 / (1.0 + (-d).exp());
				}

				p_last = p;
				p = 0;

				t = 0;
				i += 1;

				if i >= num_samples {
					break
				}
			}
		}

		if p > 0 || i > 0 {
			t += 1;
		}

		rx_last = rx;
		print_progress((i as f64) / (num_samples as f64));
		std::thread::sleep(Duration::from_millis(1000));
	}

	print_progress(1.0);
	eprintln!("");

	// Write to file

	let output = File::create(output)?;
	ciborium::ser::into_writer(&d_tr, output)?;

	Ok(())
}

fn get_rx_bytes(iface: &str) -> Result<i64> {
	let count_bytes = std::fs::read(format!("/sys/class/net/{}/statistics/rx_bytes", iface))?;
	let count = String::from_utf8(count_bytes)?;

	Ok(count.trim().parse()?)
}

fn print_progress(progress: f64) {
	let bar_progress = (50.0 * progress) as usize;
	eprint!("  {:>3.0}% [{:<50}]\r", progress * 100.0, "#".repeat(bar_progress));
}
