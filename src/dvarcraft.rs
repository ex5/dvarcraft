#[macro_use] extern crate gfx;
extern crate env_logger;
extern crate rand;
extern crate sdl2;
extern crate gfx_core;
extern crate freetype as ft;
//extern crate gfx_corell;
extern crate gfx_device_gl;
extern crate gfx_window_sdl;
extern crate cgmath;
extern crate image;
extern crate find_folder;
extern crate clock_ticks;

mod support;
mod textures;
mod selection;
mod quadtree;
mod miners;
mod tiles;

use gfx::{Device, GraphicsPoolExt};
use support::{BackbufferView, ColorFormat};
use std::collections::HashSet;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

const SPRITE_WIDTH: f32 = 32.0;
const SPRITE_VERTICES: [Vertex; 4] = [
    Vertex { position: [-SPRITE_WIDTH,  SPRITE_WIDTH] },
    Vertex { position: [ SPRITE_WIDTH,  SPRITE_WIDTH] },
    Vertex { position: [-SPRITE_WIDTH, -SPRITE_WIDTH] },
    Vertex { position: [ SPRITE_WIDTH, -SPRITE_WIDTH] },
];

const SPRITE_INDICES: [u16; 6] = [0, 1, 2, 1, 3, 2];

const SPRITE_MINER: u32 = 0;
const SPRITE_WATER: u32 = 1;
const SPRITE_GRASS: u32 = 2;
const SPRITE_CLAY: u32 = 3;
const SPRITE_STONE: u32 = 4;
const SPRITE_TREE: u32 = 5;
const SPRITE_WOOD: u32 = 6;

gfx_defines!{
    vertex Vertex {
        position: [f32; 2] = "i_position",
    }

    // color format: 0xRRGGBBAA
    vertex Instance {
        translate: [f32; 2] = "a_Translate",
        tex_id: u32 = "i_tex_id",
        is_selected: f32 = "is_selected",
    }

    pipeline pipe {
        vertex: gfx::VertexBuffer<Vertex> = (),
        instance: gfx::InstanceBuffer<Instance> = (),
        scale: gfx::Global<f32> = "u_Scale",
        matrix: gfx::Global<[[f32; 4]; 4]> = "matrix",
        tex: gfx::TextureSampler<[f32; 4]> = "tex",
        out: gfx::BlendTarget<ColorFormat> = ("f_color", gfx::state::MASK_ALL, gfx::preset::blend::ALPHA),
    }
}

fn fill_instances(instances: &mut [Instance], start_idx: usize, tiles: &Vec<&tiles::Tile>) {
    for (i, tile) in tiles.iter().enumerate() {
        instances[start_idx + i] = Instance {
            translate: [tile.position.x, tile.position.y],
            tex_id: tile.tex_id,
            is_selected: match tile.is_selected { true => 0.5, false => 0.0 },
        };
    }
 }

pub struct App<B: gfx::Backend> {
    running: bool,
    zoom: f32,
    viewport_w: f32,
    viewport_h: f32,
    views: Vec<BackbufferView<B::Resources>>,
    pso: gfx::PipelineState<B::Resources, pipe::Meta>,
    data: pipe::Data<B::Resources>,
    data_ui: pipe::Data<B::Resources>,
    slice: gfx::Slice<B::Resources>,
    slice_ui: gfx::Slice<B::Resources>,
    tiles: tiles::Tiles,
    miners: miners::Miners,
    instance_count: usize,
    prev_buttons: HashSet<sdl2::mouse::MouseButton>,
    selection: selection::Selection,
    cur_tile: Option<usize>,
}

