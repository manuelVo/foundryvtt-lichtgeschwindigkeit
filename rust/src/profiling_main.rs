mod geometry;
mod ptr_indexed_hash_set;
mod raycasting;
mod serialization;
#[cfg(test)]
mod tests;

use raycasting::*;
use serialization::*;

use std::fs::read_to_string;

fn main() {
	let data = deserialize_ascii85::<RaycastingCall>(&read_to_string("data.txt").unwrap());
	let cache = Cache::build(data.walls, TileCache::from_roofs(data.roofs));
	let mut sum = 0;
	let mut los = None;
	for _i in 0..1 {
		los = Some(compute_polygon(
			&cache,
			data.origin,
			data.height,
			data.radius,
			data.distance,
			data.density,
			VisionAngle::from_rotation_and_angle(data.rotation, data.angle, data.origin),
			PolygonType::SIGHT,
			None,
		));
		sum += los.as_ref().unwrap().0.len();
	}

	println!("{:#?}, {}", los.unwrap().0.len(), sum);
}
