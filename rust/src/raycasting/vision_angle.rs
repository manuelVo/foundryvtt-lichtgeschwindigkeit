use crate::geometry::Point;
use crate::raycasting::types::{Endpoint, FovPoint, VisionAngle, Wall, WallWithAngles};
use std::cell::RefCell;
use std::rc::Rc;

use super::util::{between, between_exclusive};

pub fn restrict_vision_angle(
	wall: &Wall,
	start: &Rc<RefCell<Endpoint>>,
	end: &Rc<RefCell<Endpoint>>,
	vision_angle: &Option<VisionAngle>,
) -> Option<[Option<WallWithAngles>; 2]> {
	if let Some(vision_angle) = vision_angle {
		if vision_angle.start < vision_angle.end {
			if !between(start.borrow().angle, vision_angle.start, vision_angle.end)
				&& !between(end.borrow().angle, vision_angle.start, vision_angle.end)
			{
				return Some([None, None]);
			}
			let wall_inverted;
			if start.borrow().angle < end.borrow().angle {
				wall_inverted = false;
			} else {
				wall_inverted = true;
			}

			let mut wall_shortened = false;
			let mut start_point = start.borrow().point;
			let mut start_angle = start.borrow().angle;
			let mut end_point = end.borrow().point;
			let mut end_angle = end.borrow().angle;
			if wall_inverted {
				if start.borrow().angle < vision_angle.start
					|| end.borrow().angle > vision_angle.end
				{
					if let Some(intersection) = vision_angle.start_ray.intersection(&wall.line) {
						wall_shortened = true;
						start_angle = vision_angle.start;
						start_point = intersection;
					}
					if let Some(intersection) = vision_angle.end_ray.intersection(&wall.line) {
						wall_shortened = true;
						end_angle = vision_angle.end;
						end_point = intersection;
					}
				}
			} else {
				if end.borrow().angle > vision_angle.start
					&& start.borrow().angle < vision_angle.start
				{
					if let Some(intersection) = vision_angle.start_ray.intersection(&wall.line) {
						wall_shortened = true;
						start_angle = vision_angle.start;
						start_point = intersection;
					}
				}
				if start.borrow().angle < vision_angle.end && end.borrow().angle > vision_angle.end
				{
					if let Some(intersection) = vision_angle.end_ray.intersection(&wall.line) {
						wall_shortened = true;
						end_angle = vision_angle.end;
						end_point = intersection;
					}
				}
			}
			if wall_shortened {
				let new_wall = WallWithAngles::new_copy_props(
					&wall,
					start_point,
					end_point,
					start_angle,
					end_angle,
				);
				return Some([Some(new_wall), None]);
			}

			if end.borrow().angle < start.borrow().angle {
				// Only remaining option is that end.angle < end.start (which means the wall is to the right, where the circle overflows)
				let mut split_walls = [None, None];
				if end.borrow().angle > vision_angle.start {
					let start_point = vision_angle.start_ray.intersection(&wall.line).unwrap();
					let start_angle = vision_angle.start;
					let end_point = end.borrow().point;
					let end_angle = end.borrow().angle;
					split_walls[0] = Some(WallWithAngles::new_copy_props(
						&wall,
						start_point,
						end_point,
						start_angle,
						end_angle,
					));
				}
				if start.borrow().angle < vision_angle.end {
					let start_point = start.borrow().point;
					let start_angle = start.borrow().angle;
					let end_point = vision_angle.end_ray.intersection(&wall.line).unwrap();
					let end_angle = vision_angle.end;
					split_walls[1] = Some(WallWithAngles::new_copy_props(
						&wall,
						start_point,
						end_point,
						start_angle,
						end_angle,
					));
				}

				if split_walls.iter().all(|wall| wall.is_none())
					&& !between_exclusive(
						start.borrow().angle,
						vision_angle.start,
						vision_angle.end,
					) && !between_exclusive(end.borrow().angle, vision_angle.start, vision_angle.end)
				{
					return None;
				}
				return Some(split_walls);
			}
		} else {
			if start.borrow().angle > end.borrow().angle {
				let mut wall_shortened = false;
				let mut start_point = start.borrow().point;
				let mut start_angle = start.borrow().angle;
				let mut end_point = end.borrow().point;
				let mut end_angle = end.borrow().angle;
				if start.borrow().angle < vision_angle.start {
					if let Some(intersection) = vision_angle.start_ray.intersection(&wall.line) {
						wall_shortened = true;
						start_angle = vision_angle.start;
						start_point = intersection;
					}
				}
				if end.borrow().angle > vision_angle.end {
					if let Some(intersection) = vision_angle.end_ray.intersection(&wall.line) {
						wall_shortened = true;
						end_angle = vision_angle.end;
						end_point = intersection;
					}
				}
				if wall_shortened {
					let new_wall = WallWithAngles::new_copy_props(
						&wall,
						start_point,
						end_point,
						start_angle,
						end_angle,
					);
					return Some([Some(new_wall), None]);
				}
			} else {
				let mut split_walls = [None, None];
				if between_exclusive(vision_angle.end, start.borrow().angle, end.borrow().angle) {
					let start_point = start.borrow().point;
					let start_angle = start.borrow().angle;
					let end_point = vision_angle.end_ray.intersection(&wall.line).unwrap();
					let end_angle = vision_angle.end;
					split_walls[0] = Some(WallWithAngles::new_copy_props(
						&wall,
						start_point,
						end_point,
						start_angle,
						end_angle,
					));
				}
				if between_exclusive(vision_angle.start, start.borrow().angle, end.borrow().angle) {
					let start_point = vision_angle.start_ray.intersection(&wall.line).unwrap();
					let start_angle = vision_angle.start;
					let end_point = end.borrow().point;
					let end_angle = end.borrow().angle;
					split_walls[1] = Some(WallWithAngles::new_copy_props(
						&wall,
						start_point,
						end_point,
						start_angle,
						end_angle,
					));
				}
				if split_walls.iter().all(|wall| wall.is_none())
					&& !between_exclusive(
						start.borrow().angle,
						vision_angle.start,
						vision_angle.end,
					) && !between_exclusive(end.borrow().angle, vision_angle.start, vision_angle.end)
				{
					return None;
				}
				return Some(split_walls);
			}
		}
	}
	None
}

