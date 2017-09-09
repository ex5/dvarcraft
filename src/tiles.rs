use vecmath;
use find_folder;
use image;
use image::Pixel;

use selection;

pub struct Tile {
    pub position: (f32, f32),
    pub tex_id: u32,
    pub is_selected: bool,
}

impl Tile {
    pub fn new(
        position: (f32, f32),
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
        let (step_x, step_y) = (33.0, 17.0);
        let (x_start, y_start) = (0.0, 0.0);
        let scale = 1.0;
        for x in 0..size_x {
            for y in 0..size_y {
                let pixel = heightmap.get_pixel(x,  y).to_rgb().data;
                let mut tex_id = 0;
                if pixel[0] < (layer_idx * layer_step) {
                    tex_id = 1;
                }

                tiles.push(
                    Tile::new((
                        x_start - scale * (step_x * x as f32) + scale * (step_x * y as f32),
                        y_start + scale * (step_y * x as f32) + scale * (step_y * y as f32),
                    ), tex_id)
                );
            }
        }
        Tiles {
            tiles: tiles,
        }
    }

    pub fn assign_closest_selected(&mut self, pos: (f32, f32)) -> Option<usize> {
        let mut min_dist = 999999.0;
        let mut idx: Option<usize> = None;
        for (id, tile) in self.tiles.iter().enumerate() {
            let tile_pos = tile.position;
            let a: vecmath::Vector2<f64> = [
                (pos.0 - tile_pos.0) as f64, (pos.1 - tile_pos.1) as f64];
            let dist = vecmath::vec2_len(a);
            if dist < min_dist {
                min_dist = dist;
                idx = Some(id);
            }
        }
        idx
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
}
