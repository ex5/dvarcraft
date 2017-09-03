extern crate ai_behavior;
extern crate current;
extern crate find_folder;
extern crate gfx_device_gl;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;
extern crate piston_window;
extern crate sprite;
extern crate uuid;
extern crate vecmath;

use current::{ Current, CurrentGuard };
use gfx_device_gl::Resources;
use piston::input::{ Button, PressEvent, ReleaseEvent, MouseCursorEvent };
use piston_window::{ PistonWindow, WindowSettings, Texture, TextureSettings, Flip, polygon, clear };
use sprite::{ Sprite, Scene };
use std::collections::HashMap;
use std::rc::Rc;

mod raw_resources;
mod miners;
mod tiles;

#[derive(Copy, Clone)]
pub enum SelectionState {
    Inactive,
    Selecting,
    Confirmed,
    Cancelled,
}

pub struct Selection {
    pub coords: [[f64; 2]; 4],
    pub pressed: bool,
    pub just_pressed: bool,
    pub state: SelectionState,
    pub released: bool,
}

impl Selection {
    pub fn new() -> Selection {
        Selection {
            state: SelectionState::Inactive,
            coords: [[0.0, 0.0], [0.0, 0.0], [0.0, 0.0], [0.0, 0.0]],
            pressed: false,
            just_pressed: false,
            released: true,
        }
    }

    pub fn is_selected(&self, pos: (f64, f64)) -> bool {
        graphics::math::inside_triangle(
            [self.coords[0], self.coords[1], self.coords[2]],
            [pos.0, pos.1]) ||
        graphics::math::inside_triangle(
            [self.coords[0], self.coords[2], self.coords[3]],
            [pos.0, pos.1])
    }
}
pub unsafe fn current_tiles() -> Current<tiles::Tiles> { Current::new() }
pub unsafe fn current_scene() -> Current<Scene<Texture<Resources>>> { Current::new() }
pub unsafe fn current_miners() -> Current<miners::Miners> { Current::new() }
pub unsafe fn current_selection() -> Current<Selection> { Current::new() }

struct Game<'a>
 {
    textures: HashMap<&'a str, Rc<Texture<Resources>>>,
    window: PistonWindow,
    assets: std::path::PathBuf,
}

pub fn get_selection_top(x0: f64, y0: f64, x1: f64, y1: f64) -> [f64; 2] {
    let (tga, tgb) = (0.52056705, 1.93912501);

    let x = (x1 - y1 * tgb + y0 * tgb + x0 * tga * tgb) / (1.0 + tga * tgb);
    let y = y0 - x * tga + x0 * tga;

    return [x, y];
}

impl <'a> Game<'a>
{
    fn load_texture(&mut self, path: &str) -> Rc<Texture<Resources>> {
        Rc::new(Texture::from_path(
                &mut self.window.factory,
                self.assets.join(path),
                Flip::None,
                &TextureSettings::new()
        ).unwrap())
    }

    pub fn new() -> Game<'a>  {
        let textures = HashMap::new();

        let (width, height) = (800, 600);
        let window: PistonWindow =
            WindowSettings::new("piston: sprite", (width, height))
            .exit_on_esc(true)
            .build()
            .unwrap();

        let assets = find_folder::Search::ParentsThenKids(3, 3)
            .for_folder("assets").unwrap();

