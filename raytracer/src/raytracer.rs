use bvh::bounding_hierarchy::BoundingHierarchy;
use bvh::bvh::Bvh;
use image::png::PNGEncoder;
use image::ColorType;
use nalgebra::Vector3;
use palette::float::Float;
use palette::Pixel;
use palette::Srgb;
use rand::seq::IteratorRandom;
use rand::Rng;
use rayon::prelude::*;
use std::fs::File;
use std::ops::Mul;
use std::time::Instant;

use crate::config::Config;
use crate::materials::Material;
use crate::materials::Scatterable;
use crate::ray::HitRecord;
use crate::ray::Hittable;
use crate::ray::Ray;
use crate::sphere::RayObject;
use crate::sphere::RayObjectKind;
use crate::sphere::Sphere;
#[cfg(test)]
use std::fs;

#[cfg(test)]
use crate::point3d::Point3D;

#[cfg(test)]
use crate::camera::Camera;
#[cfg(test)]
use crate::config::Sky;
#[cfg(test)]
use crate::materials::Lambertian;
#[cfg(test)]
use crate::materials::Light;

fn write_image(
    filename: &str,
    pixels: &[u8],
    bounds: (usize, usize),
) -> Result<(), std::io::Error> {
    let output = File::create(filename)?;
    let encoder = PNGEncoder::new(output);
    encoder.encode(pixels, bounds.0 as u32, bounds.1 as u32, ColorType::RGB(8))?;
    Ok(())
}

fn hit_world<'material>(
    world: &'material Config,
    r: &Ray,
    t_min: f32,
    t_max: f32,
) -> Option<HitRecord<'material>> {
    let mut closest_so_far = t_max;
    let mut hit_record = None;

    let ray: bvh::ray::Ray<f32, 3> = bvh::ray::Ray::new(r.origin.into(), r.direction);
    for ray_object in world.bvh.traverse_iterator(&ray, &world.objects) {
        if let Some(hit) = ray_object.hit(r, t_min, closest_so_far) {
            closest_so_far = hit.t;
            hit_record = Some(hit);
        }
    }
    hit_record
}

fn ray_color(
    ray: &Ray,
    scene: &Config,
    lights: &Vec<Sphere>,
    max_depth: usize,
    depth: usize,
    rng: &mut impl Rng,
) -> Vector3<f32> {
    if depth == 0 {
        return Vector3::zeros();
    }

    if let Some(hit_record) = hit_world(scene, ray, 0.001, std::f32::MAX) {
        if let Some((scattered_ray, albedo)) = hit_record.material.scatter(ray, &hit_record) {
            let mut light_color = Vector3::zeros();

            let prob = if let Material::Glass(_) = hit_record.material {
                0.05
            } else {
                0.1
            };

            if !lights.is_empty()
                && rng.gen::<f32>() > (1.0 - lights.len() as f32 * prob)
                && depth > (max_depth - 2)
            {
                for light in lights {
                    let r = ((rng.gen::<f32>() - 0.5).powf(2.0) * 4.0 - 1.0) * light.radius;
                    let x = rng.gen::<f32>() * r;
                    let y = rng.gen::<f32>() * r;
                    let z = rng.gen::<f32>() * r;
                    let light_pos = light.center + Vector3::new(x, y, z);
                    let light_ray = Ray::new(hit_record.point, light_pos - hit_record.point);
                    let target_color = ray_color(&light_ray, scene, lights, 2, 1, rng);
                    light_color += albedo.component_mul(&target_color);
                }
                light_color /= lights.len() as f32;
            }

            if let Some(sr) = scattered_ray {
                let target_color = ray_color(&sr, scene, lights, max_depth, depth - 1, rng);
                let t = light_color + albedo.component_mul(&target_color);
                return Vector3::new(
                    t.x.clamp(0.0, 1.0),
                    t.y.clamp(0.0, 1.0),
                    t.z.clamp(0.0, 1.0),
                );
            }
            return albedo;
        }
        return Vector3::zeros();
    }

    // Sky color calculation
    let norm = ray.direction.normalize();
    let t = 0.5 * (norm.y + 1.0);
    let u = 0.5 * (norm.x + 1.0);

    match &scene.sky {
        None => Vector3::zeros(),
        Some(sky) => match &sky.texture {
            None => Vector3::new((1.0 - t) + t * 0.5, (1.0 - t) + t * 0.7, 1.0),
            Some((pixels, width, height, _)) => {
                let x = (u * (*width - 1) as f32) as usize;
                let y = ((1.0 - t) * (*height - 1) as f32) as usize;
                let idx = (y * *width + x) * 3;
                Vector3::new(
                    0.7 * pixels[idx] as f32 / 255.0,
                    0.7 * pixels[idx + 1] as f32 / 255.0,
                    0.7 * pixels[idx + 2] as f32 / 255.0,
                )
            }
        },
    }
}

