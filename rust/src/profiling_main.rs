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
	let data = RaycastingCall::deserialize_ascii85(&read_to_string("data.txt").unwrap());

	let mut sum = 0;
	let mut los = None;
	for _i in 0..1 {
		los = Some(compute_polygon(
			data.walls.clone(),
			data.origin,
			data.radius,
			data.distance,
			data.density,
			VisionAngle::from_rotation_and_angle(data.rotation, data.angle, data.origin),
			None,
		));
		sum += los.as_ref().unwrap().0.len();
	}

	println!("{:#?}, {}", los.unwrap().0.len(), sum);
}
