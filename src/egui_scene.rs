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
  ctx:      egui::Context,
  widget:   W,
}

impl<W: EguiWidget> Scene for EguiScene<W> {
  fn init(&mut self, bus: &mut ActionBus) {
    bus.push(Action::RequestLayout(self.baseline_layout.clone()));
  }

  fn pos_changed(&mut self, r: Rect) { 
    self.rect = r; 
  }

  fn immediate(&mut self, _renderer: &mut Renderer, bus: &mut ActionBus) {
    egui::Area::new(egui::Id::new("egui_leaf"))
     // .fixed_pos(self.rect.min())
    //  .fixed_size(self.rect.size())
      .show(&self.ctx, |ui| {
          self.widget.ui(&self.ctx, ui, bus); // USER CODE
      });
  }

}

