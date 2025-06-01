use hamgraph::{action::{Action, EventKind}, action_bus::ActionBus, button_scene::ButtonScene, egui_scene::EguiWidget, layout_manager::{Dimension, Layout, Size}, scene::{Scene, SceneID}, Renderer};
use sdl2::{event::Event, mouse::MouseButton, pixels::Color, rect::{Point, Rect}};

struct MyMenu {
  music_on: bool,
}

impl EguiWidget for MyMenu {
  fn ui(&mut self, _ctx:&egui::Context, ui:&mut egui::Ui, _bus: &mut ActionBus) {

    egui::Frame::new()
      .fill(egui::Color32::from_rgb(30, 30, 40))
      .show(ui, |ui| {
        ui.set_min_size(ui.available_size());
        ui.vertical_centered(|ui| {
          if ui.button("New game").clicked() {
            println!("hello from bouton");
          }
          ui.checkbox(&mut self.music_on, "Music");
        });
      });
  }
}

pub trait RectangleBeh {
  fn color(&self) -> Color;
  fn pos(&self) -> Rect;
  fn name(&self) -> &str;

  fn rec_render(&self, renderer: &mut Renderer) {
    renderer.canvas.set_draw_color(self.color());
    renderer.canvas.fill_rect(self.pos()).map_err(|e| e.to_string()).unwrap(); 
  }

  fn rec_handle_action(&mut self, action: &Action, _action_bus: &mut ActionBus) -> bool {
    match action {
      Action::SdlEvent(evt) => {
        match evt {
          Event::MouseButtonDown{ mouse_btn: MouseButton::Left, .. } => { 
            println!("Clicked on {}", &self.name()); 
            true
          }, 
          _ => { false }
        }
      }, 
      _ => { false }
    }
  }
  
}
pub struct RectangleScene {
  pos: Rect,
  color: Color, 
  lil_name: String,
  modal: bool
}

impl RectangleScene 
{
  pub fn new(color: Color, lil_name: &str, modal: bool) -> Self { 
    Self{pos: Rect::new(0, 0, 100, 100), color, lil_name: lil_name.to_string(), modal}
  }
}

impl RectangleBeh for RectangleScene {
  fn color(&self) -> Color { self.color }
  fn pos(&self) -> Rect { self.pos }
  fn name(&self) -> &str { &self.lil_name }
}

impl Scene for RectangleScene 
{
  // Why does this scene has ID 0 ? BUG IN HAMGRAPH
  fn init(&mut self, bus: &mut ActionBus) {
    bus.push(Action::RequestLayout(Layout { ..Default::default() }));

    for _ in 1..=5 {
      bus.push(Action::Scene { 
        scene: Box::new(ButtonScene::new(
          "bou", Color::RGB(55,88,99), 
          Layout { size: Size { width: Dimension::Length(100.), height: Dimension::Length(200.) },  grow: 0.0, ..Default::default() }
        )),
        layer: 5
      });
    }

    // Push a EGUI scene at the middle 
    bus.push(Action::ImmediateUI { 
      widget: Box::new(MyMenu{music_on: false}), 
      layout: Layout { size: Size { width: Dimension::Length(100.), height: Dimension::Length(200.) },  grow: 0.0, ..Default::default()},
      layer: 5 
    });

    for _ in 1..5 {
      bus.push(Action::Scene { 
        scene: Box::new(ButtonScene::new(
          "bou", Color::RGB(55,88,99), 
          Layout { size: Size { width: Dimension::Length(100.), height: Dimension::Length(200.) },  grow: 0.0, ..Default::default() }
        )),
        layer: 5
      });
    }
  }

  fn render(&self, renderer: &mut Renderer) {
    self.rec_render(renderer);
  }

  fn handle_action(&mut self, action: &Action, _origin: Option<SceneID>, action_bus: &mut ActionBus) -> bool {
    self.rec_handle_action(action, action_bus)
  }

  fn is_modal(&self) -> bool {
    self.modal
  }

  fn left_click_zone(&self) -> Option<Rect> {
    Some(self.pos)
  }

  fn subscriptions(&self) -> EventKind {
    EventKind::SdlMouseClick
  }

  fn pos_changed(&mut self, pos: Rect) {
    println!("Mainframe pos changed to {:?}", pos);
    self.pos = pos;
  }
}
