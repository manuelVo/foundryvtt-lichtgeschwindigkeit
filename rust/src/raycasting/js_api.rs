use crate::geometry::Point;
use crate::raycasting::types::{
	Cache, PolygonType, TileCache, TileId, VisionAngle, WallBase, WallHeight,
};
use crate::raycasting::{compute_polygon, DoorState, DoorType, WallDirection, WallSenseType};
use js_sys::{Array, Object};
use rustc_hash::FxHashMap;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name=computePolygon)]
#[allow(dead_code)]
pub fn js_compute_polygon(
	cache: &Cache,
	origin: JsValue,
	height: f64,
	radius: f64,
	distance: f64,
	density: f64,
	angle: f64,
	rotation: f64,
	polygon_type: &str,
	internals_transfer: Option<InternalsTransfer>,
) -> Object {
	let origin = Point::from(&origin.into());
	let polygon_type = PolygonType::from(polygon_type);
	let (los, fov) = compute_polygon(
		&cache,
		origin,
		height,
		radius,
		distance,
		density,
		VisionAngle::from_rotation_and_angle(rotation, angle, origin),
		polygon_type,
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
#[wasm_bindgen(js_name=updateOcclusion)]
pub fn update_occlusion(cache: &mut Cache, js_tile_id: &str, occluded: bool) {
	let id = *cache.tiles.id_map.get(js_tile_id).unwrap();
	cache.tiles.occluded[id] = occluded;
}

#[allow(dead_code)]
#[wasm_bindgen(js_name=buildCache)]
pub fn build_cache(js_walls: Vec<JsValue>, enable_height: bool) -> Cache {
	let mut occluded = vec![];
	let mut id_map = FxHashMap::default();
	let mut walls = Vec::with_capacity(js_walls.len());
	for wall in js_walls {
		let wall = JsWall::from(wall);
		let roof = if let Some(roof) = wall.roof() {
			let next_id = occluded.len();
			let id = id_map.entry(roof.id()).or_insert_with(|| {
				occluded.push(roof.occluded());
				next_id
			});
			Some(*id)
		} else {
			None
		};
		walls.push(WallBase::from_js(&wall, roof, enable_height));
	}
	Cache::build(walls, TileCache { occluded, id_map })
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
	pub type JsWallFlags;
	pub type JsWallHeight;
	pub type JsTile;

	#[wasm_bindgen(method, getter)]
	fn data(this: &JsWall) -> JsWallData;

	#[wasm_bindgen(method, getter)]
	fn roof(this: &JsWall) -> Option<JsTile>;

	#[wasm_bindgen(method, getter)]
	fn id(this: &JsTile) -> String;

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

	#[wasm_bindgen(method, getter)]
	fn flags(this: &JsWallData) -> JsWallFlags;

	#[wasm_bindgen(method, getter, js_name = "wallHeight")]
	fn wall_height(this: &JsWallFlags) -> Option<JsWallHeight>;

	#[wasm_bindgen(method, getter, js_name = "wallHeightTop")]
	fn top(this: &JsWallHeight) -> Option<f64>;

	#[wasm_bindgen(method, getter, js_name = "wallHeightBottom")]
	fn bottom(this: &JsWallHeight) -> Option<f64>;
}

impl WallBase {
	pub fn from_js(wall: &JsWall, roof: Option<TileId>, enable_height: bool) -> Self {
		let data = wall.data();
		let c = data.c();
		let height = if enable_height {
			data.flags().wall_height().into()
		} else {
			WallHeight::default()
		};
		Self::new(
			Point::new(c[0].round(), c[1].round()),
			Point::new(c[2].round(), c[3].round()),
			data.sense(),
			data.sound(),
			data.door(),
			data.ds(),
			data.dir().unwrap_or(WallDirection::BOTH),
			height,
			roof,
		)
	}
}

impl From<Option<JsWallHeight>> for WallHeight {
	fn from(height: Option<JsWallHeight>) -> Self {
		let height = height
			.map(|height| (height.top(), height.bottom()))
			.unwrap_or((None, None));
		let top = height.0.unwrap_or(WallHeight::default().top);
		let bottom = height.1.unwrap_or(WallHeight::default().bottom);
		Self { top, bottom }
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
