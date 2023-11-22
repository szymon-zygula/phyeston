use glow::HasContext;
use nalgebra as na;

const POINT_SIZE: i32 = std::mem::size_of::<na::Point3<f32>>() as i32;

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Triangle(pub [u32; 3]);

pub trait Vertex {
    fn set_vertex_attrib_pointers(gl: &glow::Context);
}

impl Vertex for na::Point3<f32> {
    fn set_vertex_attrib_pointers(gl: &glow::Context) {
        unsafe {
            gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, POINT_SIZE, 0);
            gl.enable_vertex_attrib_array(0);
        }
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ClassicVertex {
    pub position: na::Point3<f32>,
    pub normal: na::Vector3<f32>,
}

impl ClassicVertex {
    pub fn new(position: na::Point3<f32>, normal: na::Vector3<f32>) -> Self {
        Self { position, normal }
    }
}

impl Vertex for ClassicVertex {
    fn set_vertex_attrib_pointers(gl: &glow::Context) {
        unsafe {
            gl.vertex_attrib_pointer_f32(
                0,
                3,
                glow::FLOAT,
                false,
                std::mem::size_of::<ClassicVertex>() as i32,
                0,
            );
            gl.enable_vertex_attrib_array(0);

            gl.vertex_attrib_pointer_f32(
                1,
                3,
                glow::FLOAT,
                false,
                std::mem::size_of::<ClassicVertex>() as i32,
                POINT_SIZE,
            );
            gl.enable_vertex_attrib_array(1);
        }
    }
}

pub struct Mesh<V: Vertex> {
    pub vertices: Vec<V>,
    pub triangles: Vec<Triangle>,
}

impl<V: Vertex> Mesh<V> {
    pub fn new(vertices: Vec<V>, triangles: Vec<Triangle>) -> Self {
        Self {
            vertices,
            triangles,
        }
    }

    pub fn empty() -> Self {
        Self {
            vertices: Vec::new(),
            triangles: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct DuckVertex {
    pub position: na::Point3<f32>,
    pub normal: na::Vector3<f32>,
    pub tex: na::Vector2<f32>,
}

impl DuckVertex {
    pub fn new(position: na::Point3<f32>, normal: na::Vector3<f32>, tex: na::Vector2<f32>) -> Self {
        Self {
            position,
            normal,
            tex,
        }
    }
}

impl Vertex for DuckVertex {
    fn set_vertex_attrib_pointers(gl: &glow::Context) {
        unsafe {
            // Positions
            gl.vertex_attrib_pointer_f32(
                0,
                3,
                glow::FLOAT,
                false,
                std::mem::size_of::<DuckVertex>() as i32,
                0,
            );
            gl.enable_vertex_attrib_array(0);

            // Normals
            gl.vertex_attrib_pointer_f32(
                1,
                3,
                glow::FLOAT,
                false,
                std::mem::size_of::<DuckVertex>() as i32,
                std::mem::size_of::<na::Point3<f32>>() as i32,
            );
            gl.enable_vertex_attrib_array(1);

            // Texture coords
            gl.vertex_attrib_pointer_f32(
                2,
                2,
                glow::FLOAT,
                false,
                std::mem::size_of::<DuckVertex>() as i32,
                std::mem::size_of::<na::Point3<f32>>() as i32
                    + std::mem::size_of::<na::Vector3<f32>>() as i32,
            );
            gl.enable_vertex_attrib_array(2);
        }
    }
}

#[derive(Debug)]
pub struct ParseError;

impl Mesh<DuckVertex> {
    pub fn from_file(path: &std::path::Path) -> Self {
        let path_string = path.to_str().expect("Cannot convert path to string");
        let error_msg = format!("Could not load mesh at {}", path_string);
        let string = std::fs::read_to_string(path).expect(&error_msg);
        Self::parse_model(&string).expect("Error parsing model file")
    }

    pub fn parse_model(string: &str) -> Result<Self, ParseError> {
        let mut lines = string.lines();

        let vertex_count = Self::parse_u32(lines.next().ok_or(ParseError {})?)?;
        let mut vertices = Vec::new();
        for _ in 0..vertex_count {
            vertices.push(Self::parse_vertex(lines.next().ok_or(ParseError {})?)?);
        }

        let triangle_count = Self::parse_u32(lines.next().ok_or(ParseError {})?)?;
        let mut triangles = Vec::new();
        for _ in 0..triangle_count {
            triangles.push(Self::parse_triangle(lines.next().ok_or(ParseError {})?)?);
        }

        Ok(Self {
            triangles,
            vertices,
        })
    }

    pub fn parse_u32(string: &str) -> Result<u32, ParseError> {
        string.parse().map_err(|_| ParseError {})
    }

    pub fn parse_vertex(string: &str) -> Result<DuckVertex, ParseError> {
        let nums: Vec<_> = string.split(' ').flat_map(|s| s.parse()).collect();
        if nums.len() != 8 {
            return Err(ParseError {});
        }

        Ok(DuckVertex {
            position: na::Point3::new(nums[0], nums[1], nums[2]),
            normal: na::Vector3::new(nums[3], nums[4], nums[5]),
            tex: na::Vector2::new(nums[6], nums[7]),
        })
    }

    pub fn parse_triangle(string: &str) -> Result<Triangle, ParseError> {
        let nums: Vec<_> = string.split(' ').flat_map(Self::parse_u32).collect();
        if nums.len() != 3 {
            return Err(ParseError {});
        }

        Ok(Triangle([nums[0], nums[1], nums[2]]))
    }
}
