#[macro_use]
extern crate glium;
extern crate rand;
extern crate clock_ticks;
extern crate cgmath;
extern crate image;
extern crate find_folder;

extern crate glium_sdl2;
extern crate sdl2;

use glium::{ Surface };
use glium::index::PrimitiveType;
use glium_sdl2::{ DisplayBuild };
use sdl2::keyboard::Keycode;
use sdl2::video::GLProfile;
use sdl2::event::Event;
use std::collections::HashSet;

mod textures;
mod shaders;
mod tiles;
mod miners;
mod selection;

#[derive(Copy, Clone)]
struct SpriteVertex {
    i_position: [f32; 2],
    i_tex_id: u32,
    is_selected: f32,
}
implement_vertex!(SpriteVertex, i_position, i_tex_id, is_selected);

fn generate_vertices(display: &glium_sdl2::SDL2Facade, tiles: &Vec<&tiles::Tile>) -> (glium::VertexBuffer<SpriteVertex>, glium::index::IndexBuffer<u16>) {
    let sprites_count = tiles.len();
    let mut vb: glium::VertexBuffer<SpriteVertex> = glium::VertexBuffer::empty_dynamic(
        display, sprites_count * 4).unwrap();
    let mut ib_data = Vec::with_capacity(sprites_count * 6);

    // initializing with positions and texture IDs
    for (num, sprite) in vb.map().chunks_mut(4).enumerate() {
        let tile = &tiles[num];
        let position = tile.position;
        let tex_id = tile.tex_id;
        let is_selected = match tile.is_selected {
            true => 0.5, false => 0.0
        };

        sprite[0].i_position[0] = position.x - 32.0;
        sprite[0].i_position[1] = position.y + 32.0;
        sprite[0].i_tex_id = tex_id;
        sprite[0].is_selected = is_selected;

        sprite[1].i_position[0] = position.x + 32.0;
        sprite[1].i_position[1] = position.y + 32.0;
        sprite[1].i_tex_id = tex_id;
        sprite[1].is_selected = is_selected;

        sprite[2].i_position[0] = position.x - 32.0;
        sprite[2].i_position[1] = position.y - 32.0;
        sprite[2].i_tex_id = tex_id;
        sprite[2].is_selected = is_selected;

        sprite[3].i_position[0] = position.x + 32.0;
        sprite[3].i_position[1] = position.y - 32.0;
        sprite[3].i_tex_id = tex_id;
        sprite[3].is_selected = is_selected;

        let num = num as u16;
        ib_data.push(num * 4);
        ib_data.push(num * 4 + 1);
        ib_data.push(num * 4 + 2);
        ib_data.push(num * 4 + 1);
        ib_data.push(num * 4 + 3);
        ib_data.push(num * 4 + 2);
    }

    (vb, glium::index::IndexBuffer::new(display, PrimitiveType::TrianglesList, &ib_data).unwrap())
}

fn main() {
    let (w, h) = (800, 600);
    let mut tiles = tiles::Tiles::new_layer_from_heightmap("heightmap_64.png", 1);
    let mut miners = miners::Miners::new(10, &tiles);
    let miners_count: usize = miners.miners.len();
    let sprites_count: usize = tiles.tiles.len();
    println!("Number of sprites: {}", sprites_count);

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(GLProfile::Core);
    gl_attr.set_context_version(3, 3);

    let display = video_subsystem.window("Dvarcraft", w, h)
        .build_glium()
        .unwrap();

    // texture atlas
    let texture = textures::load_textures(&display);
    // textured sprite shader
    let program = shaders::get_sprite_shader(&display);
    // selection shader
    let program_selection = shaders::get_selection_shader(&display);

    let params = glium::DrawParameters {
        blend: glium::Blend::alpha_blending(),
        .. Default::default()
    };
    let ortho_matrix: cgmath::Matrix4<f32> = cgmath::ortho(
        0.0, w as f32, h as f32, 0.0, 0.0, 1.0);
    let uniforms = uniform! {
        tex: &texture,
        matrix: Into::<[[f32; 4]; 4]>::into(ortho_matrix)
    };
    let mut start_ns = clock_ticks::precise_time_ns();
    let mut prev_ns = start_ns;
    let mut frames = 0;

    let mut running = true;
    let mut events = sdl_context.event_pump().unwrap();
    let mut prev_buttons = HashSet::new();
    let mut selection = selection::Selection::new();

    while running {
        let now_ns = clock_ticks::precise_time_ns();
        let tick_ns = now_ns - prev_ns;
        let tick_s = (tick_ns as f64) / 1_000_000_000f64;
        // get a mouse state
        let state = events.mouse_state();

        // Create a set of pressed Keys.
        let buttons = state.pressed_mouse_buttons().collect();

        // Get the difference between the new and old sets.
        let new_buttons = &buttons - &prev_buttons;
        let old_buttons = &prev_buttons - &buttons;

        selection.update(&state, &new_buttons, &old_buttons, &buttons);
        miners.update(tick_s as f32, &tiles);

        if selection.pressed {
            tiles.update_selected(&selection);
        }

        // building the vertex buffer and index buffers that will be filled with the data of
        // the sprites
        let (vertex_buffer, index_buffer) = generate_vertices(&display, &tiles.get_tiles());
        let (vertex_buffer_m, index_buffer_m) = generate_vertices(&display, &miners.get_tiles());
        // we must only draw the number of sprites that we have written in the vertex buffer
        // if you only want to draw 20 sprites for example, you should pass `0 .. 20 * 6` instead
        let ib_slice = index_buffer.slice(0 .. sprites_count * 6).unwrap();
        let ib_slice_m = index_buffer_m.slice(0 .. miners_count * 6).unwrap();

        // drawing a frame
        let mut target = display.draw();
        target.clear_color_and_depth((0.0, 0.0, 0.0, 0.0), 1.0);
        target.draw(&vertex_buffer, &ib_slice, &program, &uniforms, &params).unwrap();
        target.draw(&vertex_buffer_m, &ib_slice_m, &program, &uniforms, &params).unwrap();

        // Event loop: polls for events sent to all windows
        for event in events.poll_iter() {
            match event {
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } |
                    Event::Quit { .. } => {
                        running = false;
                    },
                _ => ()
            }
        }

        if selection.pressed {  // draw current selection, if active
            let (vertex_buffer, index_buffer) = selection.generate_vertices(&display);
            target.draw(&vertex_buffer, &index_buffer, &program_selection, &uniforms, &params).unwrap();
        }

        target.finish().unwrap();

        prev_buttons = buttons;

        // calculate FPS
        frames += 1;
        let duration_ns = clock_ticks::precise_time_ns() - start_ns;
        let duration_s = (duration_ns as f64) / 1_000_000_000f64;
        let fps = (frames as f64) / duration_s;
        if duration_s > 1.0 {
            start_ns = clock_ticks::precise_time_ns();
            frames = 0;
            println!("Duration: {}, count: {}", duration_s, fps);
        }
        prev_ns = now_ns;
    }
}
