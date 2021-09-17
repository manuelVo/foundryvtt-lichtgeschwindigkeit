use crate::geometry::{Line, Point};
use crate::raycasting::types::Wall;
use crate::raycasting::WallBase;
use std::f64::consts::PI;
use std::rc::Rc;

pub fn between<T: Copy + PartialOrd>(num: T, a: T, b: T) -> bool {
	let (min, max) = if a < b { (a, b) } else { (b, a) };
	num >= min && num <= max
}

pub fn between_exclusive<T: Copy + PartialOrd>(num: T, a: T, b: T) -> bool {
	let (min, max) = if a < b { (a, b) } else { (b, a) };
	num > min && num < max
}

// Check if angle1 is smaller than (i.e. is located on the circle counter clockwise from) angle2
// This check is normalized to be able to deal with the overflow between 360° and 0°
pub fn is_smaller_relative(angle1: f64, angle2: f64) -> bool {
	let mut angle_distance = angle2 - angle1;
	if angle_distance.abs() > PI {
		angle_distance *= -1.0;
	}
	return angle_distance > 0.0;
}

// TODO Use a segment class instead of this weird trait
pub trait LineSegment {
	fn line(&self) -> Line;
	fn p1(&self) -> Point;
	fn p2(&self) -> Point;
}

impl LineSegment for Wall {
	fn line(&self) -> Line {
		self.line
	}

	fn p1(&self) -> Point {
		self.p1
	}

	fn p2(&self) -> Point {
		self.p2
	}
}

impl LineSegment for WallBase {
	fn line(&self) -> Line {
		self.line
	}

	fn p1(&self) -> Point {
		self.p1
	}

	fn p2(&self) -> Point {
		self.p2
	}
}

impl<T: LineSegment> LineSegment for Rc<T> {
	fn line(&self) -> Line {
		self.as_ref().line()
	}

	fn p1(&self) -> Point {
		self.as_ref().p1()
	}

	fn p2(&self) -> Point {
		self.as_ref().p2()
	}
}

pub fn is_intersection_on_wall<S: LineSegment>(intersection: Point, wall: &S) -> bool {
	is_intersection_on_segment(intersection, wall.line(), wall.p1(), wall.p2())
}

pub fn is_intersection_on_segment(intersection: Point, line: Line, p1: Point, p2: Point) -> bool {
	if intersection.is_same_as(&p1) || intersection.is_same_as(&p2) {
		return false;
	}
	if line.is_vertical() || line.m.abs() > 1.0 {
		return between(intersection.y, p1.y, p2.y);
	}
	between(intersection.x, p1.x, p2.x)
}
