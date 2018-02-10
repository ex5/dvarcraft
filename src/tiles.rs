use find_folder;
use image;
use rand;
use rand::Rng;
use image::Pixel;
use cgmath::{ InnerSpace, Vector2 };

use selection;
use std::iter::Iterator;
use quadtree::QuadTree;

#[derive(Debug)]
pub struct Tile {
    pub position: Vector2<f32>,
    pub tex_id: u32,
    pub is_selected: bool,
}

impl Tile {
    pub fn new(
        position: Vector2<f32>,
        tex_id: u32,
    ) -> Tile {
        Tile {
            position: position,
            tex_id: tex_id,
            is_selected: false,
        }
    }
}

pub struct Tiles {
    pub tiles: Vec<Tile>,
    pub walkable: Vec<usize>,
    pub width: usize,
    pub height: usize,
    pub tree: QuadTree,
}

impl Tiles {
    pub fn new_layer_from_heightmap(filename: &str, layer_idx: u8) -> Tiles {
        let assets = find_folder::Search::ParentsThenKids(3, 3)
            .for_folder("assets").unwrap();
        /* Read the height map */
        let max_layers = 5;
        let heightmap = image::open(assets.join(filename)).unwrap().to_rgba();
        let (mut lowest, mut highest) = (255, 0);
        for pixel in heightmap.pixels() {
            // assume it's grayscale and only use one channel
            if pixel[0] < lowest { lowest = pixel[0]; }
            if pixel[0] > highest { highest = pixel[0]; }
        }
        let midpoint = highest / 2;
        let layer_step = (highest - lowest) / max_layers;
        println!("Lowest point: {:?}, mid point: {:?}, highest point: {:?}, layer step: {:?}", lowest, midpoint, highest, layer_step);

        let (size_x, size_y) = heightmap.dimensions();
        println!("Map: {:?}", heightmap.dimensions());

        let mut tiles = Vec::new();
        let mut walkable = Vec::new();

        let sprite_size = 64.0;
        let (step_x, step_y) = (sprite_size / 2.0, 17.0);
        let (x_start, y_start) = (0.0, step_y * size_y as f32);

        // pre-build a quadtree:
        let region_width = sprite_size * size_x as f32;
        let region_height = 2.0 * step_y * size_y as f32;
        let mut tree = QuadTree {
            min_width: sprite_size,
            branches: vec![],
            tiles: vec![],
            x: - region_width / 2.0,
            y: - region_height / 2.0,
            width: region_width,
            height: region_height,
        };
        tree.split();
        for x in 0..size_x {
            for y in 0..size_y {
                let pixel = heightmap.get_pixel(x,  y).to_rgb().data;
                let mut tex_id = rand::thread_rng().gen_range(2, 4); // grass, clay or stone
                if pixel[0] < (layer_idx * layer_step) {
                    tex_id = 1; // water
                }

                tiles.push(
                    Tile::new(Vector2::new(
                        x_start - step_x * x as f32 + step_x * y as f32,
                        y_start - step_y * x as f32 - step_y * y as f32,
                    ), tex_id)
                );

                let last_id = tiles.len() - 1;

                if tex_id != 1 {
                    // store walkable index
                    walkable.push(last_id);
                }
                tree.insert(&tiles[last_id].position, last_id);
            }
        }

        Tiles {
            tiles: tiles,
            walkable: walkable,
            width: size_x as usize,
            height: size_y as usize,
            tree: tree,
        }
    }

    pub fn assign_closest_selected(&mut self, pos: Vector2<f32>) -> Option<usize> {
        let mut min_dist = 999999.0;
        let mut idx: Option<usize> = None;
        for (id, tile) in self.tiles.iter().enumerate() {
            let tile_pos = tile.position;
            let dist = (pos - tile_pos).magnitude();
            if dist < min_dist {
                min_dist = dist;
                idx = Some(id);
            }
        }
        idx
    }

    pub fn get_random_walkable(&self, count: u8) -> Vec<&Tile>{
        let mut rng = rand::thread_rng();
        (0..count).map(
            |_| rng.choose(&self.walkable).unwrap()
        )
        .map(|&i| &self.tiles[i]).collect::<Vec<_>>()
    }

    pub fn get_closest_random(&self, pos: Vector2<f32>) -> Option<&Tile> {
        let w_step = self.width as i32;
        let nearby_idx = [
            -1, -2, 1, 2, - w_step , w_step, - w_step + 1, - w_step - 1,
             w_step + 1,  w_step - 1];
        let mut res: Option<&Tile> = None;
        for (id, tile) in self.tiles.iter().enumerate() {
            let tile_pos = tile.position;
            let dist = (pos - tile_pos).magnitude();
            if dist < 60.0 {
                let choices = nearby_idx.iter().cloned()
                    .filter(|x|
                            (((id as i32) + x) as usize) < self.tiles.len()
                         && (id as i32) + x >= 0)
                    .map(|x| (id as i32 + x) as usize).collect::<Vec<_>>();
                println!("CHOICES: {:?} for tile {:?}", choices, tile);
                res = Some(&self.tiles[*rand::thread_rng().choose(&choices).unwrap()]);
                break;
            }
        }
        res
    }

    pub fn get_tiles(&self) -> Vec<&Tile> {
        self.tiles.iter().map(|ref tile_ref| *tile_ref).collect::<Vec<_>>()
    }

    pub fn update_selected(&mut self, selection: &selection::Selection) {
        // find selected tiles
        for tile in self.tiles.iter_mut() {
            if selection.pressed {
                if selection.is_selected(tile.position) {
                    tile.is_selected = true;
                } else {
                    tile.is_selected = false;
                }
            } else {
                tile.is_selected = false;
            }
        }
    }

    pub fn tile_at(&mut self, position: Vector2<f32>) -> Option<&Tile> {
        let tile_id = self.tree.find(&position);
        if tile_id.is_some() {
            let i = tile_id.unwrap();
            self.tiles[i].is_selected = true;
            Some(&self.tiles[i])
        } else {
            None
        }
    }
}
