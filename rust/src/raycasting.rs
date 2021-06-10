use crate::geometry::*;

use js_sys::{Array, Object};
use std::cell::RefCell;
use std::f64::consts::PI;
use std::mem::swap;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
// TODO Try out if this is acutally the optimal hasher to use
use crate::ptr_indexed_hash_set::PtrIndexedHashSet;
use crate::wasm_types::{DoorState, DoorType, WallDirection, WallSenseType};
use rustc_hash::FxHashMap;

#[wasm_bindgen]
#[derive(Debug, Copy, Clone)]
pub struct WallBase {
	pub p1: Point,
	pub p2: Point,
	#[wasm_bindgen(skip)]
	pub line: Line,
	pub sense: WallSenseType,
	pub door: DoorType,
	pub ds: DoorState,
	pub dir: WallDirection,
}

impl WallBase {
	pub fn new(
		p1: Point,
		p2: Point,
		sense: WallSenseType,
		door: DoorType,
		ds: DoorState,
		dir: WallDirection,
	) -> Self {
		let line = Line::from_points(p1, p2);
		Self {
			p1,
			p2,
			line,
			sense,
			door,
			ds,
			dir,
		}
	}
}

#[derive(Copy, Clone)]
pub struct WallWithAngles {
	p1: Point,
	p2: Point,
	angle_p1: f64,
	angle_p2: f64,
	line: Line,
	sense: WallSenseType,
	see_through_angle: Option<f64>,
}

impl WallWithAngles {
	fn new_copy_props(prop_src: &Wall, p1: Point, p2: Point, angle_p1: f64, angle_p2: f64) -> Self {
		Self {
			p1,
			p2,
			angle_p1,
			angle_p2,
			line: Line::from_points(p1, p2),
			sense: prop_src.sense,
			see_through_angle: prop_src.see_through_angle,
		}
	}

	fn to_wall(self, end: Rc<RefCell<Endpoint>>) -> Wall {
		Wall {
			p1: self.p1,
			p2: self.p2,
			line: self.line,
			sense: self.sense,
			see_through_angle: self.see_through_angle,
			end,
		}
	}
}

pub struct Wall {
	p1: Point,
	p2: Point,
	line: Line,
	sense: WallSenseType,
	see_through_angle: Option<f64>,
	end: Rc<RefCell<Endpoint>>,
}

impl Wall {
	fn from_base(base: WallBase, end: Rc<RefCell<Endpoint>>) -> Self {
		let see_through_angle;
		if base.dir == WallDirection::BOTH {
			see_through_angle = None;
		} else {
			let offset = match base.dir {
				WallDirection::LEFT => 0.0,
				WallDirection::RIGHT => PI,
				WallDirection::BOTH => unreachable!(),
			};
			let angle = (base.p1.y - base.p2.y).atan2(base.p1.x - base.p2.x) + offset;
			see_through_angle = Some(angle);
		}
		Self {
			p1: base.p1,
			p2: base.p2,
			line: base.line,
			sense: base.sense,
			see_through_angle,
			end,
		}
	}

	fn is_see_through_from(&self, angle: f64) -> bool {
		if let Some(see_through_angle) = self.see_through_angle {
			if is_smaller_relative(angle, see_through_angle) {
				return true;
			}
		}
		false
	}
}

impl std::fmt::Debug for Wall {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		f.debug_struct("Wall")
			.field("p1", &self.p1)
			.field("p2", &self.p2)
			.field("line", &self.line)
			.finish()
	}
}

#[derive(Debug)]
struct Endpoint {
	point: Point,
	angle: f64,
	starting_walls: Vec<Rc<Wall>>,
	ending_walls: Vec<Rc<Wall>>,
	is_intersection: bool,
}

impl Endpoint {
	fn new(origin: Point, target: Point) -> Self {
		let angle = (origin.y - target.y).atan2(origin.x - target.x);
		Self::new_with_precomputed_angle(target, angle)
	}

	fn new_with_precomputed_angle(target: Point, angle: f64) -> Self {
		Self {
			point: target,
			angle,
			starting_walls: Vec::new(),
			ending_walls: Vec::new(),
			is_intersection: false,
		}
	}
}

#[wasm_bindgen]
#[allow(dead_code)]
struct ExposedEndpoint {
	pub x: f64,
	pub y: f64,
	pub angle: f64,
	#[wasm_bindgen(js_name=isIntersection)]
	pub is_intersection: bool,
}

impl From<&Endpoint> for ExposedEndpoint {
	fn from(endpoint: &Endpoint) -> Self {
		Self {
			x: endpoint.point.x,
			y: endpoint.point.y,
			angle: endpoint.angle,
			is_intersection: endpoint.is_intersection,
		}
	}
}

