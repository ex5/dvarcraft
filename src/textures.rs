use find_folder;
use image;
use gfx;
use sdl2;
const SPRITE_COUNT: usize = 7;

use support::ColorFormat;

pub fn load_textures<R, D>(device: &mut D) -> gfx::handle::ShaderResourceView<R, [f32; 4]>
    where R: gfx::Resources, D: gfx::Device<R>
{
    let assets = find_folder::Search::ParentsThenKids(3, 3)
        .for_folder("assets").unwrap();

    // store all textures in a `Texture2dArray`
    let tex_files = vec![
        "miner.png",
        "water.png",
        "grass.png", "clay.png", "stone.png",
        "tree.png", "wood.png"];

    let texture = {
        let images = tex_files.iter().map(|x| {
            image::open(assets.join(x)).unwrap().rotate180().to_rgba()
        }).collect::<Vec<_>>();

        let data: [&[u8]; SPRITE_COUNT] = [
            &images[0],
            &images[1],
            &images[2], &images[3], &images[4],
            &images[5], &images[6]];

        device.create_texture_immutable_u8::<ColorFormat>(
            gfx::texture::Kind::D2Array(64, 64, SPRITE_COUNT as u16, gfx::texture::AaMode::Single),
            &data
            ).unwrap().1
    };
    texture
}

pub fn texture_from_surface<R, D>(device: &mut D, surface: sdl2::surface::Surface) -> gfx::handle::ShaderResourceView<R, [f32; 4]>
    where R: gfx::Resources, D: gfx::Device<R>
{
    let texture = {
        let (w, h) = surface.size();
        device.create_texture_immutable_u8::<ColorFormat>(
            gfx::texture::Kind::D2Array(w as u16, h as u16, 1, gfx::texture::AaMode::Single),
            &[surface.without_lock().unwrap()]
            ).unwrap().1
    };
    texture
}
