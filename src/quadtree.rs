use std::fmt;
use rand;
use rand::Rng;
use cgmath::Vector2;
use std::collections::HashSet;

#[derive(Clone, PartialEq)]
pub struct QuadTree {
    pub branches: Vec<QuadTree>,
    pub tiles: Vec<usize>,
    pub tiles_set: HashSet<usize>,
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
                    tiles_set: HashSet::new(),
                    x: self.x,
                    y: self.y,
                    width: halfwidth,
                    height: halfheight
                },

                QuadTree {
                    min_width: self.min_width,
                    branches: vec![],
                    tiles: vec![],
                    tiles_set: HashSet::new(),
                    x: self.x,
                    y: self.y + halfheight,
                    width: halfwidth,
                    height: halfheight
                },

                QuadTree {
                    min_width: self.min_width,
                    branches: vec![],
                    tiles: vec![],
                    tiles_set: HashSet::new(),
                    x: self.x + halfwidth,
                    y: self.y,
                    width: halfwidth,
                    height: halfheight
                },

                QuadTree {
                    min_width: self.min_width,
                    branches: vec![],
                    tiles: vec![],
                    tiles_set: HashSet::new(),
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
                self.tiles_set.insert(i);
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

    pub fn find_all(&self, pos: &Vector2<f32>) -> Option<&[usize]> {
        if self.contains(pos) {
            if self.branches.len() == 0 && self.tiles.len() > 0 {
                return Some(&self.tiles[..]);
            } else {
                for branch in self.branches.iter() {
                    if branch.contains(pos) {
                        return branch.find_all(&pos);
                    }
                }
            }
        }
        None
    }

    pub fn find_around_in(&self, pos: &Vector2<f32>, subset: &HashSet<usize>) -> Option<usize> {
        let mut rng = rand::thread_rng();
        if self.contains(pos) {
            let mut id: usize = 0;
            let choices: Vec<usize> = {
                self.branches.iter().filter_map(|branch| {
                    if !branch.contains(pos) && branch.tiles_set.len() > 0 && !branch.tiles_set.is_disjoint(subset) {
                        id = **rng.choose(&branch.tiles_set.intersection(subset).collect::<Vec<_>>()).unwrap();
                        return Some(id);
                    }
                    None
                }).collect()
            };
            if choices.len() > 0 {
                let picked_one = rng.choose(&choices);
                if picked_one.is_some() {
                    return Some(*picked_one.unwrap());
                }
            }
            for _branch in self.branches.iter() {
                if _branch.contains(pos) {
                    return _branch.find_around_in(&pos, subset);
                }
            }
        }
        None
    }

    pub fn remove(&mut self, id: usize) {
        if self.branches.len() == 0 && self.tiles.len() > 0 {
            let old_len = self.tiles.len();
            self.tiles.retain(|&x| x != id);
            self.tiles_set.remove(&id);
        } else {
            for branch in self.branches.iter_mut() {
                branch.remove(id);
            }
        }
    }
}
