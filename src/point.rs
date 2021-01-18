use std::ops;

#[derive(Copy, Clone, Debug, Default)]
pub struct Point {
	pub x: f32,
	pub y: f32,
}

impl Point {
	pub fn new(x:f32, y:f32) -> Self {
		Point {
			x, y
		}
	}
}

impl PartialEq for Point {
	fn eq(&self, other: &Self) -> bool {
		(self.x - other.x).abs() + (self.y - other.y).abs() < 1e-8f32
	}
}

impl From<(f32, f32)> for Point {
	fn from(p: (f32, f32)) -> Self {
		Point {
			x: p.0, y: p.1
		}
	}
}

impl From<Point> for (f32, f32) {
	fn from(p: Point) -> Self {
		(p.x, p.y)
	}
}

impl ops::Add<Point> for Point {
	type Output = Point;

	fn add(self, rhs: Point) -> Point {
		Point {
			x: self.x + rhs.x,
			y: self.y + rhs.y,
		}
	}
}

impl ops::Sub<Point> for Point {
	type Output = Point;

	fn sub(self, rhs: Point) -> Point {
		Point {
			x: self.x-rhs.x,
			y: self.y-rhs.y,
		}
	}
}

impl ops::Mul<f32> for Point {
	type Output = Point;
	fn mul(self, rhs: f32) -> Point {
		Point {
			x: self.x*rhs,
			y: self.y*rhs,
		}
	}
}

impl ops::Mul<Point> for f32 {
	type Output = Point;
	fn mul(self, rhs: Point) -> Point {
		Point {
			x: self*rhs.x,
			y: self*rhs.y,
		}
	}
}