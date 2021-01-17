
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

	let levels = 6;

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
	let lattice_density = 1; // 97 is mathematically nice.
	let mut points = generate_lattice(img.width()*lattice_density, img.height()*lattice_density, 1.0/(lattice_density as f32));

	// Filter lattice points by the luminance levels.
	println!("Filtering {} lattice points.", &points.len());
	points = filter_points_by_luminance(&mut points, &img, levels, 0.5f32);

	// Solve Hamiltonian Path
	println!("Solving Hamiltonian Path for {} points.", &points.len());
	// All neighbors are less than 2.0 * lattice density.  Use 2.1 for numerical safety.
	let visit_order = solve_tsp(&points, 2.1f32/(lattice_density as f32));
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
		random::<f32>()*white_level > (img.get_pixel(pixel_x as u32, pixel_y as u32)[0] as f32 / levels as f32)
	}).map(|&(x, y)| {
		(x, y) // Clone
	}).collect::<Vec<(f32, f32)>>()
}

fn solve_tsp(points:&Vec<(f32, f32)>, neighbor_distance:f32) -> Vec<usize> {
	// Return the order of the visits for a max-cost hamiltonian path.
	// Since we're on a lattice, all solutions are equally good!  This means that, so long as we keep the lattice constraint, we're golden.
	// We can accomplish this by starting with a single point, extending it to a line by connecting a neighbor, and then growing it into a triangle.
	// For a randomly chosen edge in the set of edges, expand it to include a neighbor shared by the two endpoints.

	let mut neighbors = Vec::<Vec<usize>>::with_capacity(points.len());
	let neighbor_distance_sq = neighbor_distance*neighbor_distance;
	let mut visited_point_set = HashSet::<usize>::with_capacity(points.len());
	let mut boundary_point_list = Vec::<usize>::with_capacity(points.len());
	let mut edges = HashMap::<usize, usize>::new(); // From Idx -> To Idx.  Edges are indexed by starting point.
	let mut current_edge = random::<usize>() % points.len(); // The 'current edge' is defined by the index of the vertex of the start.

	// Init list.
	for _i in 0..points.len() {
		neighbors.push(vec![]);
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
	while neighbors[current_edge].len() < 3 {
		current_edge = random::<usize>() % points.len();
	}

	// Grow the starting point into an actual edge.
	// Pick two neighbor points which are also neighbors of each other.
	let mut nbr_a = current_edge;
	let mut nbr_b = current_edge;
	'outer: loop {
		for candidate_nbr_a in &neighbors[current_edge] {
			for candidate_nbr_b in &neighbors[current_edge] {
				if neighbors[*candidate_nbr_a].contains(candidate_nbr_b) && *candidate_nbr_a != current_edge && *candidate_nbr_b != current_edge && *candidate_nbr_a != *candidate_nbr_b {
					nbr_a = *candidate_nbr_a;
					nbr_b = *candidate_nbr_b;
					break 'outer;
				}
			}
		}
		current_edge = random::<usize>() % points.len();
	}
	assert_ne!(current_edge, nbr_a);
	assert_ne!(current_edge, nbr_b);
	assert_ne!(nbr_a, nbr_b);

	// Make these three neighbors into a loop.
	let starting_edge = current_edge;
	edges.insert(current_edge, nbr_a);
	edges.insert(nbr_a, nbr_b);
	edges.insert(nbr_b, current_edge);
	visited_point_set.insert(current_edge);
	visited_point_set.insert(nbr_a);
	visited_point_set.insert(nbr_b);
	boundary_point_list.push(current_edge);
	boundary_point_list.push(nbr_a);
	boundary_point_list.push(nbr_b);

	// Keep growing the edges.
	while boundary_point_list.len() > 2 && boundary_point_list.len() <= points.len() {
		println!("Edges: {}", &edges.len());
		println!("Boundary points: {}", &boundary_point_list.len());
		println!("Visited points: {}", &visited_point_set.len());

		// TODO: We should keep track of the points on the boundary, since those
		current_edge = boundary_point_list[random::<usize>() % boundary_point_list.len()];
		let edge_start_point = current_edge;
		let edge_end_point = edges[&edge_start_point];

		// Find neighbors of both of these points.  They should share _at most_ two.
		let start_nbrs = &neighbors[edge_start_point];
		let end_nbrs = &neighbors[edge_end_point];

		let mut common_neighbors = vec![];
		for pt in start_nbrs {
			if !visited_point_set.contains(pt) && end_nbrs.contains(pt) {
				common_neighbors.push(pt);
			}
		}

		// Could be we don't have any neighbors.
		if common_neighbors.len() < 1 {
			let index_to_remove = boundary_point_list.iter().position(|p| { *p == edge_start_point }).unwrap();
			boundary_point_list.remove(index_to_remove);
			continue;
		}

		// Add this new point to the boundary!
		let new_pt = common_neighbors[0];
		if let Some(v) = edges.get_mut(&edge_start_point) {
			*v = *new_pt;
		} else {
			edges.insert(edge_start_point, *new_pt);
		}
		edges.insert(*new_pt, edge_end_point);
		boundary_point_list.push(*new_pt);
		visited_point_set.insert(*new_pt);

		// Check these two new edges and, if they can't grow any more, remove them from the boundary point list.
		for p in &[edge_start_point, *new_pt, edge_end_point] {
			let mut edge_can_grow = false;
			let start_nbrs:&Vec<usize> = &neighbors[*p];
			let end_nbrs:&Vec<usize> = &neighbors[edges[p]];
			// If they also share one or more point, this edge can grow.
			for a in start_nbrs {
				if end_nbrs.contains(a) && !visited_point_set.contains(a) {
					edge_can_grow = true;
				}
			}

			if !edge_can_grow {
				if let Some(point_index) = boundary_point_list.iter().position(|p_to_remove| { *p_to_remove == *p }) {
					boundary_point_list.remove(point_index);
				}
			}
		}
	}

	// Start at some random point and move along until the end.
	current_edge = edges[&starting_edge];
	let mut final_path = vec![];
	while current_edge != starting_edge {
		final_path.push(current_edge);
		current_edge = edges[&current_edge];
	}
	final_path
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