use crate::geometry::*;

use std::cell::RefCell;
use std::mem::swap;
use std::rc::Rc;
// TODO Try out if this is acutally the optimal hasher to use
use crate::ptr_indexed_hash_set::PtrIndexedHashSet;
use crate::raycasting::js_api::InternalsTransfer;
use crate::raycasting::postprocessing::{calculate_fov, fill_gaps};
use crate::raycasting::prepare::prepare_data;
use crate::raycasting::types::*;
use crate::raycasting::util::{is_intersection_on_wall, is_smaller_relative};
use crate::raycasting::vision_angle::add_vision_wedge;

pub fn compute_polygon(
	wall_bases: Vec<WallBase>,
	origin: Point,
	radius: f64,
	distance: f64,
	density: f64,
	vision_angle: Option<VisionAngle>,
	internals_transfer: Option<InternalsTransfer>,
) -> (Vec<Point>, Vec<Point>) {
	let (endpoints, mut start_walls) = prepare_data(wall_bases, origin, &vision_angle);

	let (mut los_points, start_gap_los, mut start_gap_fov) =
		calculate_los(origin, radius, &endpoints, &mut start_walls);

	if let Some(vision_angle) = vision_angle {
		los_points = add_vision_wedge(los_points, origin, vision_angle, &mut start_gap_fov);
	}

	let mut fov_points = calculate_fov(origin, radius, &los_points, start_gap_fov);

	// Report endpoints if debugging is enabled
	if let Some(internals_transfer) = internals_transfer {
		internals_transfer.set_endpoints(
			endpoints
				.iter()
				.map(|endpoint| ExposedEndpoint::from(&*endpoint.borrow()).into())
				.collect(),
		);
	}

	// Clean up references to the walls in the endpoints to avoid a memory leak (walls and endpoints have Rc's to each other in a cyclic way
	for endpoint in endpoints {
		endpoint.borrow_mut().starting_walls.clear();
		endpoint.borrow_mut().ending_walls.clear();
	}

	let radial_density = density.to_radians();
	let los = fill_gaps(
		&mut los_points,
		start_gap_los,
		origin,
		distance,
		radial_density,
	);
	let fov = fill_gaps(
		&mut fov_points,
		start_gap_fov,
		origin,
		radius,
		radial_density,
	);

	(los, fov)
}

fn calculate_los(
	origin: Point,
	radius: f64,
	endpoints: &Vec<Rc<RefCell<Endpoint>>>,
	start_walls: &mut PtrIndexedHashSet<Wall>,
) -> (Vec<FovPoint>, bool, bool) {
	let mut los_points = Vec::new();
	let current_walls = start_walls;
	let mut current_ray_line = Line::new(0.0, origin.y as f64, origin);
	let mut closest_los_wall =
		find_closest_wall::<_, false>(origin, &current_ray_line, &*current_walls);
	let start_gap_los = closest_los_wall.is_none();
	let start_gap_fov = closest_los_wall
		.as_ref()
		.filter(|closest_wall| closest_wall.distance < radius)
		.is_none();

	for i in 0..endpoints.len() {
		let endpoint = endpoints[i].borrow();
		let old_los_wall = closest_los_wall.clone();
		current_ray_line = Line::from_points(origin, endpoint.point);
		let mut closest_wall_could_change = endpoint.is_intersection;
		for wall in &endpoint.ending_walls {
			let element_removed = current_walls.remove(wall);
			if element_removed {
				closest_wall_could_change = true;
			}
		}

		for wall in &endpoint.starting_walls {
			if wall.is_see_through_from(endpoint.angle) {
				continue;
			}
			if let Some(closest_wall) = &closest_los_wall {
				// This optimization doesn't work yet for terrain walls
				if closest_wall.wall.sense != WallSenseType::LIMITED {
					// Let's see if the wall is completely behind the currently closest wall. If so, we can skip it.
					if is_smaller_relative(
						wall.end.borrow().angle,
						closest_wall.wall.end.borrow().angle,
					) {
						// Probe if the walls have any chance of intersecting. If not, the new wall is either completely in front or behind of the currently closest wall.
						// TODO Check if the heuristic if faster. If not, adjust the above comment
						// TODO Only do segment intersection
						let mut intersection = wall.line.intersection(&closest_wall.wall.line);
						// The above only gets the intersection point between the lines. We also need to check if the point is on both wall segments
						if let Some(i) = intersection {
							if !is_intersection_on_wall(i, wall)
								|| !is_intersection_on_wall(i, &closest_wall.wall)
							{
								intersection = None;
							}
						}
						if intersection.is_none() {
							// Check if the endpoint is before or behind the currently closest wall - if it is behind the wall is completely covered, skip it.
							let intersection = current_ray_line
								.intersection(&closest_wall.wall.line)
								.unwrap();

							// For optimization purposes we use Math.pow instead of Math.hypot, because that way we save ourselfs of doing an expensive Math.sqrt, whcih wouldn't change the result of the comparison anyway
							if (origin.x - endpoint.point.x).powi(2)
								+ (origin.y - endpoint.point.y).powi(2)
								> (origin.x - intersection.x).powi(2)
									+ (origin.y - intersection.y).powi(2)
							{
								continue;
							}
						}
					}
				}
			}
			closest_wall_could_change = true;
			current_walls.insert(Rc::clone(wall));
		}

		if i + 1 < endpoints.len() && endpoints[i + 1].borrow().angle == endpoint.angle {
			continue;
		}

		if i > 0 && endpoints[i - 1].borrow().angle == endpoint.angle {
			closest_wall_could_change = true;
		}

		if closest_wall_could_change {
			closest_los_wall =
				find_closest_wall::<_, false>(origin, &current_ray_line, &*current_walls);
		}

		if old_los_wall != closest_los_wall {
			if let Some(old_closest_wall) = old_los_wall {
				let intersection = current_ray_line
					.intersection(&old_closest_wall.wall.line)
					.unwrap();
				if closest_los_wall.is_none()
					|| !closest_los_wall
						.as_ref()
						.unwrap()
						.intersection
						.is_same_as(&intersection)
				{
					los_points.push(FovPoint {
						point: intersection,
						angle: endpoint.angle,
						gap: false,
					});
				}
			}

			if let Some(closest_wall) = &mut closest_los_wall {
				los_points.push(FovPoint {
					point: closest_wall.intersection,
					angle: endpoint.angle,
					gap: false,
				});
			} else {
				los_points.last_mut().unwrap().gap = true;
			}
		}
	}

	(los_points, start_gap_los, start_gap_fov)
}

