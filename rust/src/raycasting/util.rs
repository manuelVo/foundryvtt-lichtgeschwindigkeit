use crate::geometry::Point;
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
	if intersection.x == wall.p1.x && intersection.y == wall.p1.y
		|| intersection.x == wall.p2.x && intersection.y == wall.p2.y
	{
		return false;
	}
	if wall.line.is_vertical() || wall.line.m > 1.0 {
		return between(intersection.y, wall.p1.y, wall.p2.y);
	}
	between(intersection.x, wall.p1.x, wall.p2.x)
}
