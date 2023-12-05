use super::mesh::{ClassicVertex, Mesh, Triangle};
use itertools::Itertools;
use nalgebra as na;

pub fn cube() -> Mesh<ClassicVertex> {
    let vertices = vec![
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
        // Front
        Triangle([0, 1, 2]),
        Triangle([3, 0, 2]),
        // Back
        Triangle([5, 4, 6]),
        Triangle([4, 7, 6]),
        // Top
        Triangle([9, 8, 10]),
        Triangle([8, 11, 10]),
        // Bottom
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

pub fn inverse_cube() -> Mesh<ClassicVertex> {
    let mut cube = cube();

    for vertex in &mut cube.vertices {
        vertex.normal = -vertex.normal;
    }

    for Triangle([a, b, _]) in &mut cube.triangles {
        std::mem::swap(a, b);
    }

    cube
}

/// For GlLineStrip
pub fn wire_cube() -> Vec<na::Point3<f32>> {
    vec![
        // Front
        na::point![1.0, 1.0, 1.0,],
        na::point![-1.0, 1.0, 1.0,],
        na::point![-1.0, -1.0, 1.0,],
        na::point![1.0, -1.0, 1.0,],
        na::point![1.0, 1.0, 1.0,],
        // Back
        na::point![1.0, 1.0, -1.0,],
        na::point![-1.0, 1.0, -1.0,],
        na::point![-1.0, -1.0, -1.0,],
        na::point![1.0, -1.0, -1.0,],
        na::point![1.0, 1.0, -1.0,],
        // Sides
        na::point![-1.0, 1.0, -1.0,],
        na::point![-1.0, 1.0, 1.0,],
        na::point![-1.0, -1.0, 1.0,],
        na::point![-1.0, -1.0, -1.0],
        na::point![1.0, -1.0, -1.0,],
        na::point![1.0, -1.0, 1.0,],
    ]
}

/// For GlLines
pub fn wire_grid() -> Vec<na::Point3<f32>> {
    wire_grid_from_fn(|u, v, w| na::point![u as f32 / 3.0, v as f32 / 3.0, w as f32 / 3.0])
}

pub fn wire_grid_from_fn<F: Fn(usize, usize, usize) -> na::Point3<f32>>(
    f: F,
) -> Vec<na::Point3<f32>> {
    (0..3)
        .cartesian_product(0..4)
        .cartesian_product(0..4)
        .flat_map(|((u, v), w)| {
            let un = u + 1;
            [f(u, v, w), f(un, v, w)]
        })
        .chain(
            (0..4)
                .cartesian_product(0..3)
                .cartesian_product(0..4)
                .flat_map(|((u, v), w)| {
                    let vn = v + 1;
                    [f(u, v, w), f(u, vn, w)]
                }),
        )
        .chain(
            (0..4)
                .cartesian_product(0..4)
                .cartesian_product(0..3)
                .flat_map(|((u, v), w)| {
                    let wn = w + 1;
                    [f(u, v, w), f(u, v, wn)]
                }),
        )
        .map(|p| 2.0 * p - na::vector![1.0, 1.0, 1.0])
        .collect()
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

pub fn rect() -> Mesh<na::Point3<f32>> {
    // 0 1
    // 3 2
    Mesh::new(
        vec![
            na::point!(-0.25, 0.25, 0.0),
            na::point!(0.25, 0.25, 0.0),
            na::point!(0.25, -0.25, 0.0),
            na::point!(-0.25, -0.25, 0.0),
        ],
        vec![Triangle([2, 1, 0]), Triangle([3, 2, 0])],
    )
}
