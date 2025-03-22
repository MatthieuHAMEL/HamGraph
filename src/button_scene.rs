// Temporary placed here. To be part of HamUI. 
use crate::{action::{Action, ActionKind}, action_bus::ActionBus, layout_manager::Layout, scene::{Scene, SceneID}, sprite::SpriteStore, text_scene::TextScene, utils::is_point_in_rect};
use sdl2::{event::Event, mouse::MouseButton, pixels::Color, rect::Rect, video::Window};

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
    println!("=== Init Button");
    bus.push(Action::RequestLayout(self.layout.clone()));
    bus.push(Action::CreateScene { scene: Box::new(TextScene::new(self.lil_name.clone(), "big".to_owned())), layer: 4 /*URGENT TODO  */ });
  }

  fn render(&self, renderer: &mut sdl2::render::Canvas<Window>, _sprites: &mut SpriteStore) -> Result<(), String> {
    if self.pos.is_none() { return Ok(()) }
    if self.pressed {
      renderer.set_draw_color(Color::RGB(0, 100, 255));
    }
    else {
      renderer.set_draw_color(self.color_tmp);
    }
    renderer.fill_rect(self.pos).map_err(|e| e.to_string())?; 
    Ok(())
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
            println!("Clicked on {}", &self.lil_name); 
            self.pressed = true;
            true
          }, 
          Event::MouseButtonUp { mouse_btn: MouseButton::Left, x, y, .. } => {
            if !self.pressed {
              return false; // Probably not for us 
            }
            println!("Button up !");
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
    println!("BUTTON pos changed {:?}", &rect);
    self.pos = Some(rect);
  }

  fn susbcriptions(&self) -> ActionKind {
    ActionKind::SdlMouseClickEvent
  }
}