fn render_line(pixels: &mut [u8], scene: &Config, lights: &Vec<Sphere>, y: usize) {
    let mut rng = rand::thread_rng();

    let bounds = (scene.width, scene.height);

    //let mut pixel_colors: Vector3<f32> = Vector3::zeros();

    let fu = 1.0 / (bounds.0 as f64 - 1.0);
    let fv = 1.0 / (bounds.1 as f64 - 1.0);

    // let mut rand_x = [0.0f32; 2048];
    // let mut rand_y = [0.0f32; 2048];
    // for i in 0..2048.min(scene.samples_per_pixel as usize) {
    //     rand_x[i] = rng.gen::<f32>();
    //     rand_y[i] = rng.gen::<f32>();
    // }

    for x in 0..bounds.0 {
        let mut pixel_colors = Vector3::zeros();
        for _s in 0..scene.samples_per_pixel {
            //let ru = rand_x[(_s as usize) & 2047];
            //let rv = rand_y[(_s as usize) & 2047];
            let ru = rng.gen::<f64>();
            let rv = rng.gen::<f64>();
            let u = (x as f64 + ru) * fu; // / (bounds.0 as f32 - 1.0);
            let v = (bounds.1 as f64 - (y as f64 + rv)) * fv;
            let r = scene.camera.get_ray(u as f32, v as f32);
            let c = ray_color(
                &r,
                scene,
                lights,
                scene.max_depth,
                scene.max_depth,
                &mut rng,
            );
            pixel_colors += c;
        }
        let scale = 1.0 / scene.samples_per_pixel as f32;
        // let color = Srgb::new(
        //     (scale * pixel_colors[0]).sqrt(),
        //     (scale * pixel_colors[1]).sqrt(),
        //     (scale * pixel_colors[2]).sqrt(),
        // );
        //let pixel: [u8; 3] = color.into_format().into_raw();
        pixels[x * 3] = ((pixel_colors.x * scale).sqrt() * 255.0) as u8;
        pixels[x * 3 + 1] = ((pixel_colors.y * scale).sqrt() * 255.0) as u8;
        pixels[x * 3 + 2] = ((pixel_colors.z * scale).sqrt() * 255.0) as u8;
    }
}

fn find_lights(world: &Vec<RayObject>) -> Vec<Sphere> {
    world
        .iter()
        .filter_map(|x| match &x.kind {
            RayObjectKind::Sphere(s) => Some(s),
            _ => None,
        })
        .filter(|s| match s.material {
            Material::Light(_) => true,
            _ => false,
        })
        .cloned()
        .collect::<Vec<Sphere>>()
}

pub fn render(filename: &str, mut scene: Config) {
    let image_width = scene.width;
    let image_height = scene.height;

    let bvh = Bvh::build(&mut scene.objects);
    scene.bvh = bvh;

    let mut pixels = vec![0; image_width * image_height * 3];
    let bands: Vec<(usize, &mut [u8])> = pixels.chunks_mut(image_width * 3).enumerate().collect();

    let lights = find_lights(&scene.objects);
    println!("Lights: {:?}", lights.len());

    let start = Instant::now();
    // bands.into_par_iter().for_each(|(i, band)| {
    //     render_line(band, &scene, &lights, i);
    // });
    bands.into_iter().for_each(|(i, band)| {
        render_line(band, &scene, &lights, i);
    });
    println!("Frame time: {}ms", start.elapsed().as_millis());

    write_image(filename, &pixels, (image_width, image_height)).expect("error writing image");
}

#[test]
fn test_render_full_test_scene() {
    let json = fs::read("data/test_scene.json").expect("Unable to read file");
    let mut scene = serde_json::from_slice::<Config>(&json).expect("Unable to parse json");
    scene.width = 80;
    scene.height = 60;
    render("/tmp/test_scene.png", scene);
}

#[test]
fn test_render_full_cover_scene() {
    let json = fs::read("data/cover_scene.json").expect("Unable to read file");
    let mut scene = serde_json::from_slice::<Config>(&json).expect("Unable to parse json");
    scene.width = 40;
    scene.height = 30;
    render("/tmp/cover_scene.png", scene);
}
