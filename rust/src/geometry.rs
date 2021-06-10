use std::f64::consts::PI;
use std::hash::{Hash, Hasher};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
	pub type JsPoint;

	#[wasm_bindgen(method, getter)]
	fn x(this: &JsPoint) -> f64;

	#[wasm_bindgen(method, getter)]
	fn y(this: &JsPoint) -> f64;
}

#[wasm_bindgen]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Point {
	pub x: f64,
	pub y: f64,
}

impl Hash for Point {
	fn hash<H: Hasher>(&self, hasher: &mut H) {
		self.x.to_bits().hash(hasher);
		self.y.to_bits().hash(hasher);
	}
}

impl Eq for Point {}

impl From<&JsPoint> for Point {
	fn from(point: &JsPoint) -> Self {
		Self::new(point.x(), point.y())
	}
}

impl Point {
	pub fn new(x: f64, y: f64) -> Self {
		Self { x, y }
	}

	pub fn from_line_x(line: &Line, x: f64) -> Self {
		let y = line.calc_y(x);
		Self { x, y }
	}

	pub fn distance_to(&self, other: &Self) -> f64 {
		(self.x - other.x).hypot(self.y - other.y)
	}
}

#[derive(Debug, Copy, Clone)]
pub struct Line {
	pub m: f64,
	pub b: f64,
	pub p1: Point,
}

impl Line {
	pub fn new(m: f64, b: f64, p1: Point) -> Self {
		Self { m, b, p1 }
	}

	pub fn from_points(p1: Point, p2: Point) -> Self {
		let m = (p1.y - p2.y) / (p1.x - p2.x);
		let b = p1.y - m * p1.x;
		Self { m, b, p1 }
	}

	pub fn from_point_and_angle(p1: Point, angle: f64) -> Self {
		let p2 = Point {
			x: p1.x - angle.cos(),
			y: p1.y - angle.sin(),
		};
		Line::from_points(p1, p2)
	}

	pub fn empty() -> Self {
		Self {
			m: 0.0,
			b: 0.0,
			p1: Point::new(0.0, 0.0),
		}
	}

	pub fn is_vertical(&self) -> bool {
		self.m.is_infinite()
	}

	pub fn is_horizontal(&self) -> bool {
		self.m == 0.0
	}

	pub fn calc_x(&self, y: f64) -> f64 {
		(y - self.b) / self.m
	}

	pub fn calc_y(&self, x: f64) -> f64 {
		self.m * x + self.b
	}

	pub fn intersection(&self, other: &Line) -> Option<Point> {
		// Are both lines vertical?
		if self.is_vertical() && other.is_vertical() {
			return None;
		}

		// Are the lines paralell?
		if (self.m - other.m).abs() < 0.00000005 {
			return None;
		}

		// Is one of the lines vertical?
		if self.is_vertical() || other.is_vertical() {
			let vertical;
			let regular;
			if self.is_vertical() {
				vertical = self;
				regular = other;
			} else {
				vertical = other;
				regular = self;
			}
			return Some(Point::from_line_x(&regular, vertical.p1.x));
		}

		// Calculate x coordinate of intersection point between both lines
		// Find intersection point: x * m1 + b1 = x * m2 + b2
		// Solve for x: x = (b1 - b2) / (m2 - m1)
		let x = (self.b - other.b) / (other.m - self.m);
		if self.m.abs() < other.m.abs() {
			Some(Point::from_line_x(&self, x))
		} else {
			Some(Point::from_line_x(&other, x))
		}
	}

	pub fn get_perpendicular_through_point(&self, p: Point) -> Self {
		let m = -1.0 / self.m;
		let b = p.y - m * p.x;
		Self { m, b, p1: p }
	}
}

#[derive(Copy, Clone)]
pub struct CircleIntersection {
	pub point: Point,
	pub angle: f64,
}

#[derive(Copy, Clone)]
pub struct Circle {
	pub center: Point,
	pub radius: f64,
}

impl Circle {
	// We don't care about tangent points. That's why we either return two or no points
	pub fn intersections(&self, line: &Line) -> Option<(CircleIntersection, CircleIntersection)> {
		// We first seach for the closest point of the line to the center.
		// If intersections exist, that is the halfway point between both intersections
		let perpendicular = line.get_perpendicular_through_point(self.center);
		let closest_point = perpendicular.intersection(line).unwrap();

		// Calculate how far the closest point on the line is away from the circles center
		let closest_distance = self.center.distance_to(&closest_point);

		// closestDistance > radius means 0 intersections
		// closestDistance == radius the line is a tangent
		// closestDistance < radius means 2 intersections
		// We only care about the intersections, so we filter the other cases out
		if closest_distance >= self.radius {
			return None;
		}

		let intersection1_angle;
		let intersection2_angle;
		if closest_distance > 0.0 {
			// This is the usual case where the line is *not* going through the circle's center

			// Calculate the angle of the perpendicular relative to the global coordiante system
			let perpendicular_angle =
				(self.center.y - closest_point.y).atan2(self.center.x - closest_point.x);

			// Calculate the angle between the perpendicular and the first intersection
			let intersection_to_perpendicular_angle = (closest_distance / self.radius).acos();

			// Calculate the angle between the intersection an the global coordinate system
			intersection1_angle = perpendicular_angle + intersection_to_perpendicular_angle;
			intersection2_angle = perpendicular_angle - intersection_to_perpendicular_angle;
		} else {
			// This happens if the line goes through the circles center.

			// Calculate the angle from the circle center + a point on the line
			// Taking p1 is risky, because the point could be the circle center itself. That should never happen for Lichtgeschwindigkeit though
			assert_ne!(self.center, line.p1);
			intersection1_angle = (self.center.y - line.p1.y).atan2(self.center.x - line.p1.x);
			intersection2_angle = intersection1_angle + PI;
		}

		// The first intersection point an now be determined
		let mut intersection1 = CircleIntersection {
			point: Point::new(
				self.center.x - intersection1_angle.cos() * self.radius,
				self.center.y - intersection1_angle.sin() * self.radius,
			),
			angle: intersection1_angle,
		};

		// Mirror intersection 1 along the perpendicular to find intersection 2
		let mut intersection2 = CircleIntersection {
			point: Point::new(
				closest_point.x - (intersection1.point.x - closest_point.x),
				closest_point.y - (intersection1.point.y - closest_point.y),
			),
			angle: intersection2_angle,
		};

		// Normalize the intersection angles
		if intersection1.angle > PI {
			intersection1.angle -= 2.0 * PI;
		}
		if intersection2.angle < -PI {
			intersection2.angle += 2.0 * PI;
		}

		Some((intersection1, intersection2))
	}
}
