#![cfg(test)]
use aabb::{AABB, Bounded};
use nalgebra::{Point3, Vector3};
use ray::Ray;
use std::collections::HashSet;
use bounding_hierarchy::BoundingHierarchy;

/// Define some Bounded structure.
pub struct UnitBox {
    pub id: i32,
    pub pos: Point3<f32>,
}

/// `UnitBox`'s `AABB`s are unit `AABB`s centered on the given x-position.
impl Bounded for UnitBox {
    fn aabb(&self) -> AABB {
        let min = self.pos + Vector3::new(-0.5, -0.5, -0.5);
        let max = self.pos + Vector3::new(0.5, 0.5, 0.5);
        AABB::with_bounds(min, max)
    }
}

pub fn generate_aligned_boxes() -> Vec<UnitBox> {
    // Create 21 boxes along the x-axis
    let mut shapes = Vec::new();
    for x in -10..11 {
        shapes.push(UnitBox {
            id: x,
            pos: Point3::new(x as f32, 0.0, 0.0),
        });
    }
    shapes
}

pub fn test_traverse_aligned_boxes<T: BoundingHierarchy>(structure: &T, shapes: &Vec<UnitBox>) {
    {
        // Define a ray which traverses the x-axis from afar
        let position = Point3::new(-1000.0, 0.0, 0.0);
        let direction = Vector3::new(1.0, 0.0, 0.0);
        let ray = Ray::new(position, direction);

        // It shuold hit all shapes
        let hit_shapes = structure.traverse(&ray, &shapes);
        assert!(hit_shapes.len() == 21);
        let mut xs = HashSet::new();
        for shape in &hit_shapes {
            xs.insert(shape.id);
        }
        for x in -10..11 {
            assert!(xs.contains(&x));
        }
    }

    {
        // Define a ray which traverses the y-axis from afar
        let position = Point3::new(0.0, -1000.0, 0.0);
        let direction = Vector3::new(0.0, 1.0, 0.0);
        let ray = Ray::new(position, direction);

        // It should hit only one box
        let hit_shapes = structure.traverse(&ray, &shapes);
        assert!(hit_shapes.len() == 1);
        assert!(hit_shapes[0].id == 0);
    }

    {
        // Define a ray which intersects the x-axis diagonally
        let position = Point3::new(6.0, 0.5, 0.0);
        let direction = Vector3::new(-2.0, -1.0, 0.0);
        let ray = Ray::new(position, direction);

        // It should hit exactly three boxes
        let hit_shapes = structure.traverse(&ray, &shapes);
        assert!(hit_shapes.len() == 3);
        let mut xs = HashSet::new();
        for shape in &hit_shapes {
            xs.insert(shape.id);
        }
        assert!(xs.contains(&6));
        assert!(xs.contains(&5));
        assert!(xs.contains(&4));
    }
}

/// A triangle struct. Instance of a more complex `Bounded` primitive.
pub struct Triangle {
    pub a: Point3<f32>,
    pub b: Point3<f32>,
    pub c: Point3<f32>,
    aabb: AABB,
}

impl Triangle {
    fn new(a: Point3<f32>, b: Point3<f32>, c: Point3<f32>) -> Triangle {
        Triangle {
            a: a,
            b: b,
            c: c,
            aabb: AABB::empty().grow(&a).grow(&b).grow(&c),
        }
    }
}

impl Bounded for Triangle {
    fn aabb(&self) -> AABB {
        self.aabb
    }
}

