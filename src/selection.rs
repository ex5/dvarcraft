use glium;
use glium_sdl2;
use sdl2;
use vecmath;
use glium::index::PrimitiveType;
use std::collections::HashSet;

#[derive(Copy, Clone)]
pub struct SelectionVertex {
    position: [f32; 2],
    color: [f32; 4],
}

implement_vertex!(SelectionVertex, position, color);

/// The type used for 2D vectors.
type Vec2d<f32> = vecmath::Vector2<f32>;

/// Returns a number that tells which side it is relative to a line.
///
/// Computes the cross product of the vector that gives the line
/// with the vector between point and starting point of line.
/// One side of the line has opposite sign of the other.
#[inline(always)]
fn line_side(line: [f32; 4], v: Vec2d<f32>) -> f32
{
    let (ax, ay) = (line[0], line[1]);
    let (bx, by) = (line[2], line[3]);
    (bx - ax) * (v[1] - ay) - (by - ay) * (v[0] - ax)
}

/// Returns true if point is inside triangle.
///
/// This is done by computing a `side` number for each edge.
/// If the number is inside if it is on the same side for all edges.
/// Might break for very small triangles.
fn inside_triangle(triangle: [[f32; 2]; 3], v: Vec2d<f32>) -> bool {
    let _0 = 0.0;

    let ax = triangle[0][0];
    let ay = triangle[0][1];
    let bx = triangle[1][0];
    let by = triangle[1][1];
    let cx = triangle[2][0];
    let cy = triangle[2][1];

    let min_px = 5.0;
    if (ax - bx).abs() < min_px || (ay - by).abs() < min_px || (bx - cx).abs() < min_px || (by - cy).abs() < min_px || (ax - cx).abs() < min_px || (ay - cy).abs() < min_px {
        return false;
    }

    let ab_side = line_side([ax, ay, bx, by], v);
    let bc_side = line_side([bx, by, cx, cy], v);
    let ca_side = line_side([cx, cy, ax, ay], v);

    let ab_positive = ab_side >= _0;
    let bc_positive = bc_side >= _0;
    let ca_positive = ca_side >= _0;

    ab_positive == bc_positive && bc_positive == ca_positive
}

fn get_selection_top(xy0: [f32; 2], xy1: [f32; 2]) -> [f32; 2] {
    let (tga, tgb) = (0.52056705, 1.93912501);

    let x0 = xy0[0];
    let y0 = xy0[1];
    let x1 = xy1[0];
    let y1 = xy1[1];

    let x = (x1 - y1 * tgb + y0 * tgb + x0 * tga * tgb) / (1.0 + tga * tgb);
    let y = y0 - x * tga + x0 * tga;

    return [x, y];
}

#[derive(Copy, Clone)]
pub enum SelectionState {
    Inactive,
    Selecting,
    Confirmed,
    Cancelled,
}

pub struct Selection {
    pub coords: [[f32; 2]; 4],
    pub pressed: bool,
    pub state: SelectionState,
    pub released: bool,
}

impl Selection {
    pub fn new() -> Selection {
        Selection {
            state: SelectionState::Inactive,
            coords: [[0.0, 0.0], [0.0, 0.0], [0.0, 0.0], [0.0, 0.0]],
            pressed: false,
            released: true,
        }
    }

    pub fn is_selected(&self, pos: (f32, f32)) -> bool {
        inside_triangle(
            [self.coords[0], self.coords[1], self.coords[2]],
            [pos.0, pos.1]) ||
        inside_triangle(
            [self.coords[0], self.coords[2], self.coords[3]],
            [pos.0, pos.1])
    }

    pub fn update(&mut self, state: &sdl2::mouse::MouseState,
                  new_buttons: &HashSet<sdl2::mouse::MouseButton>,
                  old_buttons: &HashSet<sdl2::mouse::MouseButton>,
                  buttons: &HashSet<sdl2::mouse::MouseButton>) {
        let left = &sdl2::mouse::MouseButton::Left; 
        if buttons.contains(left) {
            self.coords[2] = [state.x() as f32, state.y() as f32];
        }

        if new_buttons.contains(left) {
            self.pressed = true;
            self.released = false;
            if !old_buttons.contains(left) { // just pressed
                self.coords[0] = [state.x() as f32, state.y() as f32];
                println!("Just pressed LMB at {:?}", self.coords[0]);
            }
        } else if old_buttons.contains(left) {
            println!("Just released LMB at {:?}", self.coords[2]);
            self.state = SelectionState::Confirmed;
            self.pressed = false;
            self.released = true;
        }
        self.coords = [
            self.coords[0],
            get_selection_top(self.coords[0], self.coords[2]),
            self.coords[2],
            get_selection_top(self.coords[2], self.coords[0])
        ];
    }

    pub fn generate_vertices(&self, display: &glium_sdl2::SDL2Facade) -> (glium::VertexBuffer<SelectionVertex>, glium::index::IndexBuffer<u16>) {
        (glium::VertexBuffer::new(display, 
             &[
             SelectionVertex { position: self.coords[0], color: [1.0, 1.0, 0.0, 0.7] },
             SelectionVertex { position: self.coords[1], color: [1.0, 1.0, 0.0, 0.7] },
             SelectionVertex { position: self.coords[2], color: [1.0, 1.0, 0.0, 0.7] },
             SelectionVertex { position: self.coords[3], color: [1.0, 1.0, 0.0, 0.7] },
             ]
        ).unwrap(),
        glium::IndexBuffer::new(display, PrimitiveType::TrianglesList,
                                &[0u16, 1, 2, 0u16, 2, 3]).unwrap())
    }
}
