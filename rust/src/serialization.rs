use wasm_bindgen::prelude::*;

use crate::geometry::{JsPoint, Line, Point};
use crate::raycasting::*;
use js_sys::{Array, Object};
use nom::bytes::complete::take;
use nom::IResult;
use std::convert::{TryFrom, TryInto};
use std::fmt::Debug;
use std::mem::size_of;
use yazi::{compress, decompress, CompressionLevel, Format};

const CURRENT_VERSION: u8 = 3;

pub trait Serialize {
	fn serialize(&self) -> Vec<u8>;
	fn deserialize(input: &[u8], version: u8) -> IResult<&[u8], Self>
	where
		Self: Sized;
}

pub trait SerializeByte {
	fn serialize_byte(&self) -> u8;
	fn deserialize_byte(input: &[u8]) -> IResult<&[u8], Self>
	where
		Self: Sized;
}

impl<T: SerializeByte> Serialize for T
where
	Self: Sized,
{
	fn serialize(&self) -> Vec<u8> {
		vec![self.serialize_byte()]
	}

	fn deserialize(input: &[u8], _version: u8) -> IResult<&[u8], Self> {
		Self::deserialize_byte(input)
	}
}

macro_rules! ImplSerializeByteForEnum (
	($name:ident) => {
		impl SerializeByte for $name {
			fn serialize_byte(&self) -> u8 {
				// TODO Try into would be better here
				*self as u8
			}

			fn deserialize_byte(input: &[u8]) -> IResult<&[u8], Self>
			where
				Self: Sized + TryFrom<usize>,
				<Self as TryFrom<usize>>::Error: Debug,
			{
				let (input, byte) = take(1usize)(input)?;
				Ok((input, (byte[0] as usize).try_into().unwrap()))
			}
		}
	};
);

impl Serialize for f64 {
	fn serialize(&self) -> Vec<u8> {
		self.to_be_bytes().into()
	}

	fn deserialize(input: &[u8], _version: u8) -> IResult<&[u8], Self> {
		let (input, representation) = take(size_of::<Self>())(input)?;
		Ok((
			input,
			Self::from_be_bytes(representation.try_into().unwrap()),
		))
	}
}

impl Serialize for u32 {
	fn serialize(&self) -> Vec<u8> {
		self.to_be_bytes().into()
	}

	fn deserialize(input: &[u8], _version: u8) -> IResult<&[u8], Self> {
		let (input, representation) = take(size_of::<Self>())(input)?;
		Ok((
			input,
			Self::from_be_bytes(representation.try_into().unwrap()),
		))
	}
}

impl Serialize for usize {
	fn serialize(&self) -> Vec<u8> {
		u32::try_from(*self).unwrap().serialize()
	}

	fn deserialize(input: &[u8], version: u8) -> IResult<&[u8], Self> {
		let (input, value) = u32::deserialize(input, version)?;
		Ok((input, value.try_into().unwrap()))
	}
}

impl Serialize for Point {
	fn serialize(&self) -> Vec<u8> {
		let mut data = Vec::with_capacity(size_of::<Self>());
		data.append(&mut self.x.serialize());
		data.append(&mut self.y.serialize());
		data
	}

	fn deserialize(input: &[u8], version: u8) -> IResult<&[u8], Self> {
		let (input, x) = f64::deserialize(input, version)?;
		let (input, y) = f64::deserialize(input, version)?;
		Ok((input, Self { x, y }))
	}
}

impl Serialize for WallHeight {
	fn serialize(&self) -> Vec<u8> {
		let mut data = Vec::with_capacity(size_of::<Self>());
		data.append(&mut self.top.serialize());
		data.append(&mut self.bottom.serialize());
		data
	}

	fn deserialize(input: &[u8], version: u8) -> IResult<&[u8], Self> {
		let (input, top) = f64::deserialize(input, version)?;
		let (input, bottom) = f64::deserialize(input, version)?;
		Ok((input, Self { top, bottom }))
	}
}

impl<T: Serialize> Serialize for Vec<T> {
	fn serialize(&self) -> Vec<u8> {
		let mut data = Vec::new();
		data.append(&mut u32::try_from(self.len()).unwrap().to_be_bytes().into());
		for wall in self {
			data.append(&mut wall.serialize());
		}
		data
	}

	fn deserialize(input: &[u8], version: u8) -> IResult<&[u8], Self> {
		let (input, len) = take(size_of::<u32>())(input)?;
		let len = u32::from_be_bytes(len.try_into().unwrap()) as usize;
		let mut vector = Vec::with_capacity(len);
		let mut input = input;
		for _ in 0..len {
			let (new_input, entry) = T::deserialize(input, version)?;
			input = new_input;
			vector.push(entry);
		}
		Ok((input, vector))
	}
}

