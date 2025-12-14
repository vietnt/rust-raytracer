use materials::Material;
use mesh::{Mesh, Triangle};

pub mod camera;
pub mod config;
pub mod materials;
pub mod mesh;
pub mod point3d;
pub mod ray;
pub mod raytracer;
pub mod sphere;
extern crate nalgebra_glm as glm;

pub fn polygon(vs: &[glm::Vec3], material: &Material) -> Mesh {
    let mut triangles = Vec::new();
    for i in 1..vs.len() - 1 {
        triangles.push(Triangle::from_vertices(vs[0], vs[i], vs[i + 1]));
    }
    Mesh::new(triangles, material.clone())
}
