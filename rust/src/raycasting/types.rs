use wasm_bindgen::prelude::*;

use crate::geometry::{Line, Point};
use crate::raycasting::util::{is_intersection_on_wall, is_smaller_relative};
use std::cell::RefCell;
use std::convert::TryFrom;
use std::f64::consts::PI;
use std::rc::Rc;

#[derive(Clone)]
pub struct ClosestWall {
	pub wall: Rc<Wall>,
	pub intersection: Point,
	pub distance: f64,
}

impl PartialEq for ClosestWall {
	fn eq(&self, other: &Self) -> bool {
		Rc::ptr_eq(&self.wall, &other.wall)
	}
}

impl Eq for ClosestWall {}

#[derive(Debug)]
pub struct Endpoint {
	pub point: Point,
	pub angle: f64,
	pub starting_walls: Vec<Rc<Wall>>,
	pub ending_walls: Vec<Rc<Wall>>,
	pub is_intersection: bool,
}

impl Endpoint {
	pub fn new(origin: Point, target: Point) -> Self {
		let angle = (origin.y - target.y).atan2(origin.x - target.x);
		Self::new_with_precomputed_angle(target, angle)
	}

	pub fn new_with_precomputed_angle(target: Point, angle: f64) -> Self {
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
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum DoorState {
	CLOSED = 0,
	OPEN = 1,
	LOCKED = 2,
}

impl TryFrom<usize> for DoorState {
	type Error = ();
	fn try_from(value: usize) -> Result<Self, Self::Error> {
		match value {
			x if x == Self::CLOSED as usize => Ok(Self::CLOSED),
			x if x == Self::OPEN as usize => Ok(Self::OPEN),
			x if x == Self::LOCKED as usize => Ok(Self::LOCKED),
			_ => Err(()),
		}
	}
}

#[wasm_bindgen]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum DoorType {
	NONE = 0,
	DOOR = 1,
	SECRET = 2,
}

impl TryFrom<usize> for DoorType {
	type Error = ();
	fn try_from(value: usize) -> Result<Self, Self::Error> {
		match value {
			x if x == Self::NONE as usize => Ok(Self::NONE),
			x if x == Self::DOOR as usize => Ok(Self::DOOR),
			x if x == Self::SECRET as usize => Ok(Self::SECRET),
			_ => Err(()),
		}
	}
}

#[wasm_bindgen]
#[allow(dead_code)]
pub struct ExposedEndpoint {
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

#[derive(Debug, Copy, Clone)]
pub struct FovPoint {
	pub point: Point,
	pub angle: f64,
	pub gap: bool,
}

#[derive(Copy, Clone, PartialEq)]
pub enum PolygonType {
	SIGHT = 0,
	SOUND = 1,
}

impl TryFrom<usize> for PolygonType {
	type Error = ();

