pub use crate::infraglobals::set_game_path;

use std::panic;

use sdl2::image;
use sdl2::image::Sdl2ImageContext;
use sdl2::mixer::{Sdl2MixerContext, AUDIO_S16LSB, DEFAULT_CHANNELS};
use sdl2::render::Canvas;
use sdl2::ttf::Sdl2TtfContext;
use sdl2::video::Window;
use sdl2::{mixer, IntegerOrSdlError};
use sdl2::Sdl;
use sdl2::VideoSubsystem;

use winapi::shared::windef::DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2;
use winapi::um::winuser::SetProcessDpiAwarenessContext;

use crate::errors::{self, prompt_err};

// Initializations are grouped, mainly for readability: 
// I may group them differently in the future. -- TODO
// ... maybe in a single struct with the different contexts ...
// Now also initalizing the eventpump and the canvas here...
pub fn init_sdl2(
    win_title: &str,
    win_width: u32,
    win_height: u32,
) -> (Sdl,
     Sdl2ImageContext,
     Sdl2TtfContext,
     VideoSubsystem,
     Sdl2MixerContext,
     Canvas<Window>) 
{
  panic::set_hook(Box::new(|panic_info| 
    {
      // Extract the panic message (if present)
      let msg = match panic_info.payload().downcast_ref::<&str>() {
        Some(s) => *s,
        None => match panic_info.payload().downcast_ref::<String>() {
          Some(s) => &s[..],
          None => "UNKNOWN ERROR",
        },
      };
    
      prompt_err(msg, None);
      // the panic still goes on after this function returns.
    }));

  //let mut b = sdl2::hint::set_with_priority("SDL_HINT_VIDEO_HIGHDPI_DISABLED", "1", &sdl2::hint::Hint::Override);

  // For some reason the hint below was not enough and I had to do that
  unsafe {  // TODO this is only on windows ...
    SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
  }

  let mut _b = sdl2::hint::set_with_priority(
    "SDL_HINT_WINDOWS_DPI_AWARENESS ",
    "permonitorv2",
    &sdl2::hint::Hint::Override,
  );

  // I'm handling DPI scaling by myself ... For now !
  _b = sdl2::hint::set_with_priority(
    "SDL_HINT_WINDOWS_DPI_SCALING",
    "0",
    &sdl2::hint::Hint::Override,
  );

  let sdl_context = sdl2::init().unwrap_or_else(|e| {
    errors::prompt_err_and_panic("SDL initialization error", &e, None);
  });

  // sdl2::hint::set("SDL_RENDER_SCALE_QUALITY", "1"); // for pixel linear interpolation. TODO needed ?
  let image_context = sdl2::image::init(image::InitFlag::PNG).unwrap_or_else(|e| {
    errors::prompt_err_and_panic("SDL initialization error", &e, None);
  });

  let video_subsystem = sdl_context.video().unwrap_or_else(|e| {
    errors::prompt_err_and_panic("SDL video init error", &e, None);
  });

  let ttf_context = sdl2::ttf::init().unwrap_or_else(|e| {
    errors::prompt_err_and_panic(&format!("SDL ttf init error {e}"), "", None);
  });

  // TODO flag "init music ... "
  let mixer_subsystem = mixer::init(mixer::InitFlag::MP3 | mixer::InitFlag::OGG).unwrap_or_else(
    |e| {
      errors::prompt_err_and_panic("SDL mixer init error", &e, None);
  });

  sdl2::mixer::open_audio(44100, AUDIO_S16LSB, DEFAULT_CHANNELS, 1024).unwrap_or_else(
    |e| {
    errors::prompt_err_and_panic("SDL open_audio error", &e, None);
  });

  sdl2::mixer::allocate_channels(16);

  // Window creation
  let mut windowb = video_subsystem.window(win_title, win_width, win_height);
  println!("windowb flags !!!!! 3- {}", windowb.window_flags()); // TODO simplify
  windowb.allow_highdpi().position_centered().resizable().maximized();

  let window = windowb.build().unwrap_or_else(|e| {
    errors::prompt_err_and_panic(&format!("SDL initialization error {e}"), "", None);
  });

  // The main object to render textures on (<=> SDL_CreateRenderer)
  let canvas: Canvas<Window> = window
    .into_canvas()
    // .present_vsync()
    .build() // vsync : (TODO : VSYNC support vs no vsync support)
    .map_err(|e| match e {
      IntegerOrSdlError::IntegerOverflows(msg, val) => {
        format!("int overflow {}, val: {}", msg, val)
      }
      IntegerOrSdlError::SdlError(msg) => {
        format!("SDL error: {}", msg)
      }
    })
    .unwrap_or_else(|e| {
      errors::prompt_err_and_panic("SDL initialization error", &e, None);
    });

  (sdl_context, image_context, ttf_context, video_subsystem, mixer_subsystem, canvas)
  // no need to return the window, it is held by the canvas
}
