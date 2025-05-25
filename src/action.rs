use sdl2::event::Event;

use crate::egui_scene::EguiWidget;
use crate::layout_manager::Layout;
use crate::scene::Scene;
use crate::scene::SceneID;
use bitflags::bitflags;

// TODO: do better : avoid all this boilerplate !!!
// I'm not satisfied with this module 
// Have some int represent DIRECTLY the real enum variants ?
// A lightweight enum that only represents the *kind* of an action
// ... avoid that big match {} below ... 

bitflags! {
  /// Flags representing the different *kinds* of actions.
  /// Each bit stands for one type of action.
  /// todo maybe this could look less HIDEOUS
  #[derive(Clone)]
  pub struct EventKind: u64 {
    const NotAnEvent         = 0;
    const SdlMouseClick = 1 << 0;
    const SdlMouseHover = 1 << 1;
    const SdlKeyboard   = 1 << 2;
    const SdlMisc       = 1 << 3;
    const ButtonPressed = 1 << 4;
    const SceneMsg      = 1 << 5;
    const Misc          = 1 << 6;
  }
}

pub enum Action {
  // -- Raw SDL2 events (engine to user space)
  SdlEvent(sdl2::event::Event),

  // -- Engine-level commands
  // The engine itself interprets these, e.g., to push or pop scenes, start music, etc.
  // For an SDL event : propagate the events to the scenes below.

  Scene {
    scene: Box<dyn Scene>,
    layer: usize
  },
  ImmediateUI {
    widget : Box<dyn EguiWidget>,
    layout: Layout, // min / preferred / max, grow/shrink
    layer: usize
  },

  CreateText { // Create a TTf texture from a text 
    font: String, 
    size: String, // e.g. "small" "medium" "big" or directly a number like "35"
    text: String // + TODO max_width Option<u32> 
  },
  CloseCurrentScene, 
  CloseScene {
    target_id: SceneID,
  },
  // Simplistic music commands for now (TODO?)
  StartMusic {
    track: String, // e.g. "intro.mp3"
    loops: i32
  },
  StopMusic {
  },
  StartSfx {
    track: String, // e.g. "intro.mp3"
    channel: i32
  },

  RequestLayout(Layout),

  ButtonPressed,

  // -- Scene-to-scene messages
  // The engine will find the target scene and call `handle_event(Action::SceneMsg { ... })` on it.
  SceneMsg {
    target_id: SceneID,
    msg: SceneMessage, 
  },
}

impl Action {
  pub fn event_kind(&self) -> EventKind {
    match self {
      Action::SdlEvent(evt) => match evt {
        Event::MouseButtonDown {..} | Event::MouseButtonUp {..} => EventKind::SdlMouseClick,
        Event::KeyDown {..} | Event::KeyUp {..} => EventKind::SdlKeyboard,
        _ => EventKind::SdlMisc
      },
      Action::ButtonPressed { .. } => EventKind::ButtonPressed,
      Action::SceneMsg { .. } => EventKind::SceneMsg,
      _ => EventKind::NotAnEvent
    }
  }
}

// TODO 
// This is a pure sample 
// Optional sub-enum for more structured "scene-to-scene" messages
#[derive(Debug)]
pub enum SceneMessage {
    IncrementCounter(i32),
    HideCharacter(u32),
    // ... user-defined or engine-defined 
    // The user can also define their own variant if you allow extension
}
