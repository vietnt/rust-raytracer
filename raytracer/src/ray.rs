use crate::materials::Material;
use crate::point3d::Point3D;

#[cfg(test)]
use assert_approx_eq::assert_approx_eq;

#[derive(Debug, Clone, Copy)]
pub struct Ray {
    pub origin: Point3D,
    pub direction: Point3D,
}

impl Ray {
    pub fn new(origin: Point3D, direction: Point3D) -> Ray {
        Ray { origin, direction }
    }

    pub fn at(&self, t: f32) -> Point3D {
        self.origin + self.direction * t
    }
}

pub struct HitRecord<'material> {
    pub t: f32,
    pub point: Point3D,
    pub normal: Point3D,
    pub front_face: bool,
    pub material: &'material Material,
    pub u: f32,
    pub v: f32,
}

impl HitRecord<'_> {
    pub fn new(material: &Material) -> HitRecord {
        HitRecord {
            t: 0.0,
            point: Point3D::default(),
            normal: Point3D::default(),
            front_face: false,
            material,
            u: 0.0,
            v: 0.0,
        }
    }
}

pub trait Hittable {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord>;
}

#[test]
fn test_ray() {
    let p = Point3D::new(0.1, 0.2, 0.3);
    let q = Point3D::new(0.2, 0.3, 0.4);

    let r = Ray::new(p, q);

    assert_approx_eq!(r.origin.x, 0.1);
    assert_approx_eq!(r.origin.y, 0.2);
    assert_approx_eq!(r.origin.z, 0.3);
    assert_approx_eq!(r.direction.x, 0.2);
    assert_approx_eq!(r.direction.y, 0.3);
    assert_approx_eq!(r.direction.z, 0.4);
}

#[test]
fn test_ray_at() {
    let p = Point3D::new(0.0, 0.0, 0.0);
    let q = Point3D::new(1.0, 2.0, 3.0);

    let r = Ray::new(p, q);
    let s = r.at(0.5);

    assert_approx_eq!(s.x, 0.5);
    assert_approx_eq!(s.y, 1.0);
    assert_approx_eq!(s.z, 1.5);
}
