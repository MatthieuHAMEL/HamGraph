use sdl2::rect::Rect;
use tracing::debug;
use crate::{Renderer, action::Action, action_bus::ActionBus, hg::HamID, layout_manager::Layout, scene::Scene};

pub struct TextScene {
  text: String, 
  idx_text: usize, // The index of the to-be-created texture
  pos: Rect, 
  size: String // e.g. "big" or "42"
}

impl TextScene {
  pub fn new(text: String, size: String) -> Self {
    Self {text, idx_text: 0, pos: Rect::new(25, 25, 100, 100), size}
  }
}

impl Scene for TextScene {
  fn init(&mut self, action_bus: &mut ActionBus) {
    debug!(target: "hg::ttf", "Initializing Text Scene");

    let sprid = action_bus.push(Action::CreateText{
      font: "VcrOsdMono".to_owned(), // TODO set a "default font"
      size: self.size.clone(), 
      text: self.text.clone(),
    }).unwrap();

    self.idx_text = if let HamID::SpriteID(id) = sprid { id } else { unreachable!() };
    debug!(target: "hg::ttf", "ActionBus says my sprite will have the ID {}", self.idx_text);

    action_bus.push(Action::RequestLayout(Layout { ..Default::default() }));
  }

  fn render(&self, renderer: &mut Renderer) {
    renderer.render_sprite(self.idx_text, self.pos.x, self.pos.y, None);
    // TODO [BUG] it appears that this can render something else than a text sprite ...
  }

  fn pos_changed(&mut self, pos: Rect) {
    debug!(target: "hg::ttf", "pos_changed {:?}", pos);
    self.pos = pos;   
  }
}
