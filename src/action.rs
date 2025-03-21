use sdl2::event::Event;

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
  pub struct ActionKind: u64 {
    const NoSubscription     = 0;
    const SdlMouseClickEvent = 0b0000_0001;
    const SdlMouseHoverEvent = 0b0000_0010;
    const SdlKeyboardEvent   = 0b0000_0100;
    const SdlMiscEvent       = 0b0000_1000;
    const CreateScene        = 0b0001_0000;
    const CreateText         = 0b0010_0000;
    const CloseCurrentScene  = 0b0100_0000;
    const CloseScene         = 0b1000_0000;
    const StartMusic         = 0b0001_0000_0000;
    const StopMusic          = 0b0010_0000_0000;
    const StartSfx           = 0b0100_0000_0000;
    const ButtonPressed      = 0b1000_0000_0000;
    const SceneMsg           = 0b0001_0000_0000_0000;
    const Misc     = 0b0010_0000_0000_0000;
  }
}

pub enum Action {
  // -- Raw SDL2 events (engine to user space)
  SdlEvent(sdl2::event::Event),

  // -- Engine-level commands
  // The engine itself interprets these, e.g., to push or pop scenes, start music, etc.
  // For an SDL event : propagate the events to the scenes below.

  CreateScene {
    scene: Box<dyn Scene>,
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
  pub fn kind(&self) -> ActionKind {
    match self {
      Action::SdlEvent(evt) => match evt {
        Event::MouseButtonDown {..} | Event::MouseButtonUp {..} => ActionKind::SdlMouseClickEvent,
        Event::KeyDown {..} | Event::KeyUp {..} => ActionKind::SdlKeyboardEvent,
        _ => ActionKind::SdlMiscEvent
      },
      Action::CreateScene { .. } => ActionKind::CreateScene,
      Action::CreateText { .. } => ActionKind::CreateText,
      Action::CloseCurrentScene => ActionKind::CloseCurrentScene,
      Action::CloseScene { .. } => ActionKind::CloseScene,
      Action::ButtonPressed { .. } => ActionKind::ButtonPressed,
      Action::RequestLayout { .. } => ActionKind::Misc, // cannot go down
      Action::StartMusic { .. } => ActionKind::StartMusic,
      Action::StopMusic { .. } => ActionKind::StopMusic,
      Action::StartSfx { .. } => ActionKind::StartSfx,
      Action::SceneMsg { .. } => ActionKind::SceneMsg,
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
