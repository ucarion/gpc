#[macro_use]
extern crate lazy_static;

#[allow(dead_code)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
mod gpc {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

use std::ptr;

#[derive(Copy, Clone)]
pub enum ClipOperation {
    Difference,
    Intersection,
    ExclusiveOr,
    Union,
}

impl ClipOperation {
    fn into_gpc_op(self) -> gpc::gpc_op {
        match self {
            ClipOperation::Difference => gpc::gpc_op::GPC_DIFF,
            ClipOperation::Intersection => gpc::gpc_op::GPC_INT,
            ClipOperation::ExclusiveOr => gpc::gpc_op::GPC_XOR,
            ClipOperation::Union => gpc::gpc_op::GPC_UNION,
        }
    }
}

pub struct Vertex(gpc::gpc_vertex);

impl Vertex {
    pub fn new(x: f64, y: f64) -> Vertex {
        Vertex(gpc::gpc_vertex { x: x, y: y })
    }

    pub fn x(&self) -> f64 {
        self.0.x
    }

    pub fn y(&self) -> f64 {
        self.0.y
    }
}

pub struct Polygon(gpc::gpc_polygon);

impl Polygon {
    pub fn new() -> Polygon {
        Polygon(gpc::gpc_polygon {
            num_contours: 0,
            contour: ptr::null_mut(),
            hole: ptr::null_mut(),
        })
    }

    pub fn add_contour(&mut self, is_hole: bool, vertices: &[Vertex]) {
        // These don't actually need to be `mut`, but the bindgen-generated code expects *mut _, so
        // making these mutable is less ugly than coercing `*const` into `*mut`.
        let mut vertices: Vec<_> = vertices.iter().map(|vertex| vertex.0).collect();
        let mut vertex_list = gpc::gpc_vertex_list {
            num_vertices: vertices.len() as i32,
            vertex: vertices.as_mut_ptr(),
        };

        unsafe {
            gpc::gpc_add_contour(&mut self.0, &mut vertex_list, is_hole as i32);
        }
    }

    pub fn from_clipping(subject: &Polygon, clip: &Polygon, operation: ClipOperation) -> Polygon {
        let s = (&subject.0 as *const _) as *mut _;
        let c = (&clip.0 as *const _) as *mut _;
        let mut result = Polygon::new();

        unsafe {
            gpc::gpc_polygon_clip(operation.into_gpc_op(), s, c, &mut result.0);
        }

        result
    }

    pub fn contours<'a>(&'a self) -> Contours<'a> {
        Contours {
            polygon: &self,
            index: 0,
        }
    }
}

impl Drop for Polygon {
    fn drop(&mut self) {
        unsafe {
            gpc::gpc_free_polygon(&mut self.0);
        }
    }
}

pub struct Contours<'a> {
    polygon: &'a Polygon,
    index: i32,
}

impl<'a> Iterator for Contours<'a> {
    type Item = Contour<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.polygon.0.num_contours {
            return None;
        }

        self.index += 1;
        Some(Contour {
            polygon: self.polygon,
            index: self.index - 1,
        })
    }
}

pub struct Contour<'a> {
    polygon: &'a Polygon,
    index: i32,
}

impl<'a> Contour<'a> {
    pub fn is_hole(&self) -> bool {
        unsafe { *self.polygon.0.hole.offset(self.index as isize) != 0 }
    }

    pub fn vertices(&self) -> ContourVertices<'a> {
        ContourVertices {
            polygon: self.polygon,
            contour_index: self.index,
            vertex_index: 0,
        }
    }
}

pub struct ContourVertices<'a> {
    polygon: &'a Polygon,
    contour_index: i32,
    vertex_index: i32,
}

impl<'a> Iterator for ContourVertices<'a> {
    type Item = Vertex;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let vertex_list = *self.polygon.0.contour.offset(self.contour_index as isize);

            if self.vertex_index == vertex_list.num_vertices {
                return None;
            }

            let vertex = *vertex_list.vertex.offset(self.vertex_index as isize);
            self.vertex_index += 1;
            Some(Vertex(vertex))
        }
    }
}

pub struct Tristrip(gpc::gpc_tristrip);

