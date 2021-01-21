
use image;
use image::{Luma, GrayImage};
use imageproc; // For fast integral image.
use plotters::prelude::*;
use rand::random;
use std::collections::{HashSet, HashMap};
use std::env::args;
use std::io::Write;
use std::fs::File;

mod hilbert;
mod point;

use point::Point;

fn main() {
	// Process CLI.
	let arguments:Vec<String> = args().collect::<Vec<String>>();
	if arguments.len() < 3 {
		println!("Usage: {} <input filename> <output filename>", &arguments[0]);
		return;
	}
	let input_filename = &arguments[1];
	let output_filename = &arguments[2];
	let gray_levels = if arguments.len() < 4 {
		10
	} else {
		arguments[3].parse::<u8>().unwrap()
	};

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

	let mut hilbert_curve = hilbert::Hilbert::new(img.width(), 0, 0, img.height(), None);
	hilbert_curve.subdivide();
	for y in 0..img.height() {
		for x in 0..img.width() {
			let luma = img.get_pixel(x, y)[0];
			hilbert_curve.subdivide_leaf(x, y, (gray_levels - luma) as u32);
		}
	}
	let mut lines:Vec<(f32, f32)> = hilbert_curve.rasterize();

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
	if line_end == line_start {
		vec![line_start]
	} else {
		tessellate_bolt(line_start, line_end)
	}
}

fn tessellate_bolt(line_start:Point, line_end:Point) -> Vec<Point> {
	// Replace ----
	// With/\
	//    /  \
	//        \/
	// One segment becomes four of 1/4th size.  We could also do three with different sizes.
	let dpos = line_end - line_start;
	let left = Point::new(-dpos.y, dpos.x) * 0.75; // Lob-sided Left-hand normal.
	let right = Point::new(dpos.y, -dpos.x) * 0.25f32;
	let fwd = dpos * 0.5f32;
	vec![
		line_start,
		line_start + fwd + left,
		line_start + fwd,
		line_start + fwd + right,
		line_end
	]
}

fn tessellate_hex(line_start:Point, line_end:Point) -> Vec<Point> {
	//  e f
	// a d g
	//  b c
	let dpos = line_end - line_start;
	let left_normal = Point::new(-dpos.y, dpos.x)*0.3f32;
	let forward = dpos*0.3;
	let right_normal = Point::new(dpos.y, -dpos.x)*0.3f32;
	vec![
		line_start,
		line_start + forward + right_normal,
		line_start + forward + right_normal + forward,
		line_start + forward + forward,
		line_start + forward + left_normal,
		line_start + forward + left_normal + forward,
		line_end
	]
}

fn tessellate_square(line_start:Point, line_end:Point) -> Vec<Point> {
	// Replace
	// ----
	// With
	// bc
	// adg
	//  ef
	let dpos = line_end - line_start;
	let left_normal = Point::new(-dpos.y, dpos.x)*0.5f32;
	let forward = dpos*0.5;
	let right_normal = Point::new(dpos.y, -dpos.x)*0.5f32;
	vec![
		line_start,
		line_start + left_normal,
		line_start + left_normal + forward,
		line_start + forward,
		line_start + right_normal + forward,
		line_start + right_normal + forward + forward,
		line_end
	]
}

fn tessellate_tee(line_start:Point, line_end:Point) -> Vec<Point> {
	// g     h
	// f e j i
	// a x x m
	//   b l
	let dpos = line_end - line_start;
	let a = line_start;
	let l = Point::new(-dpos.y, dpos.x)*0.25f32;
	let f = dpos*0.25;
	let r = Point::new(dpos.y, -dpos.x)*0.25f32;
	vec![
		a,
		a + f + r,
		//a + f,
		a + f + l,
		a + l,
		a + l + l,
		a + l + l + f + f + f,
		a + l + f + f + f,
		a + l + f + f,
		//a + f + f,
		a + r + f + f,
		a + f + f + f
	]
}

fn tessellate_w(line_start:Point, line_end:Point) -> Vec<Point> {
	//   c
	// a   e
	//  b d
	let dpos = line_end - line_start;
	let a = line_start;
	let l = Point::new(-dpos.y, dpos.x)*0.4f32;
	let f = dpos*0.2;
	let r = Point::new(dpos.y, -dpos.x)*0.2f32;
	vec![
		a,
		a + f + r,
		a + f + f + l,
		a + f + f + f + r,
		line_end
	]
}

fn tessellate_fake_hilbert(line_start:Point, line_end:Point) -> Vec<Point> {
	// e fi j
	// dcghlk
	// ab  mn
	let dpos = line_end - line_start;
	let a = line_start;
	let l = Point::new(-dpos.y, dpos.x)*0.25f32;
	let f = dpos*0.15;
	vec![
		a,
		a + f,
		a + f + l,
		a + l,
		a + l + l,
		a + l + l + f + f,
		a + l + f + f,
		a + l + f + f + f,
		a + l + l + f + f + f,
		a + l + l + f + f + f + f + f,
		a + l + f + f + f + f + f,
		a + l + f + f + f + f,
		a + f + f + f + f,
		line_end
	]
}


fn draw_image(points:Vec<(f32, f32)>, filename:&str, canvas_width:u32, canvas_height:u32) -> Result<(), Box<dyn std::error::Error>> {
	let mut backend = SVGBackend::new(filename, (canvas_width, canvas_height));
	//chart.draw_series(LineSeries::new(vec![(0.0, 0.0), (5.0, 5.0), (8.0, 7.0)],&RED,))?;
	for i in 0..points.len()-1 {
		backend.draw_line((points[i].0 as i32, points[i].1 as i32), (points[i+1].0 as i32, points[i+1].1 as i32), &BLACK);
		//backend.draw_circle((points[i].0 as i32, points[i].1 as i32), 1u32, &BLACK, false);
	}
	//backend.draw_rect((50, 50), (200, 150), &RED, true)?;

	let mut fout = File::create(std::path::Path::new(&("raw_".to_owned() + &filename.to_owned()))).unwrap();
	points.iter().for_each(|&p|{
		fout.write(format!("{},{}\n", p.0, p.1).as_ref());
	});

	Ok(())
}