/// Creates a unit size cube centered at `pos` and pushes the triangles to `shapes`
fn push_cube(pos: Point3<f32>, shapes: &mut Vec<Triangle>) {
    let top_front_right = pos + Vector3::new(0.5, 0.5, -0.5);
    let top_back_right = pos + Vector3::new(0.5, 0.5, 0.5);
    let top_back_left = pos + Vector3::new(-0.5, 0.5, 0.5);
    let top_front_left = pos + Vector3::new(-0.5, 0.5, -0.5);
    let bottom_front_right = pos + Vector3::new(0.5, -0.5, -0.5);
    let bottom_back_right = pos + Vector3::new(0.5, -0.5, 0.5);
    let bottom_back_left = pos + Vector3::new(-0.5, -0.5, 0.5);
    let bottom_front_left = pos + Vector3::new(-0.5, -0.5, -0.5);

    shapes.push(Triangle::new(top_back_right, top_front_right, top_front_left));
    shapes.push(Triangle::new(top_front_left, top_back_left, top_back_right));
    shapes.push(Triangle::new(bottom_front_left, bottom_front_right, bottom_back_right));
    shapes.push(Triangle::new(bottom_back_right, bottom_back_left, bottom_front_left));
    shapes.push(Triangle::new(top_back_left, top_front_left, bottom_front_left));
    shapes.push(Triangle::new(bottom_front_left, bottom_back_left, top_back_left));
    shapes.push(Triangle::new(bottom_front_right, top_front_right, top_back_right));
    shapes.push(Triangle::new(top_back_right, bottom_back_right, bottom_front_right));
    shapes.push(Triangle::new(top_front_left, top_front_right, bottom_front_right));
    shapes.push(Triangle::new(bottom_front_right, bottom_front_left, top_front_left));
    shapes.push(Triangle::new(bottom_back_right, top_back_right, top_back_left));
    shapes.push(Triangle::new(top_back_left, bottom_back_left, bottom_back_right));
}

/// Implementation of splitmix64.
/// For reference see: http://xoroshiro.di.unimi.it/splitmix64.c
fn splitmix64(x: &mut u64) -> u64 {
    *x = x.wrapping_add(0x9E3779B97F4A7C15u64);
    let mut z = *x;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9u64);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EBu64);
    z ^ (z >> 31)
}

fn next_point3(seed: &mut u64) -> Point3<f32> {
    let u = splitmix64(seed);
    let a = (u >> 48 & 0xFFFFFFFF) as i32 - 0xFFFF;
    let b = (u >> 48 & 0xFFFFFFFF) as i32 - 0xFFFF;
    let c = a ^ b.rotate_left(6);
    Point3::new(a as f32, b as f32, c as f32)
}

pub fn create_n_cubes(n: u64) -> Vec<Triangle> {
    let mut vec = Vec::new();
    let mut seed = 0;

    for _ in 0..n {
        push_cube(next_point3(&mut seed), &mut vec);
    }
    vec
}

pub fn create_ray(seed: &mut u64) -> Ray {
    let origin = next_point3(seed);
    let direction = next_point3(seed).to_vector();
    Ray::new(origin, direction)
}

/// Benchmark the construction of a BVH with 120,000 triangles.
pub fn bench_build_120k_triangles<T: BoundingHierarchy>(b: &mut ::test::Bencher) {
    let triangles = create_n_cubes(10_000);

    b.iter(|| {
        T::build(&triangles);
    });
}

#[bench]
/// Benchmark intersecting 120,000 triangles directly.
fn bench_intersect_120k_triangles_list(b: &mut ::test::Bencher) {
    let triangles = create_n_cubes(10_000);
    let mut seed = 0;

    b.iter(|| {
        let ray = create_ray(&mut seed);

        // Iterate over the list of triangles
        for triangle in &triangles {
            ray.intersects_triangle(&triangle.a, &triangle.b, &triangle.c);
        }
    });
}

#[bench]
/// Benchmark intersecting 120,000 triangles with preceeding `AABB` tests.
fn bench_intersect_120k_triangles_list_aabb(b: &mut ::test::Bencher) {
    let triangles = create_n_cubes(10_000);
    let mut seed = 0;

    b.iter(|| {
        let ray = create_ray(&mut seed);

        // Iterate over the list of triangles
        for triangle in &triangles {
            // First test whether the ray intersects the AABB of the triangle
            if ray.intersects_aabb(&triangle.aabb()) {
                ray.intersects_triangle(&triangle.a, &triangle.b, &triangle.c);
            }
        }
    });
}

pub fn bench_intersect_120k_triangles<T: BoundingHierarchy>(b: &mut ::test::Bencher) {
    let triangles = create_n_cubes(10_000);
    let mut seed = 0;
    let structure = T::build(&triangles);

    b.iter(|| {
        let ray = create_ray(&mut seed);

        // Traverse the BIH recursively
        let hits = structure.traverse(&ray, &triangles);

        // Traverse the resulting list of positive AABB tests
        for triangle in &hits {
            ray.intersects_triangle(&triangle.a, &triangle.b, &triangle.c);
        }
    });
}
