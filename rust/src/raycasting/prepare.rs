use crate::geometry::Point;
use crate::ptr_indexed_hash_set::PtrIndexedHashSet;
use crate::raycasting::types::{Cache, Endpoint, VisionAngle, Wall};
use crate::raycasting::vision_angle::restrict_vision_angle;
use crate::raycasting::{DoorState, DoorType, PolygonType, WallSenseType};
use rustc_hash::FxHashMap;
use std::cell::RefCell;
use std::f64::consts::PI;
use std::mem::swap;
use std::rc::Rc;

pub fn prepare_data(
	cache: &Cache,
	origin: Point,
	height: f64,
	vision_angle: &Option<VisionAngle>,
	polygon_type: PolygonType,
) -> (Vec<Rc<RefCell<Endpoint>>>, PtrIndexedHashSet<Wall>) {
	let wall_bases = &cache.walls;
	// TODO Cell/RefCell introduces runtime overhead
	let mut endpoints = FxHashMap::default();
	let mut start_walls = PtrIndexedHashSet::new();
	let mut walls = Vec::with_capacity(wall_bases.len());
	let mut restricted_walls = Vec::new();

	for wall in wall_bases {
		if wall.p1 == wall.p2 {
			continue;
		}
		if wall.p1 == origin || wall.p2 == origin {
			continue;
		}
		if (wall.line.is_vertical() && wall.p1.x == origin.x)
			|| (wall.line.is_horizontal() && wall.line.p1.y == origin.y)
		{
			continue;
		}

		if wall.door != DoorType::NONE && wall.ds == DoorState::OPEN {
			continue;
		}

		let sense = wall.current_sense(&cache, polygon_type);
		if sense == WallSenseType::NONE {
			continue;
		}

		if wall.height.bottom > height || wall.height.top < height {
			continue;
		}

		let e1 = endpoints
			.remove(&wall.p1)
			.unwrap_or_else(|| Rc::new(RefCell::new(Endpoint::new(origin, wall.p1))));
		let e2 = endpoints
			.remove(&wall.p2)
			.unwrap_or_else(|| Rc::new(RefCell::new(Endpoint::new(origin, wall.p2))));

		// Check if the wall's line goes through the light sources center.
		// If so, the wall doesn't have any width and doesn't influence light calculation
		if e1.borrow().angle == e2.borrow().angle
			|| (e1.borrow().angle - e2.borrow().angle).abs() == PI
		{
			endpoints.insert(wall.p1, e1);
			endpoints.insert(wall.p2, e2);
			continue;
		}

		let mut start;
		let mut end;
		if e1.borrow().angle < e2.borrow().angle {
			start = e1;
			end = e2;
		} else {
			start = e2;
			end = e1;
		}
		let is_start_wall;
		if end.borrow().angle - start.borrow().angle > PI {
			swap(&mut start, &mut end);
			is_start_wall = true;
		} else {
			is_start_wall = false;
		}

		let wall = Rc::new(Wall::from_base(
			*wall,
			Rc::clone(&end),
			&cache,
			polygon_type,
		));
		if let Some(split_walls) = restrict_vision_angle(&wall, &start, &end, &vision_angle) {
			for wall in &split_walls {
				if let Some(wall) = wall {
					restricted_walls.push(*wall);
				}
			}
		} else {
			walls.push(Rc::clone(&wall));
			start.borrow_mut().starting_walls.push(Rc::clone(&wall));
			end.borrow_mut().ending_walls.push(Rc::clone(&wall));

			if is_start_wall && !wall.is_see_through_from(-PI) {
				start_walls.insert(Rc::clone(&wall));
			}
		}

		let start_point = start.borrow().point;
		let end_point = end.borrow().point;
		endpoints.insert(start_point, start);
		endpoints.insert(end_point, end);
	}

	for wall in restricted_walls {
		let e1 = endpoints.remove(&wall.p1).unwrap_or_else(|| {
			Rc::new(RefCell::new(Endpoint::new_with_precomputed_angle(
				wall.p1,
				wall.angle_p1,
			)))
		});
		let e2 = endpoints.remove(&wall.p2).unwrap_or_else(|| {
			Rc::new(RefCell::new(Endpoint::new_with_precomputed_angle(
				wall.p2,
				wall.angle_p2,
			)))
		});

		let mut start;
		let mut end;
		if e1.borrow().angle < e2.borrow().angle {
			start = e1;
			end = e2;
		} else {
			start = e2;
			end = e1;
		}
		let is_start_wall;
		if end.borrow().angle - start.borrow().angle > PI {
			swap(&mut start, &mut end);
			is_start_wall = true;
		} else {
			is_start_wall = false;
		}

		let wall = Rc::new(wall.to_wall(Rc::clone(&end)));
		walls.push(Rc::clone(&wall));
		start.borrow_mut().starting_walls.push(Rc::clone(&wall));
		end.borrow_mut().ending_walls.push(Rc::clone(&wall));

		if is_start_wall && !wall.is_see_through_from(-PI) {
			start_walls.insert(Rc::clone(&wall));
		}

		let start_point = start.borrow().point;
		let end_point = end.borrow().point;
		endpoints.insert(start_point, start);
		endpoints.insert(end_point, end);
	}

	for intersection in &cache.intersections {
		if height >= intersection.height.bottom && height <= intersection.height.top {
			endpoints
				.entry(intersection.point)
				.or_insert_with(|| Rc::new(RefCell::new(Endpoint::new(origin, intersection.point))))
				.borrow_mut()
				.is_intersection = true;
		}
	}

	let mut sorted_endpoints = endpoints
		.values()
		.filter(|val| {
			val.borrow().is_intersection
				|| val.borrow().starting_walls.len() + val.borrow().ending_walls.len() > 0
		})
		.map(|val| Rc::clone(&val))
		.collect::<Vec<_>>();
	sorted_endpoints
		.sort_unstable_by(|e1, e2| e1.borrow().angle.partial_cmp(&e2.borrow().angle).unwrap());

	(sorted_endpoints, start_walls)
}
