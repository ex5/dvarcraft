use gfx::memory::Typed;
use gfx::queue::GraphicsQueue;
use gfx::{Adapter, Backend, CommandQueue, FrameSync, GraphicsCommandPool,
          Swapchain, QueueType, WindowExt};
use gfx::format::{Rgba8, DepthStencil};
use clock_ticks;
use env_logger;
use gfx;
use gfx_core;
use gfx_device_gl;
use gfx_window_sdl;
use sdl2;
use sdl2::pixels::Color;
use std;


pub mod shade;

pub type ColorFormat = gfx::format::Rgba8;

pub type DepthFormat = gfx::format::DepthStencil;

pub type BackbufferView<R: gfx::Resources> = (gfx::handle::RenderTargetView<R, ColorFormat>,
                                              gfx::handle::DepthStencilView<R, DepthFormat>);

pub struct WindowTargets<R: gfx::Resources> {
    pub views: Vec<BackbufferView<R>>,
    pub aspect_ratio: f32,
}

pub struct SyncPrimitives<R: gfx::Resources> {
    /// Semaphore will be signalled once a frame is ready.
    pub acquisition: gfx::handle::Semaphore<R>,
    /// Indicates that rendering has been finished.
    pub rendering: gfx::handle::Semaphore<R>,
    /// Sync point to ensure no resources are in-use.
    pub frame_fence: gfx::handle::Fence<R>,
}

fn run<A, B, S>((width, height): (u32, u32),
                    mut surface: S,
                    adapters: Vec<B::Adapter>,
                    sdl_context: sdl2::Sdl,
                    font: sdl2::ttf::Font)
    where A: Sized + Application<B>,
          B: Backend,
          S: gfx_core::Surface<B>,
          B::Device: shade::ShadeExt,
{
    use self::shade::ShadeExt;
    use gfx::format::Formatted;
    use gfx::traits::Device;
    use gfx::texture;

    // Init device, requesting (at least) one graphics queue with presentation support
    let gfx_core::Gpu { mut device, mut graphics_queues, .. } =
        adapters[0].open_with(|family, ty| ((ty.supports_graphics() && surface.supports_queue(&family)) as u32, QueueType::Graphics));
    let mut queue = graphics_queues.pop().expect("Unable to find a graphics queue.");

    let config = gfx_core::SwapchainConfig::new()
                    .with_color::<ColorFormat>()
                    .with_depth_stencil::<DepthFormat>();
    let mut swap_chain = surface.build_swapchain(config, &queue);



    let views =
        swap_chain
            .get_backbuffers()
            .iter()
            .map(|&(ref color, ref ds)| {
                let color_desc = texture::RenderDesc {
                    channel: ColorFormat::get_format().1,
                    level: 0,
                    layer: None,
                };
                let rtv = device.view_texture_as_render_target_raw(color, color_desc)
                                 .unwrap();

                let ds_desc = texture::DepthStencilDesc {
                    level: 0,
                    layer: None,
                    flags: texture::DepthStencilFlags::empty(),
                };
                let dsv = device.view_texture_as_depth_stencil_raw(
                                    ds.as_ref().unwrap(),
                                    ds_desc)
                                 .unwrap();

                (Typed::new(rtv), Typed::new(dsv))
            })
            .collect();

    let shader_backend = device.shader_backend();
    let mut app = A::new(&mut device, &mut queue, shader_backend, WindowTargets {
        views: views,
        aspect_ratio: width as f32 / height as f32, //TODO
    });

    // TODO: For optimal performance we should use a ring-buffer
    let sync = SyncPrimitives {
        acquisition: device.create_semaphore(),
        rendering: device.create_semaphore(),
        frame_fence: device.create_fence(false),
    };

    let mut graphics_pool = queue.create_graphics_pool(1);
    let mut events = sdl_context.event_pump().unwrap();

    let mut frames = 0;
    let mut start_ns = clock_ticks::precise_time_ns();
    let mut prev_ns = start_ns;
    let mut fps = 0.0;

    while app.is_running() {
        let now_ns = clock_ticks::precise_time_ns();
        let tick_ns = now_ns - prev_ns;
        let tick_s = (tick_ns as f64) / 1_000_000_000f64;

        app.update(tick_s as f32, &mut events);

        graphics_pool.reset();
        let frame = swap_chain.acquire_frame(FrameSync::Semaphore(&sync.acquisition));
        // render some text into an SDL surface
        let text_surface = font.render(&format!("FPS {:.1}", fps))
            .blended(Color::RGBA(200, 200, 200, 255)).unwrap();
        app.render(&mut device, (frame, &sync), &mut graphics_pool, &mut queue, text_surface);
        swap_chain.present(&mut queue, &[]);

        device.wait_for_fences(&[&sync.frame_fence], gfx::WaitFor::All, 1_000_000);
        queue.cleanup();

        // calculate FPS
        frames += 1;
        let duration_ns = clock_ticks::precise_time_ns() - start_ns;
        let duration_s = (duration_ns as f64) / 1_000_000_000f64;
        fps = (frames as f64) / duration_s;
        if duration_s > 1.0 {
            start_ns = clock_ticks::precise_time_ns();
            frames = 0;
            println!("Duration: {}, count: {}", duration_s, fps);
        }
        prev_ns = now_ns;
    }
}

pub type DefaultBackend = gfx_device_gl::Backend;

pub trait Application<B: Backend>: Sized {
    fn is_running(&self) -> bool;
    fn new(&mut B::Device, &mut GraphicsQueue<B>,
           shade::Backend, WindowTargets<B::Resources>) -> Self;
    fn update(&mut self, tick: f32, events: &mut sdl2::EventPump);
    fn render(&mut self, device: &mut B::Device, frame: (gfx_core::Frame, &SyncPrimitives<B::Resources>),
                     pool: &mut GraphicsCommandPool<B>, queue: &mut GraphicsQueue<B>, text_surface: sdl2::surface::Surface);

    fn on_resize(&mut self, WindowTargets<B::Resources>) {}
    fn on_resize_ext(&mut self, _device: &mut B::Device, targets: WindowTargets<B::Resources>) {
        self.on_resize(targets);
    }

    fn launch_simple(w: u32, h: u32) where Self: Application<DefaultBackend> {
        env_logger::init().unwrap();
        <Self as Application<DefaultBackend>>::launch_default(w, h)
    }
    fn launch_default(w: u32, h: u32)
        where Self: Application<DefaultBackend>
    {
        use gfx_core::format::Formatted;

        let sdl_context = sdl2::init().unwrap();
        let video = sdl_context.video().unwrap();

        let ttf_context = sdl2::ttf::init().unwrap();
        // Load a font
        let mut font = ttf_context.load_font("assets/FiraSans-Regular.ttf", 128).unwrap();

        // Request opengl core 3.3:
        video.gl_attr().set_context_profile(sdl2::video::GLProfile::Core);
        video.gl_attr().set_context_version(3, 3);
        let builder = video.window("SDL Window", w, h);
        let (window, _gl_context) = gfx_window_sdl::build(builder, Rgba8::get_format(), DepthStencil::get_format()).unwrap();
        let mut window = gfx_window_sdl::Window::new(window);
        let (mut surface, adapters) = window.get_surface_and_adapters();
        //let mut canvas = window.raw().into_canvas();
        //println!("{:?}", text_surface.without_lock());

        let dim = (w, h);
        run::<Self, _, _>(dim, surface, adapters, sdl_context, font)
    }
}
