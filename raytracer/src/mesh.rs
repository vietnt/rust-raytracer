use bvh::{
    aabb::{Aabb, Bounded},
    bounding_hierarchy::BHShape,
    bvh::Bvh,
};

use crate::{
    materials::Material,
    ray::{HitRecord, Hittable, Ray},
};

#[derive(Clone)]
pub struct Cube {
    pub center: glm::Vec3,
    pub size: glm::Vec3,
    pub material: Material,
}

#[derive(Copy, Clone)]
pub struct Triangle {
    pub v1: glm::Vec3,
    pub v2: glm::Vec3,
    pub v3: glm::Vec3,
    pub n1: glm::Vec3,
    pub n2: glm::Vec3,
    pub n3: glm::Vec3,
    node_index: usize,
}

#[derive(Clone)]
pub struct Mesh {
    bvh: Bvh<f32, 3>,
    triangles: Vec<Triangle>,
    material: Material,
    aabb: Aabb<f32, 3>,
}

impl Mesh {
    pub fn new(triangles: Vec<Triangle>, material: Material) -> Self {
        let mut triangles = triangles;
        let mut aabb = triangles[0].aabb();
        for t in triangles.iter() {
            aabb.join_mut(&t.aabb());
        }
        let bvh = Bvh::build(&mut triangles);
        Self {
            bvh,
            triangles,
            material,
            aabb,
        }
    }
}

impl Hittable for Mesh {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let r = bvh::ray::Ray::new(ray.origin.into(), ray.direction);
        let mut record = HitRecord::new(&self.material);
        let mut closest_so_far = t_max;
        for t in &self.bvh.traverse(&r, &self.triangles) {
            if t.hit(ray, t_min, closest_so_far, &mut record) {
                closest_so_far = record.t;
            }
        }
        if closest_so_far < t_max {
            // println!("hit: {}", closest_so_far);
            // std::process::exit(0);
            Some(record)
        } else {
            None
        }
    }
}

impl Bounded<f32, 3> for Mesh {
    fn aabb(&self) -> Aabb<f32, 3> {
        self.aabb
    }
}

const EPSILON: f32 = 1e-8;

impl BHShape<f32, 3> for Triangle {
    fn set_bh_node_index(&mut self, index: usize) {
        self.node_index = index;
    }

    fn bh_node_index(&self) -> usize {
        self.node_index
    }
}

impl Bounded<f32, 3> for Triangle {
    fn aabb(&self) -> Aabb<f32, 3> {
        let min = glm::min3(&self.v1, &self.v2, &self.v3);
        let max = glm::max3(&self.v1, &self.v2, &self.v3);
        Aabb::with_bounds(min.into(), max.into())
    }
}

impl Cube {
    pub fn new(center: glm::Vec3, size: glm::Vec3, material: &Material) -> Self {
        Self {
            center,
            size,
            material: material.clone(),
        }
    }
}

impl Bounded<f32, 3> for Cube {
    fn aabb(&self) -> Aabb<f32, 3> {
        let min = self.center - self.size / 2.0;
        let max = self.center + self.size / 2.0;
        Aabb::with_bounds(min.into(), max.into())
    }
}

impl Hittable for Cube {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let compute_interval = |dim: usize| {
            let min = self.center[dim] - self.size[dim] / 2.0;
            let max = self.center[dim] + self.size[dim] / 2.0;
            let mut x1 = (min - ray.origin[dim]) / ray.direction[dim];
            let mut x2 = (max - ray.origin[dim]) / ray.direction[dim];
            let mut x1n: glm::Vec3 = glm::zero();
            let mut x2n: glm::Vec3 = glm::zero();
            x1n[dim] = -1.0;
            x2n[dim] = 1.0;
            if x1 > x2 {
                std::mem::swap(&mut x1, &mut x2);
                std::mem::swap(&mut x1n, &mut x2n);
            }
            (x1, x2, x1n, x2n)
        };
        let (x1, x2, x1n, x2n) = compute_interval(0);
        let (y1, y2, y1n, y2n) = compute_interval(1);
        let (z1, z2, z1n, z2n) = compute_interval(2);

        let (start, start_normal) = {
            if x1 > y1 && x1 > z1 {
                (x1, x1n)
            } else if y1 > z1 {
                (y1, y1n)
            } else {
                (z1, z1n)
            }
        };
        let (end, end_normal) = {
            if x2 < y2 && x2 < z2 {
                (x2, x2n)
            } else if y2 < z2 {
                (y2, y2n)
            } else {
                (z2, z2n)
            }
        };

        if start > end || end < t_min || start > t_max {
            return None;
        }
        let (time, normal) = if start < t_min {
            (end, end_normal)
        } else {
            (start, start_normal)
        };
        let mut record = HitRecord::new(&self.material);
        record.t = time;
        record.normal = normal;
        record.material = &self.material;
        Some(record)
    }
}

impl Triangle {
    pub fn from_vertices(v1: glm::Vec3, v2: glm::Vec3, v3: glm::Vec3) -> Self {
        let n = (v2 - v1).cross(&(v3 - v1)).normalize();
        Self {
            v1,
            v2,
            v3,
            n1: n,
            n2: n,
            n3: n,
            node_index: 0,
        }
    }

    pub fn hit(&self, ray: &Ray, t_min: f32, t_max: f32, record: &mut HitRecord) -> bool {
        let d0 = self.v2 - self.v1;
        let d1 = self.v3 - self.v1;
        let plane_normal = d0.cross(&d1).normalize();
        let cosine = plane_normal.dot(&ray.direction);
        if cosine.abs() < EPSILON {
            return false;
        }

        let time = (self.v1 - ray.origin).dot(&plane_normal) / cosine;
        if time < t_min || time >= t_max {
            return false;
        }

        // Okay, so let's compute barycentric coordinates now, fast
        // https://gamedev.stackexchange.com/a/23745
        let d2 = ray.at(time) - self.v1;
        let d00 = d0.dot(&d0);
        let d01 = d0.dot(&d1);
        let d11 = d1.dot(&d1);
        let d20 = d2.dot(&d0);
        let d21 = d2.dot(&d1);
        let denom = d00 * d11 - d01 * d01;
        let v = (d11 * d20 - d01 * d21) / denom;
        let w = (d00 * d21 - d01 * d20) / denom;
        let u = 1.0 - v - w;

        if u >= 0.0 && v >= 0.0 && w >= 0.0 {
            // record.time = time;
            // record.normal = (u * self.n1 + v * self.n2 + w * self.n3).normalize();
            // true
            let front_face = ray.direction.dot(&plane_normal) < 0.0;
            record.t = time;
            record.point = ray.at(time);
            record.normal = plane_normal;
            record.front_face = front_face;
            record.u = u;
            record.v = v;
            true
        } else {
            false
        }
    }
}
