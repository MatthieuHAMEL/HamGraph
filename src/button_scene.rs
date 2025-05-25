// Temporary placed here. To be part of HamUI. 
use crate::{action::{Action, EventKind}, action_bus::ActionBus, Renderer, layout_manager::Layout, scene::{Scene, SceneID}, text_scene::TextScene, utils::is_point_in_rect};
use sdl2::{event::Event, mouse::MouseButton, pixels::Color, rect::Rect};
use tracing::debug;

pub struct ButtonScene {
  pos: Option<Rect>,
  lil_name: String,
  pressed: bool, 
  color_tmp: Color, // temporary because there will be a better style system obviously
  layout: Layout
}

impl ButtonScene {
  pub fn new(lil_name: &str, color_tmp: Color, layout: Layout) -> Self { 
    Self{pos: None, lil_name: lil_name.to_string(), pressed: false, color_tmp, layout}
  }
}

impl Scene for ButtonScene {
  fn name(&self) -> &str { &self.lil_name }
  fn init(&mut self, bus: &mut ActionBus) {
    bus.push(Action::RequestLayout(self.layout.clone()));
    bus.push(Action::Scene { scene: Box::new(TextScene::new(self.lil_name.clone(), "big".to_owned())), layer: 4 /*URGENT TODO  */ });
  }

  fn render(&self, renderer: &mut Renderer) {
    if self.pos.is_none() { return; }
    if self.pressed {
      renderer.canvas.set_draw_color(Color::RGB(0, 100, 255));
    }
    else {
      renderer.canvas.set_draw_color(self.color_tmp);
    }
    renderer.canvas.fill_rect(self.pos).map_err(|e| e.to_string()).unwrap(); 
  }

  fn left_click_zone(&self) -> Option<Rect> {
    self.pos
  }
  
  fn handle_action(&mut self, action: &Action, _origin: Option<SceneID>, action_bus: &mut ActionBus) -> bool { // for now todo 
    if self.pos.is_none() { return false; }
    match action { 
      Action::SdlEvent(event) => {
        match event {
          Event::MouseButtonDown{ mouse_btn: MouseButton::Left, .. } => { 
            debug!(target: "hgui::button", "Clicked on {}", &self.lil_name);
            self.pressed = true;
            true
          }, 
          Event::MouseButtonUp { mouse_btn: MouseButton::Left, x, y, .. } => {
            if !self.pressed {
              return false; // Probably not for us 
            }
            debug!(target: "hgui::button", "Button up!");
            self.pressed = false;
            if is_point_in_rect(&self.pos.unwrap(), *x, *y) {
              action_bus.push(Action::ButtonPressed);
            } // Else the button is still left unpressed but no event! 
            true
          },
          _ => { false }
        }
      }, 
      _ => { false } // unexpected given the subscriptions! ... 
    }
  }

  fn pos_changed(&mut self, rect: Rect) {
    debug!(target: "hgui::button", "Pos changed to {:?}", &rect);
    self.pos = Some(rect);
  }

  fn subscriptions(&self) -> EventKind {
    EventKind::SdlMouseClick
  }
}
