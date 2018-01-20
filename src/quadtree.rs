use std::fmt;
use cgmath::Vector2;

#[derive(Clone, PartialEq)]
pub struct QuadTree {
    pub branches: Vec<QuadTree>,
    pub tiles: Vec<usize>,
    pub min_width: f32,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32
}

impl fmt::Debug for QuadTree {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "QT {}, {} -> {}, {} ({}/{})",
               self.x, self.y, self.width, self.height, self.branches.len(), self.tiles.len())
    }
}

impl QuadTree {
    fn contains(&self, p: &Vector2<f32>) -> bool {
        self.x <= p.x && self.y <= p.y && (self.x + self.width) >= p.x && (self.y + self.height) >= p.y
    }

    pub fn print(&self, level: usize) {
        println!("{}Tree {:?}", (0..level).map(|_| "\t").collect::<String>(), self);
        for branch in self.branches.iter() {
            branch.print(level + 1);
        }
    }
    pub fn split(&mut self) {
        let halfwidth = self.width / 2.0;
        let halfheight = self.height / 2.0;
        if halfwidth >= self.min_width {
            self.branches = vec![
                QuadTree {
                    min_width: self.min_width,
                    branches: vec![],
                    tiles: vec![],
                    x: self.x,
                    y: self.y,
                    width: halfwidth,
                    height: halfheight
                },

                QuadTree {
                    min_width: self.min_width,
                    branches: vec![],
                    tiles: vec![],
                    x: self.x,
                    y: self.y + halfheight,
                    width: halfwidth,
                    height: halfheight
                },

                QuadTree {
                    min_width: self.min_width,
                    branches: vec![],
                    tiles: vec![],
                    x: self.x + halfwidth,
                    y: self.y,
                    width: halfwidth,
                    height: halfheight
                },

                QuadTree {
                    min_width: self.min_width,
                    branches: vec![],
                    tiles: vec![],
                    x: self.x + halfwidth,
                    y: self.y + halfheight,
                    width: halfwidth,
                    height: halfheight
                }
            ];
        }
        for branch in self.branches.iter_mut() {
            branch.split();
        };
    }

    fn overlaps(&self, other: &QuadTree) -> bool {
        other.contains(&Vector2 { x: self.x, y: self.y })
            || other.contains(&Vector2 { x: self.x + self.width, y: self.y })
            || other.contains(&Vector2 { x: self.x, y: self.y + self.height })
            || other.contains(&Vector2 { x: self.x + self.width, y: self.y + self.height })
    }

    pub fn insert(&mut self, pos: &Vector2<f32>, i: usize) -> bool {
        if self.contains(&pos) {
            if self.branches.len() == 0 {
                self.tiles.push(i);
                return true;
            } else {
                self.branches[0].insert(&pos, i) || self.branches[1].insert(&pos, i)
                    || self.branches[2].insert(&pos, i) || self.branches[3].insert(&pos, i)
            }
        } else {
            false
        }
    }

    pub fn find(&self, pos: &Vector2<f32>) -> Option<usize> {
        if self.contains(pos) {
            if self.branches.len() == 0 && self.tiles.len() > 0 {
                return Some(self.tiles[0]);
            } else {
                for branch in self.branches.iter() {
                    if branch.contains(pos) {
                        return branch.find(&pos);
                    }
                }
            }
        }
        None
    }
}
