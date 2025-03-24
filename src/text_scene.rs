use sdl2::{rect::Rect, render::Canvas, video::Window};
use crate::{action::Action, action_bus::ActionBus, hg::HamID, layout_manager::Layout, scene::Scene, sprite::SpriteStore};

pub struct TextScene {
  text: String, 
  idx_text: usize, // The index of the to-be-created 
  pos: Rect, 
  size: String // e.g. "big" or "42"
}

impl TextScene {
  pub fn new(text: String, size: String) -> Self {
    Self {text, idx_text: 0, pos: Rect::new(0, 0, 100, 100), size}
  }
}

impl Scene for TextScene {
  fn init(&mut self, action_bus: &mut ActionBus) {
    println!("Initializing Text Scene.");

    if let Some(HamID::SpriteID(sprid)) = action_bus.push(Action::CreateText{
      font: "VcrOsdMono".to_owned(), 
      size: self.size.clone(), 
      text: self.text.clone(),
    }) 
    { // TODO set a "default font"
      self.idx_text = sprid;
      println!("ActionBus says my sprite will have the ID {}", self.idx_text);
    } else {
      panic!("Contract error [28] in text scene");
    }

    action_bus.push(Action::RequestLayout(Layout {
      ..Default::default()
    }));
  }

  fn render(&self, renderer: &mut Canvas<Window>, sprites: &mut SpriteStore) -> Result<(), String> {
    sprites.render(renderer, self.idx_text, self.pos.x, self.pos.y, None);
    // TODO [BUG] it appears that this can render something else than a text sprite ...
    Ok(())
  }

  fn pos_changed(&mut self, pos: Rect) {
    self.pos = pos;   
  }
}
