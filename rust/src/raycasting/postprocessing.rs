use crate::geometry::{Circle, Line, Point};
use crate::raycasting::types::FovPoint;
use std::f64::consts::PI;

pub fn calculate_fov(
	origin: Point,
	radius: f64,
	los_points: &Vec<FovPoint>,
	start_gap_fov: bool,
) -> Vec<FovPoint> {
	let fov = Circle {
		center: origin,
		radius,
	};
	let mut fov_points = Vec::new();
	for i in 0..los_points.len() {
		let los_point = los_points[i];
		let distance = origin.distance_to(&los_point.point);
		if distance < radius {
			if i == 0 {
				fov_points.push(los_point);
			} else {
				let previous_los = los_points[i - 1];
				let previous_fov_gap = fov_points
					.last()
					.map(|previous| previous.gap)
					.unwrap_or(start_gap_fov);
				if previous_fov_gap && !previous_los.gap {
					if previous_los.angle == los_point.angle {
						let point = Point {
							x: fov.center.x - los_point.angle.cos() * radius,
							y: fov.center.y - los_point.angle.sin() * radius,
						};
						fov_points.push(FovPoint {
							point,
							angle: los_point.angle,
							gap: false,
						});
					} else {
						let line = Line::from_points(previous_los.point, los_point.point);
						if let Some(fov_intersections) = fov.intersections(&line) {
							let relevant_intersection;
							// TODO is_smaller_relative
							if fov_intersections.0.angle > previous_los.angle
								&& fov_intersections.0.angle < los_point.angle
							{
								relevant_intersection = fov_intersections.0;
							} else {
								relevant_intersection = fov_intersections.1;
							}
							fov_points.push(FovPoint {
								point: relevant_intersection.point,
								angle: relevant_intersection.angle,
								gap: false,
							});
						}
					}
				}
				fov_points.push(los_point);

				if i == los_points.len() - 1 {
					if start_gap_fov && !los_point.gap {
						let line =
							Line::from_points(los_point.point, los_points.first().unwrap().point);
						let intersections = fov.intersections(&line).unwrap();
						let exit_intersection;
						if intersections.0.angle > intersections.1.angle {
							exit_intersection = intersections.0;
						} else {
							exit_intersection = intersections.1;
						}
						fov_points.push(FovPoint {
							point: exit_intersection.point,
							angle: exit_intersection.angle,
							gap: false,
						});
					}
				}
			}
		} else {
			let previous_fov_gap = fov_points
				.last()
				.map(|previous| previous.gap)
				.unwrap_or(start_gap_fov);
			if !previous_fov_gap {
				if i > 0 {
					let hidden_point = los_point;
					let point_before_hidden = los_points[i - 1];
					if point_before_hidden.angle == hidden_point.angle {
						let point = Point {
							x: fov.center.x - hidden_point.angle.cos() * radius,
							y: fov.center.y - hidden_point.angle.sin() * radius,
						};
						fov_points.push(FovPoint {
							point,
							angle: los_point.angle,
							gap: true,
						});
					} else {
						let line = Line::from_points(point_before_hidden.point, hidden_point.point);
						if let Some(fov_intersections) = fov.intersections(&line) {
							let relevant_intersection;
							// TODO is_smaller_relative
							if fov_intersections.0.angle > point_before_hidden.angle
								&& fov_intersections.0.angle < hidden_point.angle
							{
								relevant_intersection = fov_intersections.0;
							} else {
								relevant_intersection = fov_intersections.1;
							}
							fov_points.push(FovPoint {
								point: relevant_intersection.point,
								angle: relevant_intersection.angle,
								gap: true,
							});
						}
						if !start_gap_fov && i == los_points.len() - 1 {
							let next_los = los_points.first().unwrap();
							let line = Line::from_points(los_point.point, next_los.point);
							let intersections = fov.intersections(&line).unwrap();
							let entry;
							// The wall is to the right of the token, so the angles are inverted
							if intersections.0.angle > intersections.1.angle {
								entry = intersections.0;
							} else {
								entry = intersections.1;
							}
							fov_points.push(FovPoint {
								point: entry.point,
								angle: entry.angle,
								gap: false,
							});
						}
					}
				} else {
					let previous_los = los_points.last().unwrap();
					let line = Line::from_points(previous_los.point, los_point.point);
					let intersections = fov.intersections(&line).unwrap();
					let exit;
					// The wall is to the right of the token, so the angles are inverted
					if intersections.0.angle > intersections.1.angle {
						exit = intersections.1;
					} else {
						exit = intersections.0;
					}
					fov_points.push(FovPoint {
						point: exit.point,
						angle: exit.angle,
						gap: true,
					});
				}
			} else {
				// TODO Handle i == 0
				if i > 0 {
					let previous_los = los_points[i - 1];
					if !previous_los.gap {
						let line = Line::from_points(previous_los.point, los_point.point);
						if let Some(intersections) = fov.intersections(&line) {
							// TODO Is smaller relative?
							if intersections.0.angle > previous_los.angle
								&& intersections.1.angle > previous_los.angle
								&& intersections.0.angle < los_point.angle
								&& intersections.1.angle < los_point.angle
							{
								let (entry, exit);
								// TODO Is smaller relative?
								if intersections.0.angle < intersections.1.angle {
									entry = intersections.0;
									exit = intersections.1;
								} else {
									entry = intersections.1;
									exit = intersections.0;
								}
								fov_points.push(FovPoint {
									point: entry.point,
									angle: entry.angle,
									gap: false,
								});
								fov_points.push(FovPoint {
									point: exit.point,
									angle: exit.angle,
									gap: true,
								});
							}
						}
					}
					if !start_gap_fov && i == los_points.len() - 1 {
						let next_los = los_points.first().unwrap();
						let line = Line::from_points(los_point.point, next_los.point);
						let intersections = fov.intersections(&line).unwrap();
						let entry;
						// The wall is to the right of the token, so the angles are inverted
						if intersections.0.angle > intersections.1.angle {
							entry = intersections.0;
						} else {
							entry = intersections.1;
						}
						fov_points.push(FovPoint {
							point: entry.point,
							angle: entry.angle,
							gap: false,
						});
					}
				}
			}
		}
	}
	fov_points
}

