extern crate piston_window;
extern crate ai_behavior;
extern crate sprite;
extern crate find_folder;

use std::rc::Rc;

use piston_window::*;
use sprite::*;
use ai_behavior::{
    Action,
    Sequence,
    Wait,
};

fn make_layer<I>(size_x: usize, size_y: usize,
              scene: &mut sprite::Scene<I>,
              tex: Rc<I>) where I: piston_window::ImageSize {
    let (step_x, step_y) = (33.0, 17.0);
    let (x_start, y_start) = (size_x as f64 * step_x * 2.0, size_y as f64 * step_y);
    let scale = 2.0;
    for x in 0..size_x {
        for y in 0..size_y {
            let mut sprite = Sprite::from_texture(tex.clone());

            sprite.set_position(
                x_start - scale * (step_x * x as f64) + scale * (step_x * y as f64),
                y_start + scale * (step_y * x as f64) + scale * (step_y * y as f64));
            sprite.set_scale(scale, scale);

            &scene.add_child(sprite);
        }
    }
}

fn main() {
    let (width, height) = (800, 600);
    let opengl = OpenGL::V3_2;
    let mut window: PistonWindow =
        WindowSettings::new("piston: sprite", (width, height))
        .exit_on_esc(true)
        .opengl(opengl)
        .build()
        .unwrap();

    let assets = find_folder::Search::ParentsThenKids(3, 3)
        .for_folder("assets").unwrap();
    let mut scene = Scene::new();
    let tex = Rc::new(Texture::from_path(
            &mut window.factory,
            assets.join("tiles/ts_grass0/0.png"),
            Flip::None,
            &TextureSettings::new()
        ).unwrap());

    let tex_miner = Rc::new(Texture::from_path(
            &mut window.factory,
            assets.join("tiles/miner_0.png"),
            Flip::None,
            &TextureSettings::new()
        ).unwrap());

    let field_size = 8;
    make_layer(field_size, field_size, &mut scene, tex);

    let mut sprite = Sprite::from_texture(tex_miner.clone());
    sprite.set_position(width as f64 / 2.0, height as f64 / 2.0);
    sprite.set_scale(2.0, 2.0);
    let id = scene.add_child(sprite);

    // Run a sequence of animations.
    let seq = Sequence(vec![
        Action(MoveBy(1.0, 50.0, 100.0)),
        Wait(1.0),
        Action(MoveBy(1.0, 60.0, 50.0)),
        Wait(1.0),
        Action(MoveBy(1.0, 20.0, 20.0)),
    ]);
    scene.run(id, &seq);

    while let Some(e) = window.next() {
        scene.event(&e);

        window.draw_2d(&e, |c, g| {
            clear([0.2, 0.2, 0.2, 0.2], g);
            scene.draw(c.transform, g);
        });
        if let Some(_) = e.press_args() {
        }
    }
}
