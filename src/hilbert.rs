
pub struct Hilbert {
	orientation: QuadOrientation,
	leaves: [Option<Box<Hilbert>>; 4], // UL, UR, DL, DR
	left: u32,
	right: u32,
	top: u32,
	bottom: u32,
}

#[derive(Copy, Clone)]
pub enum QuadOrientation {
	A, B, C, D
}

impl Hilbert {
	pub fn new(right:u32, top:u32, left:u32, bottom:u32, starting_orientation:Option<QuadOrientation>) -> Self {
		let orientation = if let Some(o) = starting_orientation {
			o
		} else {
			QuadOrientation::A
		};

		Hilbert {
			orientation,
			leaves: [None, None, None, None],
			left, right, top, bottom
		}
	}

	pub fn get_leaf(&self, x:u32, y:u32) -> &Option<Box<Hilbert>> {
		let mid_x = (self.left + self.right) / 2;
		let mid_y = (self.top + self.bottom) / 2;
		return if y <= mid_y {
			if x <= mid_x {
				&self.leaves[0]
			} else {
				&self.leaves[1]
			}
		} else {
			if x <= mid_x {
				&self.leaves[2]
			} else {
				&self.leaves[3]
			}
		}
	}

	pub fn subdivide_leaf(&mut self, x:u32, y:u32, depth:u32) {
		if depth == 0 {
			return;
		}

		let mid_x = (self.left + self.right) / 2;
		let mid_y = (self.top + self.bottom) / 2;
		let mut leaf = if y <= mid_y {
			if x <= mid_x {
				&mut self.leaves[0]
			} else {
				&mut self.leaves[1]
			}
		} else {
			if x <= mid_x {
				&mut self.leaves[2]
			} else {
				&mut self.leaves[3]
			}
		};
		if let Some(leaf) = leaf {
			leaf.subdivide_leaf(x, y, depth-1);
		} else {
			self.subdivide();
			self.subdivide_leaf(x, y, depth-1);
		}
	}

	pub fn subdivide(&mut self) {
		let mid_x = (self.left + self.right) / 2;
		let mid_y = (self.top + self.bottom) / 2;
		let mut new_orientations = match self.orientation {
			// A:
			// AA
			// DB
			QuadOrientation::A => [
				QuadOrientation::A,
				QuadOrientation::A,
				QuadOrientation::D,
				QuadOrientation::B,
			],
			// B:
			// BC
			// BA
			QuadOrientation::B => [
				QuadOrientation::B,
				QuadOrientation::C,
				QuadOrientation::B,
				QuadOrientation::A,
			],
			// C:
			// DB
			// CC
			QuadOrientation::C => [
				QuadOrientation::D,
				QuadOrientation::B,
				QuadOrientation::C,
				QuadOrientation::C,
			],
			// D:
			// CD
			// AD
			QuadOrientation::D => [
				QuadOrientation::C,
				QuadOrientation::D,
				QuadOrientation::A,
				QuadOrientation::D,
			],
		};

		self.leaves = [
			Some(Box::new(Hilbert::new(mid_x, self.top, self.left, mid_y, Some(new_orientations[0])))), // UL
			Some(Box::new(Hilbert::new(self.right, self.top, mid_x, mid_y, Some(new_orientations[1])))), // UR
			Some(Box::new(Hilbert::new(mid_x, mid_y, self.left, self.bottom, Some(new_orientations[2])))), // DL
			Some(Box::new(Hilbert::new(self.right, mid_y, mid_x, self.bottom, Some(new_orientations[3])))), // DR
		];
	}

	pub fn rasterize(&self) -> Vec<(f32, f32)> {
		// Order is determined by _this_ orientation.
		let x_mid = (self.left + self.right) / 2;
		let y_mid = (self.top + self.bottom) / 2;
		let ul_pt = ((self.left+x_mid) as f32 / 2f32, (self.top+y_mid) as f32 / 2f32);
		let ur_pt = ((self.right+x_mid) as f32 / 2f32, (self.top+y_mid) as f32 / 2f32);
		let dl_pt = ((self.left+x_mid) as f32 / 2f32, (self.bottom+y_mid) as f32 / 2f32);
		let dr_pt = ((self.right+x_mid) as f32 / 2f32, (self.bottom+y_mid) as f32 / 2f32);
		let mut result = vec![];
		// UL: 0, UR: 1, DL: 2, DR: 3
		let (visit_ordering, backup_pt) = match self.orientation {
			QuadOrientation::A => {
				// DL, UL, UR, DR
				([2, 0, 1, 3], [dl_pt, ul_pt, ur_pt, dr_pt])
			},
			QuadOrientation::B => {
				// UR, UL, DL, DR
				([1, 0, 2, 3], [ur_pt, ul_pt, dl_pt, dr_pt])
			},
			QuadOrientation::C => {
				// UR, DR, DL, UL
				([1, 3, 2, 0], [ur_pt, dr_pt, dl_pt, ul_pt])
			},
			QuadOrientation::D => {
				// DL, DR, UR, UL
				([2, 3, 1, 0], [dl_pt, dr_pt, ur_pt, ul_pt])
			},
		};

		visit_ordering.iter().zip(backup_pt.iter()).for_each(|(order, back_pt)|{
			if let Some(leaf) = &self.leaves[*order] {
				result.append(&mut leaf.rasterize());
			} else {
				result.push(*back_pt);
			}
		});

		result
	}
}

/*
//convert (x,y) to d
int xy2d (int n, int x, int y) {
    int rx, ry, s, d=0;
    for (s=n/2; s>0; s/=2) {
        rx = (x & s) > 0;
        ry = (y & s) > 0;
        d += s * s * ((3 * rx) ^ ry);
        rot(n, &x, &y, rx, ry);
    }
    return d;
}

//convert d to (x,y)
void d2xy(int n, int d, int *x, int *y) {
    int rx, ry, s, t=d;
    *x = *y = 0;
    for (s=1; s<n; s*=2) {
        rx = 1 & (t/2);
        ry = 1 & (t ^ rx);
        rot(s, x, y, rx, ry);
        *x += s * rx;
        *y += s * ry;
        t /= 4;
    }
}

//rotate/flip a quadrant appropriately
void rot(int n, int *x, int *y, int rx, int ry) {
    if (ry == 0) {
        if (rx == 1) {
            *x = n-1 - *x;
            *y = n-1 - *y;
        }

        //Swap x and y
        int t  = *x;
        *x = *y;
        *y = t;
    }
}
 */