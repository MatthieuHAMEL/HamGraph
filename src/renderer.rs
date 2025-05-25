use egui::{epaint, Context};
use egui_sdl2_canvas::Painter;
use egui_sdl2_platform::Platform;
use sdl2::{pixels::Color, render::{Canvas, TextureCreator}, video::{Window, WindowContext}, Sdl, VideoSubsystem};

use crate::{font::FontStore, sprite::SpriteStore};

// experiment
pub fn temp_change_font_size(ctx: &egui::Context) {
  let mut style = (*ctx.style()).clone();
  for (_text_style, font) in style.text_styles.iter_mut() {
    font.size *= 2.0;
  }
  ctx.set_style(style);
}

pub struct Renderer<'a> {
  pub sdl_context: &'a Sdl,
  pub sdl_video: &'a mut VideoSubsystem,
  pub canvas: &'a mut Canvas<Window>,
  pub sprite_store: SpriteStore<'a>,
  pub font_store: FontStore<'a>,
  pub egui_platform: Platform,
  pub egui_painter: Painter<'a>,
  pub egui_ctx: Option<Context>,
  pub texture_creator: &'a TextureCreator<WindowContext>,
  pub first_pass: bool
}

impl<'a> Renderer<'a> {
  pub fn new(sdl_context: &'a Sdl, sdl_video: &'a mut VideoSubsystem, 
    canvas: &'a mut Canvas<Window>, sprite_store: SpriteStore<'a>, 
    font_store: FontStore<'a>, 
    texture_creator: &'a TextureCreator<WindowContext>) -> Self {
    let dim: (u32, u32) = canvas.window().size();
    Self {
      sdl_context, sdl_video, canvas, 
      sprite_store, font_store, 
      egui_platform: Platform::new(dim).unwrap(),
      egui_painter: Painter::new(),
      egui_ctx: None,
      texture_creator, 
      first_pass: true
    }
  }

  pub fn begin_egui_pass(&mut self) {
   self.egui_ctx = Some(self.egui_platform.context()); 
   if self.first_pass {// temp (TODO)
    temp_change_font_size(&self.egui_ctx.as_ref().unwrap());
    self.first_pass = false;
   } 
  }

  pub fn render(&self) { // ? TODO (abstraction over fill_rect, set_render_draw_color ...)
  }

  pub fn egui_ctx(&self) -> &Context {
    return &self.egui_ctx.as_ref().unwrap();
  }

  pub fn render_sprite(&mut self, sprite_id: usize, pos_x: i32, pos_y: i32, alpha: Option<u8>) {
    self.sprite_store.render(self.canvas, sprite_id, pos_x, pos_y, alpha);
  }

  pub fn end_egui_pass_and_paint(&mut self) {
    let output = self.egui_platform.end_frame(&mut self.sdl_video).unwrap();
    let v_primitives = self.egui_platform.tessellate(output.shapes);

    // Convert textures_delta (image data) to SDL2 textures, and draw
    if let Err(err) = self.egui_painter.paint_and_update_textures(
      self.egui_ctx.as_ref().unwrap().pixels_per_point(),
      &output.textures_delta, self.texture_creator, 
      &v_primitives, &mut self.canvas) {
        println!("{}", err);
    }
  }
}