#[macro_use]
extern crate glium;
extern crate rand;
extern crate clock_ticks;
extern crate cgmath;
extern crate image;
extern crate vecmath;
extern crate find_folder;

extern crate glium_sdl2;
extern crate sdl2;

use glium::{ Surface };
use glium::index::PrimitiveType;
use glium::texture::Texture2dDataSource;
use glium_sdl2::{ DisplayBuild };
use sdl2::video::GLProfile;

mod tiles;
mod miners;

#[derive(Copy, Clone)]
struct Vertex {
    i_position: [f32; 2],
    i_tex_id: u32,
}
implement_vertex!(Vertex, i_position, i_tex_id);


//let display = glium::Display::new(window, context, &events_loop).unwrap();
fn generate_vertices(display: &glium_sdl2::SDL2Facade, tiles: &Vec<&tiles::Tile>) -> (glium::VertexBuffer<Vertex>, glium::index::IndexBuffer<u16>) {
    let sprites_count = tiles.len();
    let mut vb: glium::VertexBuffer<Vertex> = glium::VertexBuffer::empty_dynamic(
        display, sprites_count * 4).unwrap();
    let mut ib_data = Vec::with_capacity(sprites_count * 6);

    // initializing with positions and texture IDs
    for (num, sprite) in vb.map().chunks_mut(4).enumerate() {
        let tile = &tiles[num];
        let position = tile.position;
        let tex_id = tile.tex_id;

        sprite[0].i_position[0] = position.0 - 32.0;
        sprite[0].i_position[1] = position.1 + 32.0;
        sprite[0].i_tex_id = tex_id;
        sprite[1].i_position[0] = position.0 + 32.0;
        sprite[1].i_position[1] = position.1 + 32.0;
        sprite[1].i_tex_id = tex_id;
        sprite[2].i_position[0] = position.0 - 32.0;
        sprite[2].i_position[1] = position.1 - 32.0;
        sprite[2].i_tex_id = tex_id;
        sprite[3].i_position[0] = position.0 + 32.0;
        sprite[3].i_position[1] = position.1 - 32.0;
        sprite[3].i_tex_id = tex_id;

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
    let assets = find_folder::Search::ParentsThenKids(3, 3)
        .for_folder("assets").unwrap();

    let (w, h) = (800, 600);
    let tiles = tiles::Tiles::new_layer_from_heightmap("heightmap_64.png", 1);
    let mut miners = miners::Miners::new(10);
    let sprites_count: usize = tiles.tiles.len();
    println!("Number of sprites: {}", sprites_count);

    let ortho_matrix: cgmath::Matrix4<f32> = cgmath::ortho(
        0.0, w as f32, h as f32, 0.0, 0.0, 1.0);

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(GLProfile::Core);
    gl_attr.set_context_version(3, 3);

    let display = video_subsystem.window("Dvarcraft", w, h)
        //.resizable()
        .build_glium()
        .unwrap();
    // store all textures in a `Texture2dArray`
    let tex_files = vec!["grass.png", "water.png", "miner.png"];
    let texture = {
        let images = tex_files.iter().map(|&x| {
            let image = image::open(assets.join(x)).unwrap().to_rgba();
            let image_dimensions = image.dimensions();
            glium::texture::RawImage2d::from_raw_rgba(image.into_raw(), image_dimensions).into_raw()
        }).collect::<Vec<_>>();

        glium::texture::Texture2dArray::new(&display, images).unwrap()
    };

    // we determine the texture coordinates depending on the ID the of vertex
    let program = program!(&display,
       330 => {
           vertex: "
            #version 330
            in vec2 i_position;
            in uint i_tex_id;
            out vec2 v_tex_coords;
            flat out uint v_tex_id;
            uniform mat4 matrix;
            void main() {
                gl_Position = matrix * vec4(i_position, 0.0, 1.0);
                if (gl_VertexID % 4 == 0) {
                    v_tex_coords = vec2(0.0, 1.0);
                } else if (gl_VertexID % 4 == 1) {
                    v_tex_coords = vec2(1.0, 1.0);
                } else if (gl_VertexID % 4 == 2) {
                    v_tex_coords = vec2(0.0, 0.0);
                } else {
                    v_tex_coords = vec2(1.0, 0.0);
                }
                v_tex_id = i_tex_id;
            }
        ",

        fragment: "
            #version 330
            uniform sampler2DArray tex;
            in vec2 v_tex_coords;
            flat in uint v_tex_id;
            out vec4 f_color;
            void main() {
                f_color = texture(tex, vec3(v_tex_coords, float(v_tex_id)));
                /*
                if (f_color.a < 0.5) {
                    discard;
                }
                */
            }
        "
    },
    ).unwrap();

    let params = glium::DrawParameters {
        blend: glium::Blend::alpha_blending(),
        .. Default::default()
    };
    let uniforms = uniform! {
        tex: &texture,
        matrix: Into::<[[f32; 4]; 4]>::into(ortho_matrix)
    };
    let mut start_ns = clock_ticks::precise_time_ns();
    let mut frames = 0;

    let mut running = true;
    let mut event_pump = sdl_context.event_pump().unwrap();

    while running {
        // building the vertex buffer and index buffers that will be filled with the data of
        // the sprites
        let (vertex_buffer, index_buffer) = generate_vertices(&display, &tiles.get_tiles());
        let (vertex_buffer_m, index_buffer_m) = generate_vertices(&display, &miners.get_tiles());
        // we must only draw the number of sprites that we have written in the vertex buffer
        // if you only want to draw 20 sprites for example, you should pass `0 .. 20 * 6` instead
        let ib_slice = index_buffer.slice(0 .. sprites_count * 6).unwrap();
        let ib_slice_m = index_buffer_m.slice(0 .. 10 * 6).unwrap();

        // drawing a frame
        let mut target = display.draw();
        target.clear_color_and_depth((0.0, 0.0, 0.0, 0.0), 1.0);
        target.draw(&vertex_buffer, &ib_slice, &program, &uniforms, &params).unwrap();
        target.draw(&vertex_buffer_m, &ib_slice_m, &program, &uniforms, &params).unwrap();
        target.finish().unwrap();

        // Event loop: polls for events sent to all windows
        for event in event_pump.poll_iter() {
            use sdl2::event::Event;

            match event {
                Event::Quit { .. } => {
                    running = false;
                },
                _ => ()
            }
        }

        frames += 1;
        let duration_ns = clock_ticks::precise_time_ns() - start_ns;
        let duration_s = (duration_ns as f64) / 1_000_000_000f64;
        let fps = (frames as f64) / duration_s;
        if duration_s > 1.0 {
            start_ns = clock_ticks::precise_time_ns();
            frames = 0;
            println!("Duration: {}, count: {}", duration_s, fps);
        }

    }
}
