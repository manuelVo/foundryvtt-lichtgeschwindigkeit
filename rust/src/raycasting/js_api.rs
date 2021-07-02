use crate::geometry::Point;
use crate::raycasting::types::{Cache, PolygonType, VisionAngle, WallBase};
use crate::raycasting::{compute_polygon, DoorState, DoorType, WallDirection, WallSenseType};
use js_sys::{Array, Object};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name=computePolygon)]
#[allow(dead_code)]
pub fn js_compute_polygon(
	cache: &Cache,
	origin: JsValue,
	radius: f64,
	distance: f64,
	density: f64,
	angle: f64,
	rotation: f64,
	internals_transfer: Option<InternalsTransfer>,
) -> Object {
	let origin = Point::from(&origin.into());
	let (los, fov) = compute_polygon(
		&cache,
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
	)
	.unwrap();
	js_sys::Reflect::set(
		&result,
		&JsValue::from_str("fov"),
		&fov.into_iter().map(JsValue::from).collect::<Array>(),
	)
	.unwrap();
	result
}

#[allow(dead_code)]
#[wasm_bindgen(js_name=buildCache)]
pub fn build_cache(js_walls: Vec<JsValue>, polygon_type: &str) -> Cache {
	let polygon_type = PolygonType::from(polygon_type);
	let mut walls = Vec::with_capacity(js_walls.len());
	for wall in js_walls {
		walls.push(WallBase::from_js(&wall.into(), polygon_type));
	}
	Cache::build(walls)
}

#[allow(dead_code)]
#[wasm_bindgen(js_name=wipeCache)]
pub fn wipe_cache(cache: Cache) {
	drop(cache);
}

#[allow(unused)]
macro_rules! log {
	( $( $t:tt )* ) => {
		log(&format!( $( $t )* ));
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

impl From<&str> for PolygonType {
	fn from(value: &str) -> Self {
		match value {
			"sight" => Self::SIGHT,
			"light" => Self::SIGHT,
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
extern "C" {
	pub type InternalsTransfer;

	#[wasm_bindgen(method, setter)]
	pub fn set_endpoints(this: &InternalsTransfer, endpoints: Vec<JsValue>);
}