pub struct VisionAngle {
	start: f64,
	end: f64,
	start_ray: Line,
	end_ray: Line,
}

impl VisionAngle {
	pub fn from_rotation_and_angle(rotation: f64, angle: f64, origin: Point) -> Option<Self> {
		if angle >= 360.0 || angle <= 0.0 {
			return None;
		}
		let mut rotation = rotation.to_radians();
		let angle = angle.to_radians();

		// Normalize the direction
		rotation -= 2.0 * PI * (rotation / (2.0 * PI)).trunc();
		if rotation > PI {
			rotation -= 2.0 * PI;
		}

		let rotation_offset = angle / 2.0;
		let mut start = rotation - rotation_offset;
		let mut end = rotation + rotation_offset;
		if start < -PI {
			start += 2.0 * PI;
		} else if end > PI {
			end -= 2.0 * PI;
		}

		Some(Self {
			start,
			end,
			start_ray: Line::from_point_and_angle(origin, start),
			end_ray: Line::from_point_and_angle(origin, end),
		})
	}
}

#[derive(Clone)]
struct ClosestWall {
	wall: Rc<Wall>,
	intersection: Point,
	distance: f64,
}

impl PartialEq for ClosestWall {
	fn eq(&self, other: &Self) -> bool {
		Rc::ptr_eq(&self.wall, &other.wall)
	}
}

impl Eq for ClosestWall {}

#[derive(Debug, Copy, Clone)]
struct FovPoint {
	point: Point,
	angle: f64,
	gap: bool,
}

#[wasm_bindgen]
extern "C" {
	pub type InternalsTransfer;

	#[wasm_bindgen(method, setter)]
	pub fn set_endpoints(this: &InternalsTransfer, endpoints: Vec<JsValue>);
}

#[wasm_bindgen(js_name=computeSight)]
#[allow(dead_code)]
pub fn js_compute_sight(
	js_walls: Vec<JsValue>,
	origin: JsValue,
	radius: f64,
	distance: f64,
	density: f64,
	angle: f64,
	rotation: f64,
	internals_transfer: Option<InternalsTransfer>,
) -> Object {
	let mut walls = Vec::with_capacity(js_walls.len());
	for wall in js_walls {
		walls.push(WallBase::from(&wall.into()));
	}
	let origin = Point::from(&origin.into());
	let (los, fov) = compute_sight(
		walls,
		origin,
		radius,
		distance,
		density,
		VisionAngle::from_rotation_and_angle(rotation, angle, origin),
		internals_transfer,
	);
	let result = Object::new();
	js_sys::Reflect::set(
		&result,
		&JsValue::from_str("los"),
		&los.into_iter().map(JsValue::from).collect::<Array>(),
	).unwrap();
	js_sys::Reflect::set(
		&result,
		&JsValue::from_str("fov"),
		&fov.into_iter().map(JsValue::from).collect::<Array>(),
	).unwrap();
	result
}