impl<B: gfx::Backend> support::Application<B> for App<B> {
    fn new(device: &mut B::Device,
           _: &mut gfx::queue::GraphicsQueue<B>,
           backend: support::shade::Backend,
           window_targets: support::WindowTargets<B::Resources>) -> Self
    {
        use gfx::traits::DeviceExt;
        //use gfx_corell::factory::Factory;

        let vs = support::shade::Source {
            glsl_330: include_bytes!("shader/instancing_120.glslv"),
            .. support::shade::Source::empty()
        };
        let fs = support::shade::Source {
            glsl_330: include_bytes!("shader/instancing_120.glslf"),
            .. support::shade::Source::empty()
        };

        let mut tiles = tiles::Tiles::new_layer_from_heightmap("heightmap_64.png", 2);
        let mut miners = miners::Miners::new(10, &tiles);
        let miners_count: usize = miners.miners.len();
        let sprites_count: usize = tiles.tiles.len();
        let instance_count = sprites_count + miners_count;
        println!("Number of sprites: {}", instance_count);

        let zoom = 1.0;
        let (viewport_w, viewport_h) = (800.0, 600.0);
        let ortho_matrix: cgmath::Matrix4<f32> = cgmath::ortho(
            - viewport_w / 2.0 * zoom, viewport_w / 2.0 * zoom,
            - viewport_h / 2.0 * zoom, viewport_h / 2.0 * zoom,
            -1.0, 1.0);
        let (quad_vertices, slice) = device
            .create_vertex_buffer_with_slice(&SPRITE_VERTICES, &SPRITE_INDICES[..]);
        let instances = device
            .create_buffer(instance_count,
                           gfx::buffer::Role::Vertex,
                           gfx::memory::Usage::Data,
                           gfx::TRANSFER_DST).unwrap();

        let (quad_vertices_ui, slice_ui) = device
            .create_vertex_buffer_with_slice(&SPRITE_VERTICES, &SPRITE_INDICES[..]);
        let instances_ui = device
            .create_buffer(1,
                           gfx::buffer::Role::Vertex,
                           gfx::memory::Usage::Data,
                           gfx::TRANSFER_DST).unwrap();
        App {
            running: true,
            zoom: zoom,
            viewport_w: viewport_w,
            viewport_h: viewport_h,
            miners: miners,
            tiles: tiles,
            instance_count: instance_count,
            slice: slice,
            slice_ui: slice_ui,
            data: pipe::Data {
                vertex: quad_vertices,
                instance: instances,
                scale: 1.0,
                matrix: Into::<[[f32; 4]; 4]>::into(ortho_matrix),
                // texture atlas
                tex: (textures::load_textures(device), device.create_sampler_linear()),
                out: window_targets.views[0].0.clone(),
            },
            data_ui: pipe::Data {
                vertex: quad_vertices_ui,
                instance: instances_ui,
                scale: 1.0,
                matrix: Into::<[[f32; 4]; 4]>::into(cgmath::ortho(
                    - viewport_w / 2.0 * zoom, viewport_w / 2.0 * zoom,
                    viewport_h / 2.0 * zoom, - viewport_h / 2.0 * zoom,
                    - 1.0, 1.0
                )),
                tex: (textures::load_textures(device), device.create_sampler_linear()),
                out: window_targets.views[0].0.clone(),
            },
            pso: device.create_pipeline_simple(
                vs.select(backend).unwrap(),
                fs.select(backend).unwrap(),
                pipe::new()
                ).unwrap(),
            views: window_targets.views,
            prev_buttons: HashSet::new(),
            selection: selection::Selection::new(),
            cur_tile: None,
        }
    }

