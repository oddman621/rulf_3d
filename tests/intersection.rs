

struct ConvexPolygon {
	vertices: Vec<glam::Vec2>,
	//segment_order: Vec<[usize; 2]>
}

impl ConvexPolygon {
	fn point_collision(&self, point: glam::Vec2) -> bool {
		let intersect_count = self.segment_loop().iter().filter(
			|seg| ConvexPolygon::intersection(seg[0], seg[1], point)).count();

		intersect_count & 1 == 1
	}

	// https://www.gamedevelopment.blog/collision-detection-circles-rectangles-and-polygons/
	fn intersection(v1: glam::Vec2, v2: glam::Vec2, point: glam::Vec2) -> bool {
		let point_between_y1_and_y2 = (v1.y <= point.y && v2.y > point.y) || ( v2.y <= point.y && v1.y > point.y );
		let px = (v2.x -v1.x) / (v2.y - v1.y) * (point.y - v1.y) + v1.x; // No matter whether vertex order is cw or ccw, don't know why...
		point_between_y1_and_y2 && point.x > px
	}

	fn segment_loop(&self) -> Vec<[glam::Vec2; 2]> {
		let mut segloop = Vec::<[glam::Vec2; 2]>::new();
		segloop.push([*self.vertices.last().unwrap(), *self.vertices.first().unwrap()]);
		for i in 0..self.vertices.len()-1 {
			segloop.push([self.vertices[i], self.vertices[i+1]]);
		}

		segloop
	}
}

#[test]
fn polygon_collision()
{
	let polygon_cw = ConvexPolygon {
		vertices: Vec::<glam::Vec2>::from([
			glam::vec2(1.0,0.0),
			glam::vec2(0.0,3.0),
			glam::vec2(3.0,6.0),
			glam::vec2(6.0,3.0),
			glam::vec2(5.0,0.0)
		])
	};

	assert!(polygon_cw.point_collision(glam::vec2(1.0, 1.0)));
	assert!(!polygon_cw.point_collision(glam::vec2(-1.0, 1.0)));
	assert!(!polygon_cw.point_collision(glam::vec2(7.0, 2.0)));
	
	let polygon_ccw = ConvexPolygon {
		vertices: Vec::<glam::Vec2>::from([
			glam::vec2(5.0,0.0),
			glam::vec2(6.0,3.0),
			glam::vec2(3.0,6.0),
			glam::vec2(0.0,3.0),
			glam::vec2(1.0,0.0)
		])
	};

	assert!(polygon_ccw.point_collision(glam::vec2(1.0, 1.0)));
	assert!(!polygon_ccw.point_collision(glam::vec2(-1.0, 1.0)));
	assert!(!polygon_ccw.point_collision(glam::vec2(7.0, 2.0)));
}