	fn try_from(value: usize) -> Result<Self, Self::Error> {
		match value {
			x if x == Self::SIGHT as usize => Ok(Self::SIGHT),
			x if x == Self::SOUND as usize => Ok(Self::SOUND),
			_ => Err(()),
		}
	}
}

pub struct VisionAngle {
	pub start: f64,
	pub end: f64,
	pub start_ray: Line,
	pub end_ray: Line,
}

impl VisionAngle {
	pub fn from_rotation_and_angle(rotation: f64, angle: f64, origin: Point) -> Option<Self> {
		if angle >= 360.0 || angle <= 0.0 {
			return None;
		}

		// In Foundry, 0° means down. In Lichtgeschwindigkeit, 0° defaults to left. We need to adjust the angle accordingly.
		let mut rotation = (rotation - 90.0).to_radians();
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

pub struct Wall {
	pub p1: Point,
	pub p2: Point,
	pub line: Line,
	pub sense: WallSenseType,
	pub see_through_angle: Option<f64>,
	pub end: Rc<RefCell<Endpoint>>,
}

impl Wall {
	pub fn from_base(
		base: WallBase,
		end: Rc<RefCell<Endpoint>>,
		polygon_type: PolygonType,
	) -> Self {
		let see_through_angle;
		if base.dir == WallDirection::BOTH {
			see_through_angle = None;
		} else {
			let offset = match base.dir {
				WallDirection::LEFT => 0.0,
				WallDirection::RIGHT => PI,
				WallDirection::BOTH => unreachable!(),
			};
			let mut angle = (base.p1.y - base.p2.y).atan2(base.p1.x - base.p2.x) + offset;
			if angle > PI {
				angle -= 2.0 * PI;
			}
			see_through_angle = Some(angle);
		}
		let sense = base.current_sense(polygon_type);
		Self {
			p1: base.p1,
			p2: base.p2,
			line: base.line,
			sense,
			see_through_angle,
			end,
		}
	}

	pub fn is_see_through_from(&self, angle: f64) -> bool {
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

#[wasm_bindgen]
#[derive(Debug, Copy, Clone)]
pub struct WallBase {
	pub p1: Point,
	pub p2: Point,
	#[wasm_bindgen(skip)]
	pub line: Line,
	pub sense: WallSenseType,
	pub sound: WallSenseType,
	pub door: DoorType,
	pub ds: DoorState,
	pub dir: WallDirection,
	pub height: WallHeight,
}

impl WallBase {
	pub fn new(
		p1: Point,
		p2: Point,
		sense: WallSenseType,
		sound: WallSenseType,
		door: DoorType,
		ds: DoorState,
		dir: WallDirection,
		height: WallHeight,
	) -> Self {
		let line = Line::from_points(p1, p2);
		Self {
			p1,
			p2,
			line,
			sense,
			sound,
			door,
			ds,
			dir,
			height,
		}
	}

	pub fn current_sense(&self, polygon_type: PolygonType) -> WallSenseType {
		match polygon_type {
			PolygonType::SIGHT => self.sense,
			PolygonType::SOUND => self.sound,
		}
	}
}

#[wasm_bindgen]
#[derive(Debug, Copy, Clone)]
pub struct WallHeight {
	pub top: f64,
	pub bottom: f64,
}

impl Default for WallHeight {
	fn default() -> Self {
		Self {
			top: f64::INFINITY,
			bottom: f64::NEG_INFINITY,
		}
	}
}

#[wasm_bindgen]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum WallDirection {
	BOTH = 0,
	LEFT = 1,
	RIGHT = 2,
}

impl TryFrom<usize> for WallDirection {
	type Error = ();
	fn try_from(value: usize) -> Result<Self, Self::Error> {
		match value {
			x if x == Self::BOTH as usize => Ok(Self::BOTH),
			x if x == Self::LEFT as usize => Ok(Self::LEFT),
			x if x == Self::RIGHT as usize => Ok(Self::RIGHT),
			_ => Err(()),
		}
	}
}

#[wasm_bindgen]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum WallSenseType {
	NONE = 0,
	NORMAL = 1,
	LIMITED = 2,
}

impl TryFrom<usize> for WallSenseType {
	type Error = ();
	fn try_from(value: usize) -> Result<Self, Self::Error> {
		match value {
			x if x == Self::NONE as usize => Ok(Self::NONE),
			x if x == Self::NORMAL as usize => Ok(Self::NORMAL),
			x if x == Self::LIMITED as usize => Ok(Self::LIMITED),
			_ => Err(()),
		}
	}
}

#[derive(Copy, Clone)]
pub struct WallWithAngles {
	pub p1: Point,
	pub p2: Point,
	pub angle_p1: f64,
	pub angle_p2: f64,
	pub line: Line,
	pub sense: WallSenseType,
	pub see_through_angle: Option<f64>,
}

impl WallWithAngles {
	pub fn new_copy_props(
		prop_src: &Wall,
		p1: Point,
		p2: Point,
		angle_p1: f64,
		angle_p2: f64,
	) -> Self {
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

	pub fn to_wall(self, end: Rc<RefCell<Endpoint>>) -> Wall {
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

// TODO Locate this into a different module
#[wasm_bindgen]
pub struct Cache {
	#[wasm_bindgen(skip)]
	pub walls: Vec<WallBase>,
	#[wasm_bindgen(skip)]
	pub intersections: Vec<Point>,
}

impl Cache {
	pub fn build(walls: Vec<WallBase>) -> Self {
		let intersections = Self::calc_intersections(&walls);
		Self {
			walls,
			intersections,
		}
	}

	fn calc_intersections(walls: &Vec<WallBase>) -> Vec<Point> {
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
		intersections
	}
}
