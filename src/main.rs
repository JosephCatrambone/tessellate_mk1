
use image;
use image::{Luma, GrayImage};
use plotters::prelude::*;
use rand::random;
use std::env::args;
use std::collections::{HashSet, HashMap};

fn main() {
	// Load image.
	// Posterize to n levels.
	// Generate dots along a lattice.
	// Remove dots event n steps according to the posterization levels.
	// Find least-cost hamiltonian path from start to end, but in a special way.
	// - Because it's a lattice and TSP doesn't have intersections, we can constrain the outputs from each node to just the immediate neighbors.
	// - If we constrain the path to neighboring nodes and it's on a lattice, _any_ solution that uses all points and satisfies the neighbor solution is a hamiltonian path!
	// Order the points by their visiting order before shipping to 'draw image'.

	// Process CLI.
	let arguments:Vec<String> = args().collect::<Vec<String>>();
	if arguments.len() != 3 {
		println!("Usage: {} <input filename> <output filename>", &arguments[0]);
		return;
	}
	let input_filename = &arguments[1];
	let output_filename = &arguments[2];

	let levels = 8;

	// Load image.
	println!("Loading image.");
	let mut img = image::open(input_filename);
	if img.is_err() {
		println!("Failed to open {}", input_filename);
		return;
	}
	let mut img = img.unwrap().to_luma8();

	// Posterize.
	println!("Posterizing.");
	adjust_levels(&mut img, levels);

	// Generate dots along a lattice.
	println!("Generating lattice.");
	let lattice_density = 3; // 97 is mathematically nice.
	let mut points = generate_lattice(img.width()*lattice_density, img.height()*lattice_density, 1.0/(lattice_density as f32));

	// Filter lattice points by the luminance levels.
	println!("Filtering {} lattice points.", &points.len());
	points = filter_points_by_luminance(&mut points, &img, levels, 1.0f32);

	// Solve Hamiltonian Path
	println!("Solving Hamiltonian Path for {} points.", &points.len());
	// All neighbors are less than 1.5 * lattice density.  Use 1.75 for numerical safety.
	let visit_order = solve_tsp(&points, 1.75f32/(lattice_density as f32));
	points = visit_order.iter().map(|idx|{ points[*idx] }).collect::<Vec<(f32, f32)>>();

	// Write output!
	println!("Saving output.");
	draw_image(points, output_filename, img.width(), img.height());

	println!("Saved output to {}", output_filename);
}

fn adjust_levels(img:&mut GrayImage, steps:u8) {
	img.enumerate_pixels_mut().for_each(|(_px, _py, value)| {
		// Bit-crushing.
		let floatval:f32 = value[0] as f32 / 255f32;
		*value = Luma([(floatval*steps as f32) as u8]);
	});
}

// Generate a triangular lattice with the given dimensions.
// For maximum precision, we should pick some scale such that x points are separated by a value that
// gives us near-integer values for the next offset row.  Three points on the lattice make an equilateral
// triangle, and if one point is at (0, 0) and another is at (a, 0), the third will be at (a/2, a/2 * sqrt(3)).
// 97 is the smallest source of numerical error less than 100.
// Recommend multiplying width and height by 97, then setting scale factor to 1/97.
fn generate_lattice(width:u32, height:u32, scale_factor:f32) -> Vec<(f32, f32)> {
	let mut points = vec![];

	for y in 0..height {
		for x in 0..width {
			let x_offset = if x%2 == 0 {
				0.0f32
			} else {
				0.5f32
			};
			points.push((
				(x as f32 + x_offset)*scale_factor,
				(y as f32)*scale_factor
			));
		}
	}

	points
}

fn filter_points_by_luminance(points: &Vec<(f32, f32)>, img:&GrayImage, levels:u8, white_level:f32) -> Vec<(f32, f32)> {
	// Find the bounds for the points.
	let mut x_min = 1e16f32;
	let mut y_min = 1e16f32;
	let mut x_max = -1e16f32;
	let mut y_max = -1e16f32;
	points.iter().for_each(|&(x, y)| {
		x_min = x_min.min(x);
		x_max = x_max.max(x);
		y_min = y_min.min(y);
		y_max = y_max.max(y);
	});

	// Filter image points.
	let img_width:f32 = img.width() as f32;
	let img_height:f32 = img.height() as f32;

	points.iter().filter(|&&(x, y)|{
		let pixel_x = img_width*((x - x_min)/(x_max-x_min));
		let pixel_y = img_height*((y - y_min)/(y_max-y_min));
		if pixel_y as u32 >= img.height() || pixel_x as u32 >= img.width() {
			return false;
		}
		// If this next thing is true, we add a dark spot.
		// If the pixel is low, it's easier for this to be true.  Dark pixel -> more likely to add dark spot.
		// Increase white level -> more likely for this to be true -> more likely to have dark spot -> darker image.
		// Higher white level -> darker image.  Correct.
		random::<f32>()*white_level > (img.get_pixel(pixel_x as u32, pixel_y as u32)[0] as f32 / levels as f32)
	}).map(|&(x, y)| {
		(x, y) // Clone
	}).collect::<Vec<(f32, f32)>>()
}

