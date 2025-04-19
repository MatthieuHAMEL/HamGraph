use sdl2::render::{Texture, TextureCreator};
use sdl2::surface::Surface;
use sdl2::video::WindowContext;

use crate::errors::prompt_err_and_panic;
use crate::infraglobals;

pub struct TextureStore<'a> {
  textures: Vec<Option<Texture<'a>>>,  // The main texture holder
  filenames: Vec<String>, // In their json declaration order
  texture_creator: &'a TextureCreator<WindowContext> // SDL2 thing to load textures
}

impl<'a> TextureStore<'a> {
  pub fn new(texture_creator: &'a mut TextureCreator<WindowContext>, 
             initial_texture_count: usize) -> TextureStore<'a> {
    TextureStore { 
      textures: Vec::with_capacity(initial_texture_count), 
      filenames: Vec::new(), 
      texture_creator 
    }
  }

  pub fn get_texture(&mut self, texture_id: usize) -> &Texture {
    if self.textures[texture_id].is_none() {
      use sdl2::surface::Surface;
      use sdl2::image::LoadSurface;

      let img_path = infraglobals::get_img_path().join(&self.filenames[texture_id]);
      let s = Surface::from_file(img_path) // TODO use sdl_img shortcut (file --> texture directly)
        .unwrap_or_else(|err| { prompt_err_and_panic("img_load_color_key failed", &err, None); });
        
      let tex = s.as_texture(self.texture_creator)
        .unwrap_or_else(|err| { 
          prompt_err_and_panic("img_load_color_key(as_texture) failed", &err.to_string(), None); });
      
      self.textures[texture_id] = Some(tex);
    }

    self.textures[texture_id].as_ref().unwrap()
  }

  // Ensures the size of filenames remains the same than textures.
  pub fn push_new_texture(&mut self, path: String, opt_surface: Option<Surface>) -> usize {
    if let Some(surface) = opt_surface {
      if !path.is_empty() {
        panic!("Direct texture creation from surface is only supported for TTF");
      } // cause it wouldnt be lazy texture creation.... 
      let texture = self.texture_creator.create_texture_from_surface(&surface).unwrap();
      self.textures.push(Some(texture));
    }
    else {
      self.textures.push(None); // In most cases texture is created lazily
    }

    self.filenames.push(path);
    return self.textures.len() - 1;
  }

  pub fn set_alpha(&mut self, texture_id: usize, alpha: u8) {
    { // Just ensure the texture is loaded 
      let _t = self.get_texture(texture_id);
    }

    self.textures[texture_id].as_mut().unwrap().set_blend_mode(sdl2::render::BlendMode::Blend);
    self.textures[texture_id].as_mut().unwrap().set_alpha_mod(alpha);
  }
}

/////////////////////////////////////////////

// NB: Run with cargo nextest (1 test per process)
// With "cargo test" I can only serialise in the same process
// ... or put them in integration tests but it'll generate N exes...
#[cfg(test)]
mod tests {
  use crate::init::init_sdl2;

  use super::*;

  fn init_sdl2_context() -> TextureCreator<WindowContext> {
    infraglobals::setup_test_folder();

    let (_sdl_ctx, _img_ctx, _ttf_ctx, _video, _mixer_ctx, canvas) 
      = init_sdl2("HAMGRAPH TEST", 300, 400);

    canvas.texture_creator()
  }

  fn load_some_textures<'a>(tc: &'a mut TextureCreator<WindowContext>) -> TextureStore<'a> {
    let mut texture_store = TextureStore::new(tc, 10);
    assert!(texture_store.textures.is_empty());
    assert!(texture_store.filenames.is_empty());

    for _ in 0..5 {
      texture_store.push_new_texture(
        "test_sprite.png".to_string(), None);
      texture_store.push_new_texture(
        "test_sprite_2.png".to_string(), None);
    }

    texture_store
  }

  #[test]
  fn test_texturemap_get_texture() {
    let mut ctx = init_sdl2_context();
    let mut texture_store = load_some_textures(&mut ctx);
    for i in 0..10 {
      texture_store.get_texture(i);
    }
  }

  #[test]
  #[should_panic]
  fn test_texturemap_get_texture_out_of_bounds() {
    let mut ctx = init_sdl2_context();
    let mut texture_store = load_some_textures(&mut ctx);
    texture_store.get_texture(10); // Out of bounds
  }

  #[test]
  fn test_texturemap_set_alpha() {
    let mut ctx = init_sdl2_context();
    let mut texture_store = load_some_textures(&mut ctx);
    for i in 0..10 {
      texture_store.set_alpha(i, u8::try_from(i*5).unwrap());
    }
  }

  #[test]
  #[should_panic]
  fn test_texturemap_set_alpha_out_of_bounds() {
    let mut ctx = init_sdl2_context();
    let mut texture_store = load_some_textures(&mut ctx);
    texture_store.set_alpha(10, 42); // Out of bounds
  }
}
