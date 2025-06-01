use sdl2::rect::Rect;

use crate::{action::Action, action_bus::ActionBus, layout_manager::Layout, scene::Scene, Renderer};

pub trait EguiWidget {
  // build the immediate UI; push actions as usual
  fn ui(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, bus: &mut ActionBus);
}

pub struct EguiScene {
  // immutable wishes from user
  base_layout: Layout,
  // runtime floor (only min_size.* changes)
  delta:    taffy::Style,
  rect:     Option<Rect>,
  egui_rect: Option<egui::Rect>,
  ctx:      egui::Context,
  widget:   Box<dyn EguiWidget>,
}


impl EguiScene {
  // Engine‐internal constructor used when it handles Action::ImmediateUI
  pub fn new(base_layout: Layout, widget: Box<dyn EguiWidget>) -> Self {
    Self {
      base_layout,
      delta: taffy::Style::DEFAULT,
      rect: None, // will be set in pos_changed
      egui_rect: None,
      ctx: egui::Context::default(),
      widget,
    }
  }
}

impl Scene for EguiScene {
  fn init(&mut self, bus: &mut ActionBus) {
    bus.push(Action::RequestLayout(self.base_layout.clone()));
  }

  fn is_immediate(&self) -> bool { true }

  fn pos_changed(&mut self, r: Rect) { 
    self.rect = Some(r); 
    self.egui_rect = Some(egui::Rect::from_min_size(
      egui::pos2(self.rect.unwrap().x as f32, self.rect.unwrap().y as f32),
      egui::vec2(self.rect.unwrap().width() as f32, self.rect.unwrap().height() as f32),
    ));
  }

  // TODO "negociating mode / phase ? don't render anything in this case"
  fn immediate(&mut self, _renderer: &mut Renderer, bus: &mut ActionBus) -> Option<Rect> {
    if self.rect.is_none() {
      println!("Not displaying anything, we havent received layout");
      return None;
    }

    // TODO. Negociate before! if negociation has not ended, don't display !
    let resp = egui::Area::new(egui::Id::new("egui_leaf")) // TODO wtf ?? I dont care about this ID!
      .fixed_pos(self.egui_rect.unwrap().min)
      .default_size(self.egui_rect.unwrap().size()) // TODO I need egui 0.31 
      .show(_renderer.egui_ctx(), |ui| {
        self.widget.ui(_renderer.egui_ctx(), ui, bus); // USER CODE!! TODO how to let the user choose between Area Window etc?
      });

    // I return the used rectangle for the flexbox negociation
    let egui_rect = resp.response.rect;
    let width = (egui_rect.max.x - egui_rect.min.x).max(0.0) as u32;
    let height = (egui_rect.max.y - egui_rect.min.y).max(0.0) as u32;
    Some(Rect::new(egui_rect.min.x as i32, egui_rect.min.y as i32, width, height))
  }

}

