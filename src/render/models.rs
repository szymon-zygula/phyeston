use super::mesh::{ClassicVertex, Mesh, Triangle};
use nalgebra as na;

pub fn cube() -> Mesh<ClassicVertex> {
    let vertices = vec![
        // Top
        ClassicVertex::new(na::point![1.0, 1.0, 1.0], na::vector![0.0, 0.0, 1.0]),
        ClassicVertex::new(na::point![-1.0, 1.0, 1.0], na::vector![0.0, 0.0, 1.0]),
        ClassicVertex::new(na::point![-1.0, -1.0, 1.0], na::vector![0.0, 0.0, 1.0]),
        ClassicVertex::new(na::point![1.0, -1.0, 1.0], na::vector![0.0, 0.0, 1.0]),
        // Bottom
        ClassicVertex::new(na::point![1.0, 1.0, -1.0], na::vector![0.0, 0.0, -1.0]),
        ClassicVertex::new(na::point![-1.0, 1.0, -1.0], na::vector![0.0, 0.0, -1.0]),
        ClassicVertex::new(na::point![-1.0, -1.0, -1.0], na::vector![0.0, 0.0, -1.0]),
        ClassicVertex::new(na::point![1.0, -1.0, -1.0], na::vector![0.0, 0.0, -1.0]),
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
        Triangle([3, 0, 2]),
        // Bottom
        Triangle([5, 4, 6]),
        Triangle([4, 7, 6]),
        // Front
        Triangle([9, 8, 10]),
        Triangle([8, 11, 10]),
        // Back
        Triangle([12, 13, 14]),
        Triangle([15, 12, 14]),
        // Right
        Triangle([16, 17, 18]),
        Triangle([19, 16, 18]),
        // Left
        Triangle([21, 20, 22]),
        Triangle([20, 23, 22]),
    ];

    Mesh {
        vertices,
        triangles,
    }
}

pub fn double_plane() -> Mesh<ClassicVertex> {
    let up = na::vector![0.0, 1.0, 0.0];
    let down = na::vector![0.0, -1.0, 0.0];
    let vertices = vec![
        // Top
        ClassicVertex::new(na::point![-1.0, 0.0, -1.0], up),
        ClassicVertex::new(na::point![1.0, 0.0, -1.0], up),
        ClassicVertex::new(na::point![1.0, 0.0, 1.0], up),
        ClassicVertex::new(na::point![-1.0, 0.0, 1.0], up),
        // Bottom
        ClassicVertex::new(na::point![-1.0, 0.0, -1.0], down),
        ClassicVertex::new(na::point![1.0, 0.0, -1.0], down),
        ClassicVertex::new(na::point![1.0, 0.0, 1.0], down),
        ClassicVertex::new(na::point![-1.0, 0.0, 1.0], down),
    ];

    let triangles = vec![
        // Top
        Triangle([2, 1, 0]),
        Triangle([3, 2, 0]),
        // Bottom
        Triangle([4, 5, 6]),
        Triangle([4, 6, 7]),
    ];

    Mesh {
        vertices,
        triangles,
    }
}
