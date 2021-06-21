use std::fs::read_to_string;

use crate::{
	raycasting::{compute_polygon, VisionAngle},
	serialization::{deserialize_ascii85, TestCase},
};

fn run_test(filename: &str) {
	let test_root_dir = "tests/".to_owned();
	let test = deserialize_ascii85::<TestCase>(
		&read_to_string(test_root_dir + filename + ".txt").unwrap(),
	);
	let (los, fov) = compute_polygon(
		test.call.walls,
		test.call.origin,
		test.call.radius,
		test.call.distance,
		test.call.density,
		VisionAngle::from_rotation_and_angle(test.call.rotation, test.call.angle, test.call.origin),
		None,
	);
	let e = 0.1;
	assert_eq!(test.los.len(), los.len());
	for (expected, actual) in test.los.iter().zip(los) {
		assert!(expected.distance_to(&actual) < e);
	}

	assert_eq!(test.fov.len(), fov.len());
	for (expected, actual) in test.fov.iter().zip(fov) {
		assert!(expected.distance_to(&actual) < e);
	}
}

macro_rules! raytracing_test (
	($name:ident,$path:expr) => {
		#[test]
		fn $name() {
			run_test($path);
		}
	};
);

raytracing_test!(zero_width_walls, "zero_width_walls");
raytracing_test!(t_junction_issue_5, "5-t_junction");
raytracing_test!(zero_length_walls_issue_6, "6-zero_length_walls");