pub fn fill_gaps(
	points: &mut Vec<FovPoint>,
	start_gap: bool,
	origin: Point,
	radius: f64,
	radial_density: f64,
) -> Vec<Point> {
	let mut output = Vec::new();

	if points.len() == 0 {
		let mut a = -PI;
		while a < PI {
			output.push(Point::new(
				origin.x - (a.cos() * radius),
				origin.y - (a.sin() * radius),
			));
			a += radial_density;
		}
	} else {
		if points.last().unwrap().point != origin {
			points.last_mut().unwrap().gap = start_gap;
		}
		for i in 0..points.len() {
			// TODO This produces a quite big assembly. Think of something faster
			let (lower, upper) = points.split_at(i);
			let (current, upper) = upper.split_at(1);
			let current = current.first().unwrap();
			let previous;
			if i == 0 {
				previous = upper.last().unwrap();
			} else {
				previous = lower.last().unwrap();
			}
			if previous.gap {
				let mut previous_angle = previous.angle;
				if previous_angle > current.angle {
					previous_angle -= 2.0 * PI;
				}
				let mut a = previous_angle;
				let first_filler =
					Point::new(origin.x - (a.cos() * radius), origin.y - (a.sin() * radius));
				if !first_filler.is_same_as(&previous.point) {
					output.push(first_filler);
				}
				a += radial_density;
				while a < current.angle {
					output.push(Point::new(
						origin.x - (a.cos() * radius),
						origin.y - (a.sin() * radius),
					));
					a += radial_density;
				}
				let last_filler = Point::new(
					origin.x - (current.angle.cos() * radius),
					origin.y - (current.angle.sin() * radius),
				);
				if !last_filler.is_same_as(&current.point) {
					output.push(last_filler);
				}
			}
			output.push(current.point);
		}
	}

	output
}
