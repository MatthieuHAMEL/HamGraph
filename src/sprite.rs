use crate::errors::*;
use crate::font::FontStore;
use crate::texture::TextureStore;
use crate::infraglobals;

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{TextureCreator, WindowCanvas};
use sdl2::surface::Surface;
use sdl2::video::WindowContext;
use tracing::debug;
use std::cell::Cell;
use std::path::PathBuf;
use std::rc::Rc;

pub struct SpriteStore<'a>
{
  store: Vec<Sprite>,
  current_len: Rc<Cell<usize>>,
  texture_store: TextureStore<'a>,

  cached_text: Option<Surface<'a>>
}

impl<'a> SpriteStore<'a>
{
  // Loads the store from JSON files at the start of the game (without loading textures)
  pub fn new(texture_creator: &'a mut TextureCreator<WindowContext>) -> Self 
  {
    let json_path = infraglobals::get_conf_path().join("spritedesc.json");
    if !json_path.exists() {
      prompt_err_and_panic(&format!("SpriteStore Config file not found: {:?}", json_path), "", None)
    }

    // Vector of spritesheets which are themselves vectors of sprite JSON representations
    let v_jsheets = load_sprites_from_json(&json_path);
    let mut texture_store = TextureStore::new(texture_creator, v_jsheets.len());

    // Fill in the sprite store with sprites pointing to empty textures (no surface loading)
    let mut store = Vec::new();
    for jsheet in v_jsheets { // 1 sheet == 1 texture == N sprites
      let tex_id = texture_store.push_new_texture(jsheet.file, None);
      for js in jsheet.sprites { // js is the json representation of a sprite
        debug!(target: "hg::sprite", "Sprite {} made alone ? {:?}", js.name, js.make_alone);
        store.push(Sprite::new(Rect::new(js.x, js.y, js.w, js.h), tex_id));
      }
    }
    let cur_len: usize = store.len();
    SpriteStore { store, current_len: Rc::new(Cell::new(cur_len)), texture_store, cached_text: None }
  }

  pub fn render(&mut self, canvas: &mut WindowCanvas, sprite_id: usize, x: i32, y: i32, alpha: Option<u8>) {
    // Find the sprite metadata in the registry
    let sprite = &self.store[sprite_id];
    // If the texture hasn't been set yet, load it now
    // It may not be set as the texture of that sprite but the texture may have been loaded before!
    
    if let Some(alph) = alpha {
      self.texture_store.set_alpha(sprite.texture_id, alph);
    } 
    let tex = self.texture_store.get_texture(sprite.texture_id) ;
    let dest_rect = Rect::new(x, y, sprite.src_rect.width(), sprite.src_rect.height());
    canvas.copy(tex, sprite.src_rect, dest_rect).unwrap();
  }

  pub fn shared_len(&self) -> Rc<Cell<usize>> {
    Rc::clone(&self.current_len)
  }

  pub fn try_ttf_texture(&mut self, font_store: &FontStore, font_name: &str, text: String, max_width: u32) -> (u32, u32) {
    let font = font_store.get(font_name); 
      
    // Render text to a surface, and convert surface to a texture
    // TODO 
    self.cached_text = Some(font.render(&text).blended(Color::RGB(0, 0, 50)).unwrap());// TODO color custo 

    let w = self.cached_text.as_ref().unwrap().width();
    let h = self.cached_text.as_ref().unwrap().height();

    debug!(target: "hg::sprite", "try_ttf_texture w=<{}>, h=<{}>", w, h);
    (w, h)
  }

  pub fn commit_ttf_texture(&mut self) -> usize {
    if let Some(cached_text) = self.cached_text.as_ref() {
      let width = cached_text.width();
      let height = cached_text.height();
      let tex_id = self.texture_store.push_new_texture("".to_owned(), self.cached_text.take());
      self.store.push(Sprite::new(Rect::new(0, 0, width, height), tex_id));
      self.current_len.set(self.current_len.get() + 1);
      return tex_id;
    } else {
      prompt_err_and_panic("commit_ttf_texture failed: cached_text is None", "", None);
    }
  }
}

struct Sprite {
  src_rect: Rect,      // Source rectangle defining the sprite's portion in the texture
  //scenes: Vec<String>,  // Scenes where this sprite is used (TODO scenename)
  texture_id: usize // the index of the texture path in the texture path vector
}

impl Sprite {
  pub fn new(src_rect: Rect, texture_id: usize) -> Sprite {
    Sprite { src_rect, texture_id}
  }
}

// Represent deserialized sprite data
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct SpriteJsonRep {
  name: String,
  x: i32,
  y: i32,
  w: u32,
  h: u32,
  make_alone: Option<bool> // The sprite should have its own texture (TODO useful ?)
}

#[derive(Deserialize, Debug)]
pub struct SpriteSheetJsonRep {
  file: String,
  sprites: Vec<SpriteJsonRep>,
}

pub type SpriteDescJsonRep = Vec<SpriteSheetJsonRep>;

// Deserialize sprite data from json
use std::fs::File;
use std::io::BufReader;
use serde_json::from_reader;

pub fn load_sprites_from_json(file_path: &PathBuf) -> SpriteDescJsonRep {
  let file = File::open(file_path)
    .unwrap_or_else(|err| { prompt_err_and_panic("load_sprites_from_json failed(open)", &err.to_string(), None); });
  let reader = BufReader::new(file);
  let sprite_data: SpriteDescJsonRep = from_reader(reader)
    .unwrap_or_else(|err| { prompt_err_and_panic("load_sprites_from_json failed(read)", &err.to_string(), None); });
  
  sprite_data
}
