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

pub trait Serialize {
	fn serialize(&self) -> Vec<u8>;
	fn deserialize(input: &[u8]) -> IResult<&[u8], Self>
	where
		Self: Sized;
}

trait SerializeByte {
	fn serialize(&self) -> u8;
	fn deserialize(input: &[u8]) -> IResult<&[u8], Self>
	where
		Self: Sized + TryFrom<usize>,
		<Self as TryFrom<usize>>::Error: Debug,
	{
		let (input, byte) = take(1usize)(input)?;
		Ok((input, (byte[0] as usize).try_into().unwrap()))
	}
}

impl Serialize for f64 {
	fn serialize(&self) -> Vec<u8> {
		self.to_be_bytes().into()
	}

	fn deserialize(input: &[u8]) -> IResult<&[u8], Self> {
		let (input, representation) = take(size_of::<Self>())(input)?;
		Ok((
			input,
			Self::from_be_bytes(representation.try_into().unwrap()),
		))
	}
}

impl Serialize for Point {
	fn serialize(&self) -> Vec<u8> {
		let mut data = Vec::with_capacity(size_of::<Self>());
		data.append(&mut self.x.serialize());
		data.append(&mut self.y.serialize());
		data
	}

	fn deserialize(input: &[u8]) -> IResult<&[u8], Self> {
		let (input, x) = f64::deserialize(input)?;
		let (input, y) = f64::deserialize(input)?;
		Ok((input, Self { x, y }))
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

	fn deserialize(input: &[u8]) -> IResult<&[u8], Self> {
		let (input, len) = take(size_of::<u32>())(input)?;
		let len = u32::from_be_bytes(len.try_into().unwrap()) as usize;
		let mut vector = Vec::with_capacity(len);
		let mut input = input;
		for _ in 0..len {
			let (new_input, entry) = T::deserialize(input)?;
			input = new_input;
			vector.push(entry);
		}
		Ok((input, vector))
	}
}

impl SerializeByte for WallSenseType {
	fn serialize(&self) -> u8 {
		return *self as u8;
	}
}

impl SerializeByte for DoorType {
	fn serialize(&self) -> u8 {
		return *self as u8;
	}
}

impl SerializeByte for DoorState {
	fn serialize(&self) -> u8 {
		return *self as u8;
	}
}

impl SerializeByte for WallDirection {
	fn serialize(&self) -> u8 {
		return *self as u8;
	}
}

impl SerializeByte for PolygonType {
	fn serialize(&self) -> u8 {
		*self as u8
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

	fn deserialize(input: &[u8]) -> IResult<&[u8], Self> {
		let (input, call) = RaycastingCall::deserialize(input)?;
		let (input, los) = Vec::deserialize(input)?;
		let (input, fov) = Vec::deserialize(input)?;
		Ok((input, Self { call, los, fov }))
	}
}

pub fn serialize_ascii85<T: Serialize>(data: T) -> String {
	let data = data.serialize();
	let mut compressed = compress(&data, Format::Zlib, CompressionLevel::BestSize).unwrap();
	compressed.insert(0, 0u8);
	ascii85::encode(&compressed)
}

pub fn deserialize_ascii85<T: Serialize>(input: &str) -> T {
	let input = ascii85::decode(input).unwrap();
	if input[0] != 0 {
		panic!("Data stream has a wrong version number.");
	}
	let input = &input[1..];
	let (input, _) = &decompress(input, Format::Zlib).unwrap();
	T::deserialize(&input).unwrap().1
}

pub struct RaycastingCall {
	pub walls: Vec<WallBase>,
	pub origin: Point,
	pub radius: f64,
	pub distance: f64,
	pub density: f64,
	pub angle: f64,
	pub rotation: f64,
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
		data.append(&mut self.origin.serialize());
		data.append(&mut self.radius.serialize());
		data.append(&mut self.distance.serialize());
		data.append(&mut self.density.serialize());
		data.append(&mut self.angle.serialize());
		data.append(&mut self.rotation.serialize());
		data
	}

	fn deserialize(input: &[u8]) -> IResult<&[u8], Self> {
		let (input, walls) = Vec::deserialize(input)?;
		let (input, origin) = Point::deserialize(input)?;
		let (input, radius) = f64::deserialize(input)?;
		let (input, distance) = f64::deserialize(input)?;
		let (input, density) = f64::deserialize(input)?;
		let (input, angle) = f64::deserialize(input)?;
		let (input, rotation) = f64::deserialize(input)?;
		Ok((
			input,
			Self {
				walls,
				origin,
				radius,
				distance,
				density,
				angle,
				rotation,
			},
		))
	}
}

impl Serialize for WallBase {
	fn serialize(&self) -> Vec<u8> {
		let mut data = Vec::with_capacity(size_of::<Self>());
		data.append(&mut self.p1.serialize());
		data.append(&mut self.p2.serialize());
		data.push(self.sense.serialize());
		data.push(self.door.serialize());
		data.push(self.ds.serialize());
		data.push(self.dir.serialize());
		data
	}

	fn deserialize(input: &[u8]) -> IResult<&[u8], Self> {
		let (input, p1) = Point::deserialize(input)?;
		let (input, p2) = Point::deserialize(input)?;
		let line = Line::from_points(p1, p2);
		let (input, sense) = WallSenseType::deserialize(input)?;
		let (input, door) = DoorType::deserialize(input)?;
		let (input, ds) = DoorState::deserialize(input)?;
		let (input, dir) = WallDirection::deserialize(input)?;
		Ok((
			input,
			Self {
				p1,
				p2,
				line,
				sense,
				door,
				ds,
				dir,
			},
		))
	}
}

#[wasm_bindgen(js_name=serializeData)]
#[allow(dead_code)]
pub fn js_serialize_data(
	cache: &Cache,
	origin: JsPoint,
	radius: f64,
	distance: f64,
	density: f64,
	angle: f64,
	rotation: f64,
) -> String {
	let data = RaycastingCall {
		walls: cache.walls.clone(),
		origin: Point::from(&origin.into()),
		radius,
		distance,
		density,
		angle,
		rotation,
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
	let cache = Cache::build(data.walls.clone());
	let (los, fov) = compute_polygon(
		&cache,
		data.origin,
		data.radius,
		data.distance,
		data.density,
		VisionAngle::from_rotation_and_angle(data.rotation, data.angle, data.origin),
		None,
	);
	serialize_ascii85(TestCase {
		call: data,
		los,
		fov,
	})
}