    fn render(&mut self, device: &mut B::Device, (frame, sync): (
            gfx::Frame, &support::SyncPrimitives<B::Resources>),
            pool: &mut gfx::GraphicsCommandPool<B>,
            queue: &mut gfx::queue::GraphicsQueue<B>,
            text_surface: sdl2::surface::Surface)
    {
        use gfx::traits::DeviceExt;

        let upload = device.create_upload_buffer(self.instance_count).unwrap();
        {
            let mut writer = device.write_mapping(&upload).unwrap();
            fill_instances(&mut writer, 0, &self.tiles.get_tiles());
            fill_instances(&mut writer, self.tiles.tiles.len(), &self.miners.get_tiles());
        };

        self.slice.instances = Some((self.instance_count as u32, 0));
        self.data.instance = device
            .create_buffer(self.instance_count,
                           gfx::buffer::Role::Vertex,
                           gfx::memory::Usage::Data,
                           gfx::TRANSFER_DST).unwrap();
        self.data.matrix = Into::<[[f32; 4]; 4]>::into(cgmath::ortho(
            - self.viewport_w / 2.0 * self.zoom, self.viewport_w / 2.0 * self.zoom,
            - self.viewport_h / 2.0 * self.zoom, self.viewport_h / 2.0 * self.zoom,
            -1.0, 1.0));

        let mut encoder = pool.acquire_graphics_encoder();
        encoder.copy_buffer(&upload, &self.data.instance,
                            0, 0, upload.len()).unwrap();
        self.data.out = self.views[frame.id()].clone().0;
        encoder.clear(&self.data.out, [0.1, 0.2, 0.3, 1.0]);
        encoder.draw(&self.slice, &self.pso, &self.data);

        // update UI pipeline data
        let text_upload = device.create_upload_buffer(1).unwrap();
        {
            let mut writer = device.write_mapping(&text_upload).unwrap();
            fill_instances(&mut writer, 0, &vec![self.tiles.get_tiles()[1500]]);
        };
        self.data_ui.instance = device
            .create_buffer(1,
                           gfx::buffer::Role::Vertex,
                           gfx::memory::Usage::Data,
                           gfx::TRANSFER_DST).unwrap();
        self.data_ui.tex = (textures::texture_from_text(device, text_surface), device.create_sampler_linear());
        self.data_ui.out = self.views[frame.id()].clone().0;
        encoder.copy_buffer(&text_upload, &self.data_ui.instance,
                            0, 0, text_upload.len()).unwrap();
        // make a separate draw call for the UI pipeline data
        encoder.draw(&self.slice_ui, &self.pso, &self.data_ui);

        encoder.synced_flush(queue, &[&sync.rendering], &[], Some(&sync.frame_fence))
               .expect("Could not flush encoder");
    }

    fn on_resize(&mut self, window_targets: support::WindowTargets<B::Resources>) {
        self.views = window_targets.views;
    }

    fn update(&mut self, tick: f32, events: &mut sdl2::EventPump) {
        // get a mouse state
        let state = events.mouse_state();

        // Create a set of pressed Keys.
        let buttons = state.pressed_mouse_buttons().collect();

        // Get the difference between the new and old sets.
        let new_buttons = &buttons - &self.prev_buttons;
        let old_buttons = &self.prev_buttons - &buttons;

        let x = state.x() as f32 - self.viewport_w / 2.0;
        let y = self.viewport_h / 2.0 - state.y() as f32;

        // right mouse click
        let right = &sdl2::mouse::MouseButton::Right;
        if new_buttons.contains(right) {
            println!("Mouse coord: {:?}, {:?}", x, y);
            let picked_tile_id = self.tiles.tree.find(&cgmath::Vector2::new(x * self.zoom, y * self.zoom));
            if self.cur_tile.is_some() {
                self.tiles.tiles[self.cur_tile.unwrap()].is_selected = false;
            }
            if picked_tile_id.is_some() {
                let sel_id = picked_tile_id.unwrap();
                self.tiles.tiles[sel_id].is_selected = true;
                println!("Sprite coords: {:?}", self.tiles.tiles[sel_id].position);
                self.cur_tile = Some(sel_id);
            }
            println!("Clicked at tile: {:?}", picked_tile_id);
        }
        self.selection.update(x, y, &new_buttons, &old_buttons, &buttons);
        self.miners.update(tick as f32, &self.tiles);

        if self.selection.pressed {
            self.tiles.update_selected(&self.selection);
        }

        // handle events
        for event in events.poll_iter() {
            match event {
                Event::Quit { .. } |
                Event::KeyUp { keycode: Some(Keycode::Escape), .. } => {
                    self.running = false;
                },
                Event::KeyDown { keycode: Some(Keycode::Equals), .. } => {
                    self.zoom -= 0.5;
                },
                Event::KeyDown { keycode: Some(Keycode::Minus), .. } => {
                    self.zoom += 0.5;
                },
                _ => {}
            }
        }
    }

    fn is_running(&self) -> bool {
        self.running
    }
}

pub fn main() {
    use support::Application;
    App::launch_simple(800, 600);
}
