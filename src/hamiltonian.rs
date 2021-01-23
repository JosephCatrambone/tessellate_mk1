use rand::{thread_rng, Rng, RngCore};

/// Measure the length of the path between the given points.
/// If `order` is None, will measure the length of the points sequentially.
/// If `close` is True, will add the distance between the last point and the start.
pub fn tour_length(points:&Vec<(f32, f32)>, order:Option<&Vec<usize>>, close:bool) -> f32 {
	assert!(points.len() > 1);
	let mut length:f32 = 0f32;
	let mut order = if let Some(ord) = order {
		ord.clone()
	} else {
		(0..points.len()).into_iter().collect()
	};
	if close {
		order.push(*order.first().unwrap());
	}

	for i in 0..order.len()-1 {
		let p = points[order[i]];
		let q = points[order[i+1]];
		let dx = q.0 - p.0;
		let dy = q.1 - p.1;
		length += ((dx*dx) + (dy*dy)).sqrt();
	}

	length
}

pub fn solve_tsp_approx(points:&Vec<(f32, f32)>, max_iterations:u64, verbose:bool) -> Vec<usize> {
	let mutation_odds = 0.01f64;
	let num_paths = 500;
	let mut tours = vec![];
	let mut rng = thread_rng();

	// Make a bunch of candidate tours.
	for _ in 0..num_paths {
		let tour:Vec<usize> = tour_from_unselected(
			points.len(),
			(0..points.len()).into_iter().map(|p|{ rng.next_u64() as usize }).collect()
		);
		tours.push(tour);
	}

	for _ in 0..max_iterations {
		// Calculate the length of each tour and keep the two best.
		let mut best_idx = 0;
		let mut best_length:f32 = tour_length(points, Some(&tours[0]), true);
		let mut second_idx = 1;
		let mut second_length:f32 = tour_length(points, Some(&tours[1]), true);

		for idx in 2..tours.len() {
			let tour_len = tour_length(points, Some(&tours[idx]), true);
			if tour_len < best_length {
				second_idx = best_idx;
				second_length = best_length;
				best_idx = idx;
				best_length = tour_len;
			} else if tour_len < second_length {
				second_length = tour_len;
				second_idx = idx;
			}
		}

		let mut next_tours = vec![];
		next_tours.push(tours[best_idx].clone());
		next_tours.push(tours[second_idx].clone());
		for _ in 0..num_paths-2 {
			next_tours.push(cross_vectors(&tours[best_idx], &tours[second_idx], mutation_odds, points.len()));
		}
		tours = next_tours;

		if verbose {
			println!("Shortest tour: {}", best_length);
		}
	}

	tours[0].clone()
}

/// Perform some random cross between two 'genes' with mutation.
/// Given two vectors...
/// [1, 2, 3, 4, 5]
/// [a, b, c, d, e]
/// Flip a coin to see which parent's base will be used.
/// Possible outputs:
/// [a, 2, 3, d, 5]
/// [1, 2, 3, d, 5]
/// [a, b, c, d, e]
/// [a, b, 3, 4, 5]
/// If mutation_odds is greater than zero, will, with that probability, select a random value to insert, rather than a value from either parent.
fn cross_vectors(p:&Vec<usize>, q:&Vec<usize>, mutation_odds:f64, num_points:usize) -> Vec<usize> {
	let mut res = vec![];
	let mut rng = thread_rng();

	// p and q should be the same size in theory, but...
	for i in 0..p.len().min(q.len()) {
		res.push(
			if rng.gen_bool(mutation_odds) {
				rng.next_u64() as usize % num_points
			} else {
				if rng.gen_bool(0.5f64 - mutation_odds as f64/2f64) {
					p[i]
				} else {
					q[i]
				}
		});
	}

	res
}

/// Build a tour that touches every point from a vec of indices.
/// If we have points a, b, c and get `unselected` = [0, 0, 0], we give back [a, b, c].
/// If we get `unselected` = [2, 1, 0], we give back [c, b, a].
/// Maps each entry in `unselected` to some index%num_points, with the num_points decreasing as more
/// are drawn from the pile.  Will never select more than one visit to the same item.
fn tour_from_unselected(num_points:usize, unselected:Vec<usize>) -> Vec<usize> {
	let mut points:Vec<usize> = (0..num_points).collect();
	let mut ordering = vec![];

	for idx in unselected {
		let next = points.remove(idx%points.len());
		ordering.push(next);
	}

	ordering
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn test_sanity() {
		// Just see if it runs without crashing.
		let pts = vec![(0f32, 0f32), (1f32, 0f32), (0f32, 1f32), (1f32, 1f32)];
		let tour = solve_tsp_approx(&pts, 10, false);
	}

	#[test]
	fn test_round_trip() {
		let pts = vec![(0f32, 0f32), (1f32, 0f32)];
		assert_eq!(tour_length(&pts, None, true), 2f32);
	}

	#[test]
	fn test_one_way_trip() {
		let pts = vec![(0f32, 0f32), (1f32, 0f32)];
		assert_eq!(tour_length(&pts, None, false), 1f32);
	}

	#[test]
	fn test_back_and_forth() {
		let pts = vec![(0f32, 0f32), (1f32, 0f32)];
		assert_eq!(tour_length(&pts, Some(&vec![0usize, 1, 0, 1]), false), 3f32);
	}
}