pub fn add_vision_wedge(
	mut los_points: Vec<FovPoint>,
	origin: Point,
	vision_angle: VisionAngle,
	start_gap_fov: &mut bool,
) -> Vec<FovPoint> {
	let mut visible_points_from_start: &[FovPoint];
	let mut visible_points_to_end: &[FovPoint];
	let start_end_swapped;
	if vision_angle.start < vision_angle.end {
		if los_points.len() > 0 && los_points.last().unwrap().angle == vision_angle.end {
			los_points.last_mut().unwrap().gap = false;
		}
		visible_points_from_start = &los_points;
		visible_points_to_end = &[];
		start_end_swapped = false;
		*start_gap_fov = false;
	} else {
		let mut start_index = los_points.len();
		let mut end_index = los_points.len();
		for i in 0..los_points.len() {
			// TODO Check if > or >=
			if los_points[i].angle > vision_angle.end {
				end_index = i;
				break;
			}
		}
		for i in end_index..los_points.len() {
			if los_points[i].angle >= vision_angle.start {
				start_index = i;
				break;
			}
		}
		if end_index < los_points.len() {
			let last_point = &mut los_points[end_index];
			if last_point.angle == vision_angle.end {
				last_point.gap = false;
			}
		}
		visible_points_to_end = &los_points[..end_index];
		if start_index < los_points.len() {
			visible_points_from_start = &los_points[start_index..];
		} else {
			visible_points_from_start = &[];
		}
		start_end_swapped = true;
	}

	let entry;
	/* The angles being exactly equal isn't as unlikely as it seems because we have
	introduced endpoints with perfectly matching angle during endpoint generation */
	if visible_points_from_start.len() > 0
		&& visible_points_from_start.first().unwrap().angle == vision_angle.start
	{
		entry = FovPoint {
			point: origin,
			angle: vision_angle.start,
			gap: false,
		};
	} else {
		entry = FovPoint {
			point: origin,
			angle: vision_angle.start,
			gap: true,
		};
	}

	let exit;
	if start_end_swapped
		&& visible_points_to_end.len() > 0
		&& visible_points_to_end.last().unwrap().angle == vision_angle.end
	{
		let (point, remaining) = visible_points_to_end.split_last().unwrap();
		visible_points_to_end = remaining;
		let mut point = *point;
		point.gap = false;
		exit = vec![
			point,
			FovPoint {
				point: origin,
				angle: vision_angle.end,
				gap: false,
			},
		];
	} else if !start_end_swapped
		&& visible_points_from_start.len() > 0
		&& visible_points_from_start.last().unwrap().angle == vision_angle.end
	{
		let (point, remaining) = visible_points_from_start.split_last().unwrap();
		visible_points_from_start = remaining;
		let mut point = *point;
		point.gap = false;
		exit = vec![
			point,
			FovPoint {
				point: origin,
				angle: vision_angle.end,
				gap: false,
			},
		];
	} else {
		exit = vec![FovPoint {
			point: origin,
			angle: vision_angle.end,
			gap: false,
		}];
	}

	if start_end_swapped {
		[
			visible_points_to_end,
			&exit,
			&[entry],
			visible_points_from_start,
		]
		.concat()
	} else {
		[&[entry], visible_points_from_start, &exit].concat()
	}
}
