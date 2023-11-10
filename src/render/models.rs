use super::mesh::{ClassicVertex, Mesh, Triangle};
use nalgebra as na;

pub fn cube() -> Mesh<ClassicVertex> {
    let vertices = vec![
        // Top
        ClassicVertex::new(na::point![1.0, 1.0, 1.0], na::vector![0.0, 1.0, 0.0]),
        ClassicVertex::new(na::point![-1.0, 1.0, 1.0], na::vector![0.0, 1.0, 0.0]),
        ClassicVertex::new(na::point![-1.0, 1.0, -1.0], na::vector![0.0, 1.0, 0.0]),
        ClassicVertex::new(na::point![1.0, 1.0, -1.0], na::vector![0.0, 1.0, 0.0]),
        // Bottom
        ClassicVertex::new(na::point![1.0, -1.0, 1.0], na::vector![0.0, -1.0, 0.0]),
        ClassicVertex::new(na::point![-1.0, -1.0, 1.0], na::vector![0.0, -1.0, 0.0]),
        ClassicVertex::new(na::point![-1.0, -1.0, -1.0], na::vector![0.0, -1.0, 0.0]),
        ClassicVertex::new(na::point![1.0, -1.0, -1.0], na::vector![0.0, -1.0, 0.0]),
        // Front
        ClassicVertex::new(na::point![1.0, 1.0, 1.0], na::vector![0.0, 0.0, 1.0]),
        ClassicVertex::new(na::point![-1.0, 1.0, 1.0], na::vector![0.0, 0.0, 1.0]),
        ClassicVertex::new(na::point![-1.0, -1.0, 1.0], na::vector![0.0, 0.0, 1.0]),
        ClassicVertex::new(na::point![1.0, -1.0, 1.0], na::vector![0.0, 0.0, 1.0]),
        // Back
        ClassicVertex::new(na::point![1.0, 1.0, -1.0], na::vector![0.0, 0.0, -1.0]),
        ClassicVertex::new(na::point![-1.0, 1.0, -1.0], na::vector![0.0, 0.0, -1.0]),
        ClassicVertex::new(na::point![-1.0, -1.0, -1.0], na::vector![0.0, 0.0, -1.0]),
        ClassicVertex::new(na::point![1.0, -1.0, -1.0], na::vector![0.0, 0.0, -1.0]),
        // Right
        ClassicVertex::new(na::point![1.0, 1.0, 1.0], na::vector![1.0, 0.0, 0.0]),
        ClassicVertex::new(na::point![1.0, -1.0, 1.0], na::vector![1.0, 0.0, 0.0]),
        ClassicVertex::new(na::point![1.0, -1.0, -1.0], na::vector![1.0, 0.0, 0.0]),
        ClassicVertex::new(na::point![1.0, 1.0, -1.0], na::vector![1.0, 0.0, 0.0]),
        // Left
        ClassicVertex::new(na::point![-1.0, 1.0, 1.0], na::vector![-1.0, 0.0, 0.0]),
        ClassicVertex::new(na::point![-1.0, -1.0, 1.0], na::vector![-1.0, 0.0, 0.0]),
        ClassicVertex::new(na::point![-1.0, -1.0, -1.0], na::vector![-1.0, 0.0, 0.0]),
        ClassicVertex::new(na::point![-1.0, 1.0, -1.0], na::vector![-1.0, 0.0, 0.0]),
    ];

    let triangles = vec![
        // Top
        Triangle([0, 1, 2]),
        Triangle([0, 3, 2]),
        // Bottom
        Triangle([4, 5, 6]),
        Triangle([4, 7, 6]),
        // Front
        Triangle([8, 9, 10]),
        Triangle([8, 11, 10]),
        // Back
        Triangle([12, 13, 14]),
        Triangle([12, 15, 14]),
        // Right
        Triangle([16, 17, 18]),
        Triangle([16, 19, 18]),
        // Left
        Triangle([20, 21, 22]),
        Triangle([20, 23, 22]),
    ];

    Mesh {
        vertices,
        triangles,
    }
}

pub fn double_plane() -> Mesh<na::Point3<f32>> {
    let vertices = vec![
        na::point!(-1.0, 0.0, -1.0),
        na::point!(1.0, 0.0, -1.0),
        na::point!(1.0, 0.0, 1.0),
        na::point!(-1.0, 0.0, 1.0),
    ];

    let triangles = vec![
        // Bottom
        Triangle([2, 0, 1]),
        Triangle([3, 0, 2]),
        // Top
        Triangle([0, 2, 1]),
        Triangle([0, 3, 2]),
    ];

    Mesh {
        vertices,
        triangles,
    }
}
