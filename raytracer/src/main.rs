use std::env;
use std::fs;

extern crate nalgebra_glm as glm;

use nalgebra::Vector3;
use raytracer::camera::Camera;
use raytracer::camera::CameraParams;
use raytracer::config::Config;
use raytracer::config::Sky;
use raytracer::materials::Lambertian;
use raytracer::materials::Light;
use raytracer::materials::Material;
use raytracer::materials::Metal;
use raytracer::mesh::Cube;
use raytracer::point3d::Point3D;
use raytracer::polygon;
use raytracer::raytracer::render;
use raytracer::sphere::RayObject;
use raytracer::sphere::Sphere;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        println!("Usage: {} <config_file> <output_file>", args[0]);
        return;
    }

    let camera = Camera::from(CameraParams {
        look_from: Point3D::new(278.0, 273.0, -800.0),
        look_at: Point3D::new(278.0, 273.0 - 10.0, 0.0),
        vup: Point3D::new(0.0, 1.0, 0.0),
        vfov: 40.0,
        aspect: 1.0,
    });

    let white = Material::Lambertian(Lambertian::new(Vector3::new(0.73, 0.73, 0.73)));
    //let white = Material::Metal(Metal::new(Vector3::new(0.73, 0.73, 0.73), 0.8));
    let red = Material::Lambertian(Lambertian::new(Vector3::new(0.65, 0.05, 0.05)));
    let green = Material::Lambertian(Lambertian::new(Vector3::new(0.12, 0.45, 0.15)));
    let light_mtl = Material::Light(Light::new());

    let floor = polygon(
        &[
            glm::vec3(0.0, -0.01, 0.0),
            glm::vec3(0.0, 0.0, 559.2),
            glm::vec3(556.0, 0.0, 559.2),
            glm::vec3(556.0, 0.0, 0.0),
        ],
        &white,
    );
    let ceiling = polygon(
        &[
            glm::vec3(0.0, 548.8, 0.0),
            glm::vec3(556.0, 548.9, 0.0),
            glm::vec3(556.0, 548.9, 559.2),
            glm::vec3(0.0, 548.9, 559.2),
        ],
        &white,
    );
    // let light_rect = polygon(
    //     &[
    //         glm::vec3(343.0, 548.8, 227.0),
    //         glm::vec3(343.0, 548.8, 332.0),
    //         glm::vec3(213.0, 548.8, 332.0),
    //         glm::vec3(213.0, 548.8, 227.0),
    //     ],
    //     &light_mtl,
    // );
    let light = Sphere::new(Point3D::new(278.0, 529.9, 270.0), 25.0, light_mtl);
    let back_wall = polygon(
        &[
            glm::vec3(0.0, 0.0, 559.2 - 0.001),
            glm::vec3(0.0, 548.9, 559.2),
            glm::vec3(556.0, 548.9, 559.2),
            glm::vec3(556.0, 0.0, 559.2),
        ],
        &white,
    );
    let right_wall = polygon(
        &[
            glm::vec3(0.0 - 0.01, 0.0, 0.0),
            glm::vec3(0.0, 548.9, 0.0),
            glm::vec3(0.0, 548.9, 559.2),
            glm::vec3(0.0, 0.0, 559.2),
        ],
        &red,
    );
    let left_wall = polygon(
        &[
            glm::vec3(556.0 - 0.01, 0.0, 0.0),
            glm::vec3(556.0, 0.0, 559.2),
            glm::vec3(556.0, 548.9, 559.2),
            glm::vec3(556.0, 548.9, 0.0),
        ],
        &green,
    );

    // let large_box = cube()
    //     .scale(&glm::vec3(165.0, 330.0, 165.0))
    //     .rotate_y(glm::two_pi::<f64>() * (-253.0 / 360.0))
    //     .translate(&glm::vec3(368.0, 165.0, 351.0));
    let large_box = Cube::new(
        glm::vec3(368.0, 165.0, 351.0),
        glm::vec3(165.0, 330.0, 165.0),
        &white,
    );
    // let small_box = cube()
    //     .scale(&glm::vec3(165.0, 165.0, 165.0))
    //     .rotate_y(glm::two_pi::<f64>() * (-197.0 / 360.0))
    //     .translate(&glm::vec3(185.0, 82.5, 169.0));
    let small_box = Cube::new(
        glm::vec3(185.0, 82.5, 169.0),
        glm::vec3(165.0, 165.0, 165.0),
        &white,
    );

    let objs = vec![
        RayObject::mesh(floor),
        RayObject::mesh(ceiling),
        RayObject::mesh(back_wall),
        RayObject::mesh(right_wall),
        RayObject::mesh(left_wall),
        RayObject::sphere(light),
        RayObject::cube(large_box),
        RayObject::cube(small_box),
        //RayObject::sphere(Sphere::new(Point3D::new(278.0, 273.0, 140.0), 100.0, white)),
    ];
    let scene = Config {
        width: 500,
        height: 500,
        samples_per_pixel: 100,
        max_depth: 4,
        sky: Some(Sky::new_default_sky()),
        camera,
        objects: objs,
        bvh: raytracer::config::empty_bvh(),
    };

    //let json = fs::read(&args[1]).expect("Unable to read config file.");
    //let scene = serde_json::from_slice::<Config>(&json).expect("Unable to parse config json");

    let filename = &args[2]; //format!("{}_{:0>3}.png", args[2], i);
    println!("\nRendering {}", filename);
    render(&filename, scene);
}
