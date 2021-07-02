use std::fs::read_to_string;

use crate::{
	raycasting::{compute_polygon, Cache, VisionAngle},
	serialization::{deserialize_ascii85, TestCase},
};

fn run_test(filename: &str) {
	let test_root_dir = "tests/".to_owned();
	let test = deserialize_ascii85::<TestCase>(
		&read_to_string(test_root_dir + filename + ".ascii85").unwrap(),
	);
	let cache = Cache::build(test.call.walls);
	let (los, fov) = compute_polygon(
		&cache,
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

raytracing_test!(limited_vision_angle, "limited_vision_angle");
raytracing_test!(zero_width_walls, "zero_width_walls");
raytracing_test!(directional_walls_issue_4, "4-directional_walls");
raytracing_test!(t_junction_issue_5, "5-t_junction");
raytracing_test!(zero_length_walls_issue_6, "6-zero_length_walls");
raytracing_test!(
	overflow_wall_not_overflowing_in_fov_issue_14,
	"14-overflow_wall_not_overflowing_in_fov"
);
raytracing_test!(
	overflow_wall_both_points_seen_issue_15,
	"15-overflow_wall_both_points_seen"
);
raytracing_test!(
	overflow_wall_top_point_seen_issue_15,
	"15-overflow_wall_top_point_seen"
);
raytracing_test!(
	overflow_wall_bottom_point_seen_issue_15,
	"15-overflow_wall_bottom_point_seen"
);
raytracing_test!(
	overflow_wall_no_point_seen_wall_close_issue_15,
	"15-overflow_wall_no_point_seen_wall_close"
);
raytracing_test!(
	overflow_wall_no_point_seen_wall_far_issue_15,
	"15-overflow_wall_no_point_seen_wall_far"
);