fn solve_tsp(points:&Vec<(f32, f32)>, neighbor_distance:f32) -> Vec<usize> {
	// Return the order of the visits for a max-cost hamiltonian path.
	// Since we're on a lattice, all solutions are equally good!  This means that, so long as we keep the lattice constraint, we're golden.

	let mut neighbors = Vec::<Vec<usize>>::with_capacity(points.len());
	let neighbor_distance_sq = neighbor_distance*neighbor_distance;
	let mut parent = HashMap::<usize, usize>::new(); // Idx -> idx.
	let mut max_path = HashMap::<usize, usize>::new();
	let mut candidate_pts = Vec::<usize>::new(); // Possible next indices.
	let mut unvisited = HashSet::<usize>::new();

	// Init list.
	for i in 0..points.len() {
		neighbors.push(vec![]);
		unvisited.insert(i);
	}

	// Expensive O(n^2) conversion to neighbor list.
	for i in 0..points.len() {
		for j in i+1..points.len() {
			let (a_x, a_y) = points[i];
			let (b_x, b_y) = points[j];
			let dx = b_x - a_x;
			let dy = b_y - a_y;
			let dist_sq = dx * dx + dy * dy;
			if dist_sq <= neighbor_distance_sq {
				neighbors[i].push(j);
				neighbors[j].push(i);
			}
		}
	}

	// Pick a random starting point.
	loop {
		let start = random::<usize>() % points.len();
		if neighbors[start].len() < 1usize {
			continue;
		}
		candidate_pts.push(start);
		parent.insert(start, start);
		max_path.insert(start, 0);
		break;
	}

	let mut longest_path_length = 0;
	let mut longest_path_end = 0;

	// Find a path from start to end that maximizes the number of steps.
	while candidate_pts.len() > 0 {
		let current_pt = candidate_pts.pop();
		if current_pt.is_none() {
			continue;
		}
		let current_pt = current_pt.unwrap();
		unvisited.remove(&current_pt);

		// What's the distance to this one?
		let dist = max_path[&parent[&current_pt]] + 1;

		// For each of the children, if they're unvisited, set their parent to this one IF it would make a longer path.
		for nbr in &neighbors[current_pt] {
			// If unvisited...
			if unvisited.contains(&nbr) {
				// Visit them next.
				parent.insert(*nbr, current_pt);
				max_path.insert(*nbr, dist+1); // To prevent others from overwriting it.
				//candidate_pts.insert(0, nbr); // For BFS
				candidate_pts.push(*nbr); // For DFS.

				if dist+1 > longest_path_length {
					longest_path_length = dist+1;
					longest_path_end = *nbr;
				}
			}
		}
	}

	// Trace a path from the end to the start.
	let mut path = vec![];
	while parent[&longest_path_end] != longest_path_end {
		path.push(longest_path_end);
		longest_path_end = parent[&longest_path_end];
	}
	path
}

fn draw_image(points:Vec<(f32, f32)>, filename:&str, canvas_width:u32, canvas_height:u32) -> Result<(), Box<dyn std::error::Error>> {
	let mut backend = SVGBackend::new(filename, (canvas_width, canvas_height));
	//chart.draw_series(LineSeries::new(vec![(0.0, 0.0), (5.0, 5.0), (8.0, 7.0)],&RED,))?;
	for i in 0..points.len()-1 {
		backend.draw_line((points[i].0 as i32, points[i].1 as i32), (points[i+1].0 as i32, points[i+1].1 as i32), &BLACK);
		//backend.draw_circle((points[i].0 as i32, points[i].1 as i32), 1u32, &BLACK, false);
	}
	//backend.draw_rect((50, 50), (200, 150), &RED, true)?;

	Ok(())
}