impl<T: Serialize> Serialize for Option<T> {
	fn serialize(&self) -> Vec<u8> {
		let mut data = vec![];
		data.append(&mut self.is_some().serialize());
		if let Some(value) = self {
			data.append(&mut value.serialize());
		}
		data
	}

	fn deserialize(input: &[u8], version: u8) -> IResult<&[u8], Self> {
		let (input, is_some) = bool::deserialize(input, version)?;
		if !is_some {
			return Ok((input, None));
		}
		let (input, value) = T::deserialize(input, version)?;
		Ok((input, Some(value)))
	}
}

ImplSerializeByteForEnum!(WallSenseType);
ImplSerializeByteForEnum!(DoorType);
ImplSerializeByteForEnum!(DoorState);
ImplSerializeByteForEnum!(WallDirection);
ImplSerializeByteForEnum!(PolygonType);

impl SerializeByte for bool {
	fn serialize_byte(&self) -> u8 {
		if *self {
			1
		} else {
			0
		}
	}

	fn deserialize_byte(input: &[u8]) -> IResult<&[u8], Self> {
		let (input, data) = take(1usize)(input)?;
		let data = match data[0] {
			0 => false,
			1 => true,
			_ => unreachable!(),
		};
		Ok((input, data))
	}
}

pub struct TestCase {
	pub call: RaycastingCall,
	pub los: Vec<Point>,
	pub fov: Vec<Point>,
}

impl Serialize for TestCase {
	fn serialize(&self) -> Vec<u8> {
		let mut data = Vec::new();
		data.append(&mut self.call.serialize());
		data.append(&mut self.los.serialize());
		data.append(&mut self.fov.serialize());
		data
	}

	fn deserialize(input: &[u8], version: u8) -> IResult<&[u8], Self> {
		let (input, call) = RaycastingCall::deserialize(input, version)?;
		let (input, los) = Vec::deserialize(input, version)?;
		let (input, fov) = Vec::deserialize(input, version)?;
		Ok((input, Self { call, los, fov }))
	}
}

pub fn serialize_ascii85<T: Serialize>(data: T) -> String {
	let version = CURRENT_VERSION;
	let data = data.serialize();
	let mut compressed = compress(&data, Format::Zlib, CompressionLevel::BestSize).unwrap();
	compressed.insert(0, version);
	ascii85::encode(&compressed)
}

pub fn deserialize_ascii85<T: Serialize>(input: &str) -> T {
	let input = ascii85::decode(input).unwrap();
	let version = input[0];
	if version > CURRENT_VERSION {
		panic!("Data stream has a wrong version number.");
	}
	let input = &input[1..];
	let (input, _) = &decompress(input, Format::Zlib).unwrap();
	T::deserialize(&input, version).unwrap().1
}

pub struct RaycastingCall {
	pub walls: Vec<WallBase>,
	pub roofs: Vec<bool>,
	pub origin: Point,
	pub height: f64,
	pub radius: f64,
	pub distance: f64,
	pub density: f64,
	pub angle: f64,
	pub rotation: f64,
	pub polygon_type: PolygonType,
}

impl From<RaycastingCall> for Object {
	fn from(value: RaycastingCall) -> Self {
		use js_sys::Reflect::set;
		let result = Object::new();
		set(
			&result,
			&JsValue::from_str("walls"),
			&value
				.walls
				.into_iter()
				.map::<JsValue, _>(|wall| wall.into())
				.collect::<Array>(),
		)
		.unwrap();
		set(&result, &JsValue::from_str("origin"), &value.origin.into()).unwrap();
		set(&result, &JsValue::from_str("height"), &value.height.into()).unwrap();
		set(&result, &JsValue::from_str("radius"), &value.radius.into()).unwrap();
		set(
			&result,
			&JsValue::from_str("distance"),
			&value.distance.into(),
		)
		.unwrap();
		set(
			&result,
			&JsValue::from_str("density"),
			&value.density.into(),
		)
		.unwrap();
		set(&result, &JsValue::from_str("angle"), &value.angle.into()).unwrap();
		set(
			&result,
			&JsValue::from_str("rotation"),
			&value.rotation.into(),
		)
		.unwrap();
		result
	}
}

impl Serialize for RaycastingCall {
	fn serialize(&self) -> Vec<u8> {
		let mut data = Vec::new();
		data.append(&mut self.walls.serialize());
		data.append(&mut self.roofs.serialize());
		data.append(&mut self.origin.serialize());
		data.append(&mut self.height.serialize());
		data.append(&mut self.radius.serialize());
		data.append(&mut self.distance.serialize());
		data.append(&mut self.density.serialize());
		data.append(&mut self.angle.serialize());
		data.append(&mut self.rotation.serialize());
		data.append(&mut self.polygon_type.serialize());
		data
	}

