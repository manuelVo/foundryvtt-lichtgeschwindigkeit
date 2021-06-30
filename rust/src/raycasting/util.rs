use crate::geometry::{Line, Point};
use crate::raycasting::types::Wall;
use std::f64::consts::PI;

pub fn between<T: Copy + PartialOrd>(num: T, a: T, b: T) -> bool {
	let (min, max) = if a < b { (a, b) } else { (b, a) };
	num >= min && num <= max
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

pub fn is_intersection_on_wall(intersection: Point, wall: &Wall) -> bool {
	is_intersection_on_segment(intersection, wall.line, wall.p1, wall.p2)
}

pub fn is_intersection_on_segment(intersection: Point, line: Line, p1: Point, p2: Point) -> bool {
	if intersection.is_same_as(&p1) || intersection.is_same_as(&p2) {
		return false;
	}
	if line.is_vertical() || line.m > 1.0 {
		return between(intersection.y, p1.y, p2.y);
	}
	between(intersection.x, p1.x, p2.x)
}
