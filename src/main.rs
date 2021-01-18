
use image;
use image::{Luma, GrayImage};
use imageproc; // For fast integral image.
use plotters::prelude::*;
use rand::random;
use std::env::args;
use std::collections::{HashSet, HashMap};

mod point;

use point::Point;

fn main() {
	// Process CLI.
	let arguments:Vec<String> = args().collect::<Vec<String>>();
	if arguments.len() != 3 {
		println!("Usage: {} <input filename> <output filename>", &arguments[0]);
		return;
	}
	let input_filename = &arguments[1];
	let output_filename = &arguments[2];
	let gray_levels = 8;

	// Load image.
	println!("Loading image.");
	let mut img = image::open(input_filename);
	if img.is_err() {
		println!("Failed to open {}", input_filename);
		return;
	}
	let mut img = img.unwrap().to_luma8();
	adjust_levels(&mut img, gray_levels);
	//let integral:imageproc::definitions::Image<image::Luma<u8>> = imageproc::integral_image::integral_image(&img);

	// Start with a single horizontal line.
	let mut lines:Vec<Point> = vec![(0f32, img.height() as f32/2f32).into(), (img.width() as f32, img.height() as f32/2f32).into()];
	for pass in 0..gray_levels {
		let mut next_lines = vec![];
		// For each line segment in the iteration...
		for p_idx in 0..lines.len()-1 {
			let line_start = lines[p_idx] as Point;
			let line_end = lines[p_idx+1] as Point;

			// Find how much area this line covers.  This means finding the bounding box.
			// We want a square region, so find the midpoint of the line.
			let midpoint = (line_start + line_end) * 0.5f32;
			let dxdy = (line_end - line_start);
			let radius = (dxdy.x*dxdy.x + dxdy.y*dxdy.y).sqrt();
			let topleft = midpoint - Point::new(radius, radius);
			let bottomright = midpoint + Point::new(radius, radius);

			// Find the average luminance of this area, assuming it's big enough.
			//let luma = imageproc::integral_image::sum_image_pixels(&integral, topleft.x as u32, topleft.y as u32, bottomright.x as u32, bottomright.y as u32);
			let mut luminance = 0u32;
			if radius*radius > 1f32 {
				for y in topleft.y as u32..bottomright.y as u32 {
					for x in topleft.x as u32..bottomright.x as u32 {
						if x < 0 || x >= img.width() || y < 0 || y >= img.height() {
							continue;
						}
						let pxl = img.get_pixel(x, y);
						luminance += pxl[0] as u32;
					}
				}
				luminance /= ((2f32*radius)*(2f32*radius)) as u32; // Average luminance.
			}
			println!("Luminance: {}", &luminance);
			if luminance < (gray_levels - pass).into() && radius*radius > 1f32 {
				next_lines.extend(tessellate(line_start, line_end));
			} else {
				next_lines.push(line_start);
			}
		}
		next_lines.push(lines.last().unwrap().clone());
		lines = next_lines;
	}

	// Convert 'lines' to points.
	let points:Vec<(f32, f32)> = lines.iter().map(|&p| { p.into() }).collect();

	// Write output!
	println!("Saving output.");
	draw_image(points, output_filename, img.width(), img.height());

	println!("Saved output to {}", output_filename);
}

fn adjust_levels(img:&mut GrayImage, steps:u8) {
	// Crush the image luminance from 0-255 to `steps` distinct values from 0 to `steps`.
	img.enumerate_pixels_mut().for_each(|(_px, _py, value)| {
		// Bit-crushing.
		let floatval:f32 = value[0] as f32 / 255f32;
		*value = Luma([(floatval*steps as f32) as u8]);
	});
}

fn tessellate(line_start:Point, line_end:Point) -> Vec<Point> {
	tessellate_bolt(line_start, line_end)
}

fn tessellate_bolt(line_start:Point, line_end:Point) -> Vec<Point> {
	// Replace ----
	// With/\
	//    /  \
	//        \/
	// One segment becomes four of 1/4th size.  We could also do three with different sizes.
	let dpos = line_end - line_start;
	let normal = Point::new(-dpos.y, dpos.x)*0.25f32; // Left-hand normal.
	let pa = line_start;
	let pb = line_start + dpos*0.25f32 + normal;
	let pc = line_start + dpos*0.5f32;
	let pd = line_start + dpos*0.75f32 - normal;
	vec![pa, pb, pc, pd] // OMIT LINE END!
}

fn tessellate_hilbert(line_start:Point, line_end:Point) -> Vec<Point> {
	// Replace ----
	// With
	// +-+ +-+
	// | +-+ |
	// +-+ +-+
	// --+ +--
	// One segment becomes four of 1/4th size.  We could also do three with different sizes.
	let dpos = line_end - line_start;
	let normal = Point::new(-dpos.y, dpos.x)*0.25f32; // Left-hand normal.
	let pa = line_start;
	let pb = line_start + dpos*0.25f32 + normal;
	let pc = line_start + dpos*0.5f32;
	let pd = line_start + dpos*0.75f32 - normal;
	vec![pa, pb, pc, pd] // OMIT LINE END!
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