fn find_closest_wall<'a, I, const IS_TIEBREAKER: bool>(
	origin: Point,
	current_ray_line: &Line,
	current_walls: I,
) -> Option<ClosestWall>
where
	I: IntoIterator<Item = &'a Rc<Wall>>,
{
	let mut closest_wall = None;
	let mut second_closest_wall = None;
	let use_y_distance = current_ray_line.is_vertical() || current_ray_line.m.abs() > 1.0;
	let mut ties = Vec::new();
	let mut second_closest_ties = Vec::new();
	for wall in current_walls {
		let intersection = current_ray_line.intersection(&wall.line);
		if let Some(intersection) = intersection {
			let distance;
			if use_y_distance {
				distance = (intersection.y - origin.y).abs();
			} else {
				distance = (intersection.x - origin.x).abs();
			}
			if let Some(closest_distance) = closest_wall
				.as_ref()
				.map(|wall: &ClosestWall| wall.distance)
			{
				let e = 0.0001;
				// distance == closest_distance
				if (distance - closest_distance).abs() < e {
					if !IS_TIEBREAKER {
						ties.push(Rc::clone(wall));
					}
				} else if distance < closest_distance {
					second_closest_wall = closest_wall;
					closest_wall = Some(ClosestWall {
						wall: Rc::clone(wall),
						intersection,
						distance,
					});
					swap(&mut ties, &mut second_closest_ties);
					ties.clear();
				} else if let Some(second_closest_distance) =
					second_closest_wall.as_ref().map(|wall| wall.distance)
				{
					// distance == second_closest_distance
					if (distance - second_closest_distance).abs() < e {
						if !IS_TIEBREAKER {
							second_closest_ties.push(Rc::clone(wall));
						}
					} else if distance < second_closest_distance {
						second_closest_wall = Some(ClosestWall {
							wall: Rc::clone(wall),
							intersection,
							distance,
						});
						second_closest_ties.clear();
					}
				} else {
					second_closest_wall = Some(ClosestWall {
						wall: Rc::clone(wall),
						intersection,
						distance,
					});
				}
			} else {
				closest_wall = Some(ClosestWall {
					wall: Rc::clone(wall),
					intersection,
					distance,
				});
			}
		}
	}
	if !IS_TIEBREAKER && ties.len() > 0 {
		let closest_wall = closest_wall.as_mut().unwrap();
		ties.push(Rc::clone(&closest_wall.wall));
		closest_wall.wall = find_closest_wall_tiebreaker(origin, &ties).unwrap().wall;
	}
	if let Some(closest_wall_ref) = &mut closest_wall {
		if !IS_TIEBREAKER && closest_wall_ref.wall.sense == WallSenseType::LIMITED {
			if ties.len() > 0 {
				let ties: Vec<_> = ties
					.into_iter()
					.filter(|wall| !Rc::ptr_eq(wall, &closest_wall_ref.wall))
					.collect();
				closest_wall_ref.wall = find_closest_wall_tiebreaker(origin, &ties).unwrap().wall;
			} else if second_closest_ties.len() > 0 {
				closest_wall = second_closest_wall;
				let closest_wall = closest_wall.as_mut().unwrap();
				second_closest_ties.push(Rc::clone(&closest_wall.wall));
				closest_wall.wall = find_closest_wall_tiebreaker(origin, &second_closest_ties)
					.unwrap()
					.wall;
			} else {
				closest_wall = second_closest_wall;
			}
		}
	}
	closest_wall
}

fn find_closest_wall_tiebreaker(origin: Point, ties: &Vec<Rc<Wall>>) -> Option<ClosestWall> {
	let first_ending_wall = ties
		.iter()
		.reduce(|w1, w2| {
			if is_smaller_relative(w1.end.borrow().angle, w2.end.borrow().angle) {
				w1
			} else {
				w2
			}
		})
		.unwrap();
	let ray_to_endpoint = Line::from_points(origin, first_ending_wall.end.borrow().point);
	find_closest_wall::<_, true>(origin, &ray_to_endpoint, ties)
}
