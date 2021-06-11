use crate::geometry::Point;
use crate::raycasting::WallBase;
use std::convert::TryFrom;
use wasm_bindgen::prelude::*;

#[allow(unused)]
macro_rules! log {
	( $( $t:tt )* ) => {
		crate::wasm_types::log(&format!( $( $t )* ));
	};
}

#[wasm_bindgen]
extern "C" {
	#[wasm_bindgen(js_namespace = console, js_name=warn)]
	pub fn log(s: &str);
}

#[wasm_bindgen]
extern "C" {

	pub type JsWall;
	pub type JsWallData;
	pub type JsTile;

	#[wasm_bindgen(method, getter)]
	fn data(this: &JsWall) -> JsWallData;

	#[wasm_bindgen(method, getter)]
	fn roof(this: &JsWall) -> Option<JsTile>;

	#[wasm_bindgen(method, getter)]
	fn occluded(this: &JsTile) -> bool;

	#[wasm_bindgen(method, getter)]
	fn c(this: &JsWallData) -> Vec<f64>;

	#[wasm_bindgen(method, getter)]
	fn door(this: &JsWallData) -> DoorType;

	#[wasm_bindgen(method, getter)]
	fn ds(this: &JsWallData) -> DoorState;

	#[wasm_bindgen(method, getter)]
	fn sense(this: &JsWallData) -> WallSenseType;

	#[wasm_bindgen(method, getter)]
	fn sound(this: &JsWallData) -> WallSenseType;

	#[wasm_bindgen(method, getter)]
	fn dir(this: &JsWallData) -> Option<WallDirection>;
}

impl WallBase {
	pub fn from_js(wall: &JsWall, polygon_type: PolygonType) -> Self {
		let data = wall.data();
		let c = data.c();
		let mut sense = match polygon_type {
			PolygonType::SIGHT => data.sense(),
			PolygonType::SOUND => data.sound(),
		};
		if polygon_type == PolygonType::SIGHT {
			let is_interior = !wall.roof().map(|roof| roof.occluded()).unwrap_or(true);
			if is_interior {
				sense = WallSenseType::NORMAL;
			}
		}
		Self::new(
			Point::new(c[0].round(), c[1].round()),
			Point::new(c[2].round(), c[3].round()),
			sense,
			data.door(),
			data.ds(),
			data.dir().unwrap_or(WallDirection::BOTH),
		)
	}
}

#[derive(Copy, Clone, PartialEq)]
pub enum PolygonType {
	SIGHT = 0,
	SOUND = 1,
}

impl From<&str> for PolygonType {
	fn from(value: &str) -> Self {
		match value {
			"sight" => Self::SIGHT,
			"sound" => Self::SOUND,
			_ => {
				log!(
					"Lichtgeschwindigkeit | Unknown polygon type '{}', assuming 'sight'",
					value
				);
				Self::SIGHT
			}
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