impl Tristrip {
    pub fn new() -> Tristrip {
        Tristrip(gpc::gpc_tristrip {
            num_strips: 0,
            strip: ptr::null_mut(),
        })
    }

    pub fn from_clipping(subject: &Polygon, clip: &Polygon, operation: ClipOperation) -> Tristrip {
        let s = (&subject.0 as *const _) as *mut _;
        let c = (&clip.0 as *const _) as *mut _;
        let mut result = Tristrip::new();

        unsafe {
            gpc::gpc_tristrip_clip(operation.into_gpc_op(), s, c, &mut result.0);
        }

        result
    }

    pub fn triangle_strips<'a>(&'a self) -> TriangleStrips<'a> {
        TriangleStrips {
            tristrip: &self,
            index: 0,
        }
    }
}

impl Drop for Tristrip {
    fn drop(&mut self) {
        unsafe {
            gpc::gpc_free_tristrip(&mut self.0);
        }
    }
}

pub struct TriangleStrips<'a> {
    tristrip: &'a Tristrip,
    index: i32,
}

impl<'a> Iterator for TriangleStrips<'a> {
    type Item = TriangleStrip<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.tristrip.0.num_strips {
            return None;
        }

        self.index += 1;
        Some(TriangleStrip {
            tristrip: self.tristrip,
            index: self.index - 1,
        })
    }
}

pub struct TriangleStrip<'a> {
    tristrip: &'a Tristrip,
    index: i32,
}

impl<'a> TriangleStrip<'a> {
    pub fn vertices(&'a self) -> TriangleStripVertices<'a> {
        TriangleStripVertices {
            tristrip: self.tristrip,
            strip_index: self.index,
            vertex_index: 0,
        }
    }
}

pub struct TriangleStripVertices<'a> {
    tristrip: &'a Tristrip,
    strip_index: i32,
    vertex_index: i32,
}

impl<'a> Iterator for TriangleStripVertices<'a> {
    type Item = Vertex;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let triangle_strip = *self.tristrip.0.strip.offset(self.strip_index as isize);

            if triangle_strip.num_vertices == self.vertex_index {
                return None;
            }

            let vertex = *triangle_strip.vertex.offset(self.vertex_index as isize);
            self.vertex_index += 1;
            Some(Vertex(vertex))
        }
    }
}

lazy_static! {
    pub static ref GPC_VERSION: &'static str = {
        use std::ffi::CStr;

        CStr::from_bytes_with_nul(gpc::GPC_VERSION)
            .expect("Could not convert GPC_VERSION to CStr")
            .to_str()
            .expect("GPC_VERSION was not valid UTF-8")
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version() {
        assert_eq!("2.32", *GPC_VERSION);
    }

    #[test]
    fn clip_polygon() {
        let mut polygon1 = Polygon::new();
        polygon1.add_contour(
            false,
            &[
                Vertex::new(1.0, 1.0),
                Vertex::new(0.0, 1.0),
                Vertex::new(0.0, 0.0),
                Vertex::new(1.0, 0.0),
            ],
        );

        let mut polygon2 = Polygon::new();
        polygon2.add_contour(
            false,
            &[
                Vertex::new(1.5, 1.5),
                Vertex::new(0.5, 1.5),
                Vertex::new(0.5, 0.5),
                Vertex::new(1.5, 0.5),
            ],
        );

        let polygon = Polygon::from_clipping(&polygon1, &polygon2, ClipOperation::ExclusiveOr);

        let holes: Vec<_> = polygon
            .contours()
            .map(|contour| contour.is_hole())
            .collect();
        assert_eq!(vec![false, false], holes);

        let points: Vec<Vec<_>> = polygon
            .contours()
            .map(|contour| {
                contour
                    .vertices()
                    .map(|vertex| (vertex.x(), vertex.y()))
                    .collect()
            })
            .collect();

        let expected: Vec<Vec<(f64, f64)>> = vec![
            vec![(1.5, 0.5), (1.0, 0.5), (1.0, 1.0), (0.5, 1.0), (0.5, 1.5), (1.5, 1.5)],
            vec![(0.5, 0.5), (1.0, 0.5), (1.0, 0.0), (0.0, 0.0), (0.0, 1.0), (0.5, 1.0)],
        ];
        assert_eq!(expected, points);
    }
}
