use sdl2::rect::Rect;

use crate::{action::Action, action_bus::ActionBus, layout_manager::Layout, scene::Scene, Renderer};

pub trait EguiWidget {
  // build the immediate UI; push actions as usual
  fn ui(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, bus: &mut ActionBus);
}
pub struct EguiScene<W: EguiWidget + 'static> {
  /// immutable wishes from user
  baseline_layout: Layout,
  /// runtime floor (only min_size.* changes)
  delta:    taffy::Style,
 // node:     taffy::Node,
  rect:     Rect,
  egui_rect: egui::Rect,
  ctx:      egui::Context,
  widget:   W,
}

impl<W: EguiWidget> Scene for EguiScene<W> {
  fn init(&mut self, bus: &mut ActionBus) {
    bus.push(Action::RequestLayout(self.baseline_layout.clone()));
  }

  fn pos_changed(&mut self, r: Rect) { 
    self.rect = r; 
    self.egui_rect = egui::Rect::from_min_size(
      egui::pos2(self.rect.x as f32, self.rect.y as f32),
      egui::vec2(self.rect.width() as f32, self.rect.height() as f32),
    );
  }

  fn immediate(&mut self, _renderer: &mut Renderer, bus: &mut ActionBus) -> Option<Rect> {
    let resp = egui::Area::new(egui::Id::new("egui_leaf"))
     // .fixed_pos(self.rect.min())
      .default_size(self.egui_rect.size()) // TODO I need egui 0.31 
      .show(&self.ctx, |ui| {
          self.widget.ui(&self.ctx, ui, bus); // USER CODE
      });

    // I return the used rectangle for the flexbox negociation
    let egui_rect = resp.response.rect;
    let width = (egui_rect.max.x - egui_rect.min.x).max(0.0) as u32;
    let height = (egui_rect.max.y - egui_rect.min.y).max(0.0) as u32;
    Some(Rect::new(
      egui_rect.min.x as i32,
      egui_rect.min.y as i32,
      width,
      height,
    ))
  }

}