        Game {
            assets: assets,
            textures: textures,
            window: window,
        }
    }

    pub fn init(&mut self) {
        let mut path = self.load_texture("tiles/ts_grass0/0.png");
        self.textures.insert("grass", path);
        path = self.load_texture("tiles/miner_0.png");
        self.textures.insert("miner", path);
    }

    pub fn make_layer(&mut self, size_x: usize, size_y: usize) {
        let (step_x, step_y) = (33.0, 17.0);
        //let (x_start, y_start) = (size_x as f64 * step_x, size_y as f64 * step_y);
        let (x_start, y_start) = (0.0, 0.0);
        let scale = 1.0;
        let mut tiles = unsafe { current_tiles() };
        for x in 0..size_x {
            for y in 0..size_y {
                let mut sprite = Sprite::from_texture(self.textures["grass"].clone());

                sprite.set_position(
                    x_start - scale * (step_x * x as f64) + scale * (step_x * y as f64),
                    y_start + scale * (step_y * x as f64) + scale * (step_y * y as f64));
                //sprite.set_scale(scale, scale);

                let id = unsafe{ current_scene() }.add_child(sprite);
                tiles.tiles.push(tiles::Tile::new(id, raw_resources::RawResource::Mud));
            }
        }
    }

    pub fn run(&mut self) {
        let (dx, dy) = (0.0, 0.0);

        let mut cursor = [0.0, 0.0];
        let mut cursor_pressed = [0.0, 0.0];

        let mut scene = unsafe{ current_scene() };
        let mut tiles = unsafe{ current_tiles() };
        let mut miners = unsafe{ current_miners() };
        let mut selection = unsafe{ current_selection() };
        let window = &mut self.window;

        // Add a miner
        let miner = miners::Miner::new(self.textures["miner"].clone());
        miners.miners.push(miner);

        while let Some(e) = window.next() {
            scene.event(&e);

            if let Some(Button::Mouse(button)) = e.press_args() {
                //println!("Pressed mouse button '{:?}'", button);
                selection.just_pressed = true;
                selection.pressed = true;
                selection.released = false;
            }
            if let Some(Button::Mouse(button)) = e.release_args() {
                //println!("Released mouse button '{:?}'", button);
                selection.pressed = false;
                selection.just_pressed = false;
                selection.released = true;
                for tile in tiles.tiles.iter_mut() {
                    if tile.state == ai_behavior::State::new(
                        ai_behavior::Action(tiles::TileState::Selecting)) {
                        tile.set_state(tiles::TileState::Selected);
                    } else {
                        tile.set_state(tiles::TileState::Idle);
                    }
                }
            }
            e.mouse_cursor(|x, y| {
                if selection.just_pressed == true {
                    //println!("Cursor at '{} {} {}'", x, y, pressed);
                    cursor_pressed = [dx + x, dx + y];
                    selection.just_pressed = false;
                }
                if selection.pressed == true {
                    cursor = [dx + x, dy + y];
                }
            });

            if selection.pressed == true && !selection.just_pressed {
                selection.coords = [
                    cursor_pressed,
                    get_selection_top(
                    cursor_pressed[0], cursor_pressed[1], cursor[0], cursor[1]),
                    cursor,
                    get_selection_top(
                    cursor[0], cursor[1], cursor_pressed[0], cursor_pressed[1])
                ];

                for tile in tiles.tiles.iter_mut() {
                    let sprite = scene.child(tile.id).unwrap();
                    let pos = sprite.get_position();
                    if selection.is_selected(pos) {
                        tile.set_state(tiles::TileState::Selecting);
                    } else {
                        tile.set_state(tiles::TileState::Idle);
                    }
                }
            }

            tiles::update_tiles(&e);

            miners::update_miners(&e);

            window.draw_2d(&e, |c, g| {
                clear([0.2, 0.2, 0.2, 0.2], g);
                scene.draw(c.transform, g);
                if selection.pressed == true && !selection.just_pressed {
                    polygon([1.0, 0.0, 0.0, 0.2],
                        &selection.coords, c.transform, g);
                }
            });
        }
    }
}

fn main() {
    let field_size = 100;
    let mut game = Game::new();

    let mut scene: Scene<Texture<Resources>> = Scene::new();
    let mut tiles = tiles::Tiles::new();
    let mut miners = miners::Miners::new();
    let mut selection = Selection::new();

    let scene_guard = CurrentGuard::new(&mut scene);
    let tiles_guard = CurrentGuard::new(&mut tiles);
    let miners_guard = CurrentGuard::new(&mut miners);
    let selection_guard = CurrentGuard::new(&mut selection);

    game.init();
    println!("layer rendering");
    game.make_layer(field_size, field_size);
    println!("layer done");

    game.run();

    drop(miners_guard);
    drop(scene_guard);
    drop(selection_guard);
    drop(tiles_guard);
}