pub fn compute_sight(
	wall_bases: Vec<WallBase>,
	origin: Point,
	radius: f64,
	distance: f64,
	density: f64,
	vision_angle: Option<VisionAngle>,
	internals_transfer: Option<InternalsTransfer>,
) -> (Vec<Point>, Vec<Point>) {
	let (endpoints, mut start_walls) = prepare_data(wall_bases, origin, &vision_angle);

	let (mut los_points, start_gap_los, start_gap_fov) =
		calculate_los(origin, radius, &endpoints, &mut start_walls);

	if let Some(vision_angle) = vision_angle {
		los_points = add_vision_wedge(los_points, origin, vision_angle);
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

fn prepare_data(
	wall_bases: Vec<WallBase>,
	origin: Point,
	vision_angle: &Option<VisionAngle>,
) -> (Vec<Rc<RefCell<Endpoint>>>, PtrIndexedHashSet<Wall>) {
	// TODO Cell/RefCell introduces runtime overhead
	let mut endpoints = FxHashMap::default();
	let mut start_walls = PtrIndexedHashSet::new();
	let mut walls = Vec::with_capacity(wall_bases.len());
	let mut restricted_walls = Vec::new();

	for wall in wall_bases {
		if (wall.line.is_vertical() && wall.p1.x == origin.x)
			|| (wall.line.is_horizontal() && wall.line.p1.y == origin.y)
		{
			continue;
		}

		if wall.door != DoorType::NONE && wall.ds == DoorState::OPEN {
			continue;
		}
		if wall.sense == WallSenseType::NONE {
			continue;
		}

		let e1 = endpoints
			.remove(&wall.p1)
			.unwrap_or_else(|| Rc::new(RefCell::new(Endpoint::new(origin, wall.p1))));
		let e2 = endpoints
			.remove(&wall.p2)
			.unwrap_or_else(|| Rc::new(RefCell::new(Endpoint::new(origin, wall.p2))));

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

		let wall = Rc::new(Wall::from_base(wall, Rc::clone(&end)));
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

	// This condition is to skip populating the cache in case we're handling a universal light
	if walls.len() > 0 {
		for intersection in calc_interections(walls) {
			endpoints
				.entry(*intersection)
				.or_insert_with(|| Rc::new(RefCell::new(Endpoint::new(origin, *intersection))))
				.borrow_mut()
				.is_intersection = true;
		}
	}

	let mut sorted_endpoints = endpoints
		.values()
		.map(|val| Rc::clone(&val))
		.collect::<Vec<_>>();
	sorted_endpoints
		.sort_unstable_by(|e1, e2| e1.borrow().angle.partial_cmp(&e2.borrow().angle).unwrap());

	(sorted_endpoints, start_walls)
}

fn restrict_vision_angle(
	wall: &Wall,
	start: &Rc<RefCell<Endpoint>>,
	end: &Rc<RefCell<Endpoint>>,
	vision_angle: &Option<VisionAngle>,
) -> Option<[Option<WallWithAngles>; 2]> {
	if let Some(vision_angle) = vision_angle {
		if vision_angle.start < vision_angle.end {
			let wall_inverted;
			if start.borrow().angle < end.borrow().angle {
				if start.borrow().angle < vision_angle.start
					&& end.borrow().angle < vision_angle.start
				{
					return Some([None, None]);
				}
				if start.borrow().angle > vision_angle.end && end.borrow().angle > vision_angle.end
				{
					return Some([None, None]);
				}
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
				if start.borrow().angle > vision_angle.end
					&& start.borrow().angle < vision_angle.start
					&& end.borrow().angle > vision_angle.end
					&& end.borrow().angle < vision_angle.start
				{
					return Some([None, None]);
				}
				let mut wall_shortened = false;
				let mut start_point = start.borrow().point;
				let mut start_angle = start.borrow().angle;
				let mut end_point = end.borrow().point;
				let mut end_angle = end.borrow().angle;
				if start.borrow().angle < vision_angle.start
					&& start.borrow().angle > vision_angle.end
				{
					if let Some(intersection) = vision_angle.start_ray.intersection(&wall.line) {
						wall_shortened = true;
						start_angle = vision_angle.start;
						start_point = intersection;
					}
				}
				if end.borrow().angle > vision_angle.end && end.borrow().angle < vision_angle.start
				{
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
			}
		}
	}
	None
}

fn add_vision_wedge(
	mut los_points: Vec<FovPoint>,
	origin: Point,
	vision_angle: VisionAngle,
) -> Vec<FovPoint> {
	let mut start_index = los_points.len();
	let mut end_index = los_points.len();
	let visible_points_from_start: &[FovPoint];
	let visible_points_to_end: &[FovPoint];
	let start_end_swapped;
	if vision_angle.start < vision_angle.end {
		start_index = 0;
		if los_points.len() > 0 && los_points.last().unwrap().angle == vision_angle.end {
			los_points.last_mut().unwrap().gap = false;
		}
		visible_points_from_start = &los_points;
		visible_points_to_end = &[];
		start_end_swapped = false;
	} else {
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
	// This isn't as it seems because we've previously set some points' angles to exactly vision_angle.start
	if start_index < los_points.len() && los_points[start_index].angle == vision_angle.start {
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

	let exit = FovPoint {
		point: origin,
		angle: vision_angle.end,
		gap: false,
	};

	if start_end_swapped {
		[
			visible_points_to_end,
			&[exit],
			&[entry],
			visible_points_from_start,
		]
		.concat()
	} else {
		[&[entry], visible_points_from_start, &[exit]].concat()
	}
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
							// Check if the end point of the new wall is further away from the origin than the end point of the currently closest wall. If so the new wall is completely covered - skip it.
							// For optimization purposes we use Math.pow instead of Math.hypot, because that way we save ourselfs of doing an expensive Math.sqrt, whcih wouldn't change the result of the comparison anyway
							if (origin.x - wall.end.borrow().point.x).powi(2)
								+ (origin.y - wall.end.borrow().point.y).powi(2)
								> (origin.x - closest_wall.wall.end.borrow().point.x).powi(2)
									+ (origin.y - closest_wall.wall.end.borrow().point.y).powi(2)
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
					|| closest_los_wall.as_ref().unwrap().intersection != intersection
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

fn calculate_fov(
	origin: Point,
	radius: f64,
	los_points: &Vec<FovPoint>,
	start_gap_fov: bool,
) -> Vec<FovPoint> {
	let fov = Circle {
		center: origin,
		radius,
	};
	let mut fov_points = Vec::new();
	for i in 0..los_points.len() {
		let los_point = los_points[i];
		let distance = origin.distance_to(&los_point.point);
		if distance <= radius {
			// TODO Properly handle i == 0
			if i == 0 {
				fov_points.push(los_point);
			} else {
				let previous_los = los_points[i - 1];
				// TODO What if there is no previous?
				let previous_fov_gap = fov_points
					.last()
					.map(|previous| previous.gap)
					.unwrap_or(start_gap_fov);
				if previous_fov_gap && !previous_los.gap {
					if previous_los.angle == los_point.angle {
						let point = Point {
							x: fov.center.x - los_point.angle.cos() * radius,
							y: fov.center.y - los_point.angle.sin() * radius,
						};
						fov_points.push(FovPoint {
							point,
							angle: los_point.angle,
							gap: false,
						});
					} else {
						let line = Line::from_points(previous_los.point, los_point.point);
						// TODO Verify that there is always an intersection
						let fov_intersections = fov.intersections(&line).unwrap();
						let relevant_intersection;
						// TODO is_smaller_relative
						if fov_intersections.0.angle > previous_los.angle
							&& fov_intersections.0.angle < los_point.angle
						{
							relevant_intersection = fov_intersections.0;
						} else {
							relevant_intersection = fov_intersections.1;
						}
						fov_points.push(FovPoint {
							point: relevant_intersection.point,
							angle: relevant_intersection.angle,
							gap: false,
						})
					}
				}
				fov_points.push(los_point);
			}
		} else {
			let previous_fov_gap = fov_points
				.last()
				.map(|previous| previous.gap)
				.unwrap_or(start_gap_fov);
			if !previous_fov_gap {
				if i > 0 {
					let hidden_point = los_point;
					let point_before_hidden = los_points[i - 1];
					if point_before_hidden.angle == hidden_point.angle {
						let point = Point {
							x: fov.center.x - hidden_point.angle.cos() * radius,
							y: fov.center.y - hidden_point.angle.sin() * radius,
						};
						fov_points.push(FovPoint {
							point,
							angle: los_point.angle,
							gap: true,
						});
					} else {
						let line = Line::from_points(point_before_hidden.point, hidden_point.point);
						if let Some(fov_intersections) = fov.intersections(&line) {
							let relevant_intersection;
							// TODO is_smaller_relative
							if fov_intersections.0.angle > point_before_hidden.angle
								&& fov_intersections.0.angle < hidden_point.angle
							{
								relevant_intersection = fov_intersections.0;
							} else {
								relevant_intersection = fov_intersections.1;
							}
							fov_points.push(FovPoint {
								point: relevant_intersection.point,
								angle: relevant_intersection.angle,
								gap: true,
							});
						}
						if !start_gap_fov && i == los_points.len() - 1 {
							let next_los = los_points.first().unwrap();
							let line = Line::from_points(los_point.point, next_los.point);
							let intersections = fov.intersections(&line).unwrap();
							let entry;
							// The wall is to the right of the token, so the angles are inverted
							if intersections.0.angle > intersections.1.angle {
								entry = intersections.0;
							} else {
								entry = intersections.1;
							}
							fov_points.push(FovPoint {
								point: entry.point,
								angle: entry.angle,
								gap: false,
							});
						}
					}
				} else {
					let previous_los = los_points.last().unwrap();
					let line = Line::from_points(previous_los.point, los_point.point);
					let intersections = fov.intersections(&line).unwrap();
					let exit;
					// The wall is to the right of the token, so the angles are inverted
					if intersections.0.angle > intersections.1.angle {
						exit = intersections.1;
					} else {
						exit = intersections.0;
					}
					fov_points.push(FovPoint {
						point: exit.point,
						angle: exit.angle,
						gap: true,
					});
				}
			} else {
				// TODO Handle i == 0
				if i > 0 {
					let previous_los = los_points[i - 1];
					if !previous_los.gap {
						let line = Line::from_points(previous_los.point, los_point.point);
						if let Some(intersections) = fov.intersections(&line) {
							// TODO Is smaller relative?
							if intersections.0.angle > previous_los.angle
								&& intersections.1.angle > previous_los.angle
								&& intersections.0.angle < los_point.angle
								&& intersections.1.angle < los_point.angle
							{
								let (entry, exit);
								// TODO Is smaller relative?
								if intersections.0.angle < intersections.1.angle {
									entry = intersections.0;
									exit = intersections.1;
								} else {
									entry = intersections.1;
									exit = intersections.0;
								}
								fov_points.push(FovPoint {
									point: entry.point,
									angle: entry.angle,
									gap: false,
								});
								fov_points.push(FovPoint {
									point: exit.point,
									angle: exit.angle,
									gap: true,
								});
							}
						}
					}
					if !start_gap_fov && i == los_points.len() - 1 {
						let next_los = los_points.first().unwrap();
						let line = Line::from_points(los_point.point, next_los.point);
						let intersections = fov.intersections(&line).unwrap();
						let entry;
						// The wall is to the right of the token, so the angles are inverted
						if intersections.0.angle > intersections.1.angle {
							entry = intersections.0;
						} else {
							entry = intersections.1;
						}
						fov_points.push(FovPoint {
							point: entry.point,
							angle: entry.angle,
							gap: false,
						});
					}
				}
			}
		}
	}
	fov_points
}

fn fill_gaps(
	points: &mut Vec<FovPoint>,
	start_gap: bool,
	origin: Point,
	radius: f64,
	radial_density: f64,
) -> Vec<Point> {
	let mut output = Vec::new();

	if points.len() == 0 {
		let mut a = -PI;
		while a < PI {
			output.push(Point::new(
				origin.x - (a.cos() * radius),
				origin.y - (a.sin() * radius),
			));
			a += radial_density;
		}
	} else {
		if points.last().unwrap().point != origin {
			points.last_mut().unwrap().gap = start_gap;
		}
		for i in 0..points.len() {
			// TODO This produces a quite big assembly. Think of something faster
			let (lower, upper) = points.split_at(i);
			let (current, upper) = upper.split_at(1);
			let current = current.first().unwrap();
			let previous;
			if i == 0 {
				previous = upper.last().unwrap();
			} else {
				previous = lower.last().unwrap();
			}
			if previous.gap {
				let mut previous_angle = previous.angle;
				if previous_angle > current.angle {
					previous_angle -= 2.0 * PI;
				}
				let mut a = previous_angle;
				while a < current.angle {
					output.push(Point::new(
						origin.x - (a.cos() * radius),
						origin.y - (a.sin() * radius),
					));
					a += radial_density;
				}
				output.push(Point::new(
					origin.x - (current.angle.cos() * radius),
					origin.y - (current.angle.sin() * radius),
				));
			}
			output.push(current.point);
		}
	}

	output
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

static mut INTERSECTION_CACHE: Option<Vec<Point>> = None;

#[allow(dead_code)]
#[wasm_bindgen(js_name=wipeCache)]
pub fn wipe_cache() {
	unsafe {
		INTERSECTION_CACHE = None;
	}
}

fn calc_interections(walls: Vec<Rc<Wall>>) -> &'static Vec<Point> {
	let cache;
	unsafe {
		cache = INTERSECTION_CACHE.as_ref();
		//cache = None;
	}
	match cache {
		Some(cache) => cache,
		None => {
			let mut intersections = Vec::new();
			if walls.len() >= 2 {
				for i in 0..walls.len() - 1 {
					for j in 0..walls.len() - i - 1 {
						let (i_walls, j_walls) = walls.split_at(i + 1);
						let wall1 = &i_walls[i];
						let wall2 = &j_walls[j];
						let intersection = wall1.line.intersection(&wall2.line);
						match intersection {
							Some(intersection) => {
								if is_intersection_on_wall(intersection, wall1)
									&& is_intersection_on_wall(intersection, wall2)
								{
									intersections.push(intersection);
								}
							}
							None => {}
						};
					}
				}
			}
			unsafe {
				INTERSECTION_CACHE = Some(intersections);
				INTERSECTION_CACHE.as_ref().unwrap()
			}
		}
	}
}

fn is_intersection_on_wall(intersection: Point, wall: &Wall) -> bool {
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

fn between<T: Copy + PartialOrd>(num: T, a: T, b: T) -> bool {
	let (min, max) = if a < b { (a, b) } else { (b, a) };
	num >= min && num <= max
}

// Check if angle1 is smaller than (i.e. is located on the circle counter clockwise from) angle2
// This check is normalized to be able to deal with the overflow between 360° and 0°
fn is_smaller_relative(angle1: f64, angle2: f64) -> bool {
	let mut angle_distance = angle2 - angle1;
	if angle_distance.abs() > PI {
		angle_distance *= -1.0;
	}
	return angle_distance > 0.0;
}
