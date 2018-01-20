use find_folder;
use image;
use gfx;
use sdl2;

use support::ColorFormat;

pub fn load_textures<R, D>(device: &mut D) -> gfx::handle::ShaderResourceView<R, [f32; 4]>
    where R: gfx::Resources, D: gfx::Device<R>
{
    let assets = find_folder::Search::ParentsThenKids(3, 3)
        .for_folder("assets").unwrap();

    // store all textures in a `Texture2dArray`
    let tex_files = vec!["grass.png", "water.png", "miner.png"];

    let texture = {
        let images = tex_files.iter().map(|x| {
            image::open(assets.join(x)).unwrap().rotate180().to_rgba()
        }).collect::<Vec<_>>();

        let data: [&[u8]; 3] = [&images[0], &images[1], &images[2]];

        device.create_texture_immutable_u8::<ColorFormat>(
            gfx::texture::Kind::D2Array(64, 64, 3, gfx::texture::AaMode::Single),
            &data
            ).unwrap().1
    };
    texture
}

pub fn texture_from_text<R, D>(device: &mut D, text_surface: sdl2::surface::Surface) -> gfx::handle::ShaderResourceView<R, [f32; 4]>
    where R: gfx::Resources, D: gfx::Device<R>
{
    let texture = {
        let (w, h) = text_surface.size();
        device.create_texture_immutable_u8::<ColorFormat>(
            gfx::texture::Kind::D2Array(w as u16, h as u16, 1, gfx::texture::AaMode::Single),
            &[text_surface.without_lock().unwrap()]
            ).unwrap().1
    };
    texture
}
