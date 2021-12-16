use nalgebra::{DMatrix, DVector, DVectorSlice};
use rayon::prelude::*;

pub fn partial_dtw(query: &DVector<f64>, template: &DVector<f64>) -> f64 {
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