	fn deserialize(input: &[u8], version: u8) -> IResult<&[u8], Self> {
		let (input, walls) = Vec::deserialize(input, version)?;
		let (input, roofs) = if version >= 3 {
			Vec::deserialize(input, version)?
		} else {
			(input, vec![])
		};
		let (input, origin) = Point::deserialize(input, version)?;
		let (input, height) = if version >= 2 {
			f64::deserialize(input, version)?
		} else {
			(input, 0.0)
		};
		let (input, radius) = f64::deserialize(input, version)?;
		let (input, distance) = f64::deserialize(input, version)?;
		let (input, density) = f64::deserialize(input, version)?;
		let (input, angle) = f64::deserialize(input, version)?;
		let (input, rotation) = f64::deserialize(input, version)?;
		let (input, polygon_type) = if version >= 3 {
			PolygonType::deserialize(input, version)?
		} else {
			(input, PolygonType::SIGHT)
		};
		Ok((
			input,
			Self {
				walls,
				roofs,
				origin,
				height,
				radius,
				distance,
				density,
				angle,
				rotation,
				polygon_type,
			},
		))
	}
}

impl Serialize for WallBase {
	fn serialize(&self) -> Vec<u8> {
		let mut data = Vec::with_capacity(size_of::<Self>());
		data.append(&mut self.p1.serialize());
		data.append(&mut self.p2.serialize());
		data.append(&mut self.movement.serialize());
		data.append(&mut self.sense.serialize());
		data.append(&mut self.sound.serialize());
		data.append(&mut self.door.serialize());
		data.append(&mut self.ds.serialize());
		data.append(&mut self.dir.serialize());
		data.append(&mut self.height.serialize());
		data.append(&mut self.roof.serialize());
		data
	}

	fn deserialize(input: &[u8], version: u8) -> IResult<&[u8], Self> {
		let (input, p1) = Point::deserialize(input, version)?;
		let (input, p2) = Point::deserialize(input, version)?;
		let line = Line::from_points(p1, p2);
		let (input, movement) = if version >= 3 {
			WallSenseType::deserialize(input, version)?
		} else {
			(input, WallSenseType::NORMAL)
		};
		let (input, sense) = WallSenseType::deserialize(input, version)?;
		let (input, sound) = if version >= 3 {
			WallSenseType::deserialize(input, version)?
		} else {
			(input, sense)
		};
		let (input, door) = DoorType::deserialize(input, version)?;
		let (input, ds) = DoorState::deserialize(input, version)?;
		let (input, dir) = WallDirection::deserialize(input, version)?;
		let (input, height) = if version >= 1 {
			WallHeight::deserialize(input, version)?
		} else {
			(input, WallHeight::default())
		};
		let (input, roof) = if version >= 3 {
			Option::deserialize(input, version)?
		} else {
			(input, None)
		};
		Ok((
			input,
			Self {
				p1,
				p2,
				line,
				movement,
				sense,
				sound,
				door,
				ds,
				dir,
				height,
				roof,
			},
		))
	}
}

#[wasm_bindgen(js_name=serializeData)]
#[allow(dead_code)]
pub fn js_serialize_data(
	cache: &Cache,
	origin: JsPoint,
	height: f64,
	radius: f64,
	distance: f64,
	density: f64,
	angle: f64,
	rotation: f64,
	polygon_type: &str,
) -> String {
	let polygon_type = PolygonType::from(polygon_type);
	let data = RaycastingCall {
		walls: cache.walls.clone(),
		roofs: cache.tiles.occluded.clone(),
		origin: Point::from(&origin.into()),
		height,
		radius,
		distance,
		density,
		angle,
		rotation,
		polygon_type,
	};
	serialize_ascii85(data)
}

#[wasm_bindgen(js_name=deserializeData)]
#[allow(dead_code)]
pub fn js_deserialize_data(str: &str) -> Object {
	let data = deserialize_ascii85::<RaycastingCall>(str);
	data.into()
}

#[wasm_bindgen(js_name=generateTest)]
#[allow(dead_code)]
pub fn js_generate_test(str: &str) -> String {
	let data = deserialize_ascii85::<RaycastingCall>(str);
	let cache = Cache::build(
		data.walls.clone(),
		TileCache::from_roofs(data.roofs.clone()),
	);
	let (los, fov) = compute_polygon(
		&cache,
		data.origin,
		data.height,
		data.radius,
		data.distance,
		data.density,
		VisionAngle::from_rotation_and_angle(data.rotation, data.angle, data.origin),
		data.polygon_type,
		None,
	);
	serialize_ascii85(TestCase {
		call: data,
		los,
		fov,
	})
}
