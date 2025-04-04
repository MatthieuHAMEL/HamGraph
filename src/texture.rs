use sdl2::render::{Texture, TextureCreator};
use sdl2::surface::Surface;
use sdl2::video::WindowContext;

use crate::errors::prompt_err_and_panic;
use crate::infraglobals;

pub struct TextureMap<'a>
{
  textures: Vec<Option<Texture<'a>>>,  // The main texture holder
  filenames: Vec<String>, // In their json declaration order
  texture_creator: &'a TextureCreator<WindowContext> // SDL2 thing to load textures
}

impl<'a> TextureMap<'a>
{
  pub fn new(texture_creator: &'a mut TextureCreator<WindowContext>) -> TextureMap<'a>
  {
    TextureMap { textures: Vec::new(), filenames: Vec::new(), texture_creator }
  }

  pub fn get_texture(&mut self, texture_id: usize) -> &Texture
  {
    if self.textures[texture_id].is_none() 
    {
      use sdl2::surface::Surface;
      use sdl2::image::LoadSurface;

      let img_path = infraglobals::get_img_path().join(&self.filenames[texture_id]);
      let /*mut*/ s = Surface::from_file(img_path)
        .unwrap_or_else(|err| { prompt_err_and_panic("img_load_color_key failed", &err, None); });
      
      /* TODO color keying 
      it seems it is not needed with modern textures and alpha channel 
      then we could use directly the load texture function from SDL_IMG_... 
      match color_key 
      {
        Some(col) => 
        { 
          s.set_color_key(true, col)
          .unwrap_or_else(|err| { prompt_err_and_panic("img_load_color_key(set_color_key) failed", &err, None); }); 
        },
        None => {}
      } */
        
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
      }
      let texture = self.texture_creator
        .create_texture_from_surface(&surface).unwrap();
      self.textures.push(Some(texture));
    }
    else {
      self.textures.push(None); // In most cases texture is created lazily
    }

    self.filenames.push(path);

    self.textures.len() - 1
  }

  pub fn set_alpha(&mut self, texture_id: usize, alpha: u8) {
    // Just ensure it is loaded 
    {
      let _t = self.get_texture(texture_id);
    }

    self.textures[texture_id].as_mut().unwrap().set_blend_mode(sdl2::render::BlendMode::Blend);
    self.textures[texture_id].as_mut().unwrap().set_alpha_mod(alpha);
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn test_texturemap() {
    /*
    let (sdl_ctx, img_ctx, ttf_ctx, video, mixer_ctx, canvas) 
      = init_sdl2("HAMGRAPH TEST",300,400);

    let texture_map = TextureMap::new(canvas);
    */
  }
}