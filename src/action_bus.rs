use std::{cell::Cell, rc::Rc};
use tracing::debug;
use crate::{action::Action, hg::HamID, scene::SceneID};

pub(crate) struct ActionPriv {
  pub(crate) source_scene: SceneID,
  // To be used by the engine : 
  // "the push function gave back that ID when the user did send a CreateXXX action"
  // "So if the engine creates a scene, it must have that ID ! ... "
  pub(crate) back_id: Option<HamID>, 
  pub(crate) action: Action,
}

impl ActionPriv {
  fn new(source_scene: SceneID, back_id: Option<HamID>, action: Action) -> Self {
    Self { source_scene, back_id, action }
  }
}

pub struct ActionBus {
  cur_processed_scene: SceneID, // To set the right source scene ID when actions are pushed
  pub(crate) next_sprite_id: Rc<Cell<usize>>, // Also used to sync with the sprite store as back_id
  sprite_id_offset: usize, // in case there are more sprites 

  actions_priv: Vec<ActionPriv>,

  // Actions that cannot wait one frame to be handled by HAMGRAPH 
  // This is the case of "creation actions". We returned a HamID to the user, like a 
  // sprite ID. So the user expects to be able to use it in its render() method ... 
  prioritary: Vec<ActionPriv>,
  // The prepare() function transfers the "next_scene_id" given by SceneStack to the Bus.
  // The bus only increments it if it receives several CreateScene actions.
  pub(crate) future_scene_id: SceneID,
}

impl ActionBus {
  // Constructor: Expose only within the crate
  pub(crate) fn new(shared_spritestore_len: Rc<Cell<usize>>) -> Self {
    ActionBus {
      cur_processed_scene: 0,
      next_sprite_id: shared_spritestore_len,
      sprite_id_offset: 0, 

      actions_priv: Vec::new(),
      prioritary: Vec::new(),
      future_scene_id: 0 // as root scene is 0, user root scene 1, and other scenes must be created using Actions
    }
  }

  // Sync with the other data structures ... 
  pub fn prepare(&mut self, cur_processed_scene: SceneID, next_scene_id: SceneID) {
    self.cur_processed_scene = cur_processed_scene;
    self.future_scene_id = next_scene_id;
  }

  // Allow pushing actions, make it public
  // For scene creation actions it will return an ID 
  // Maybe it can be extended to other ID/Handle-Based mechanisms.
  pub fn push(&mut self, action: Action) -> Option<HamID> {
    debug!(target: "hg::bus", "Action pushed in bus.");
    let mut ret: Option<HamID> = None;

    match action {
      Action::Scene {..} | Action::ImmediateUI {..} => {
        debug!(target="hg::bus", "User asked for new scene - provided id=<{}>", self.future_scene_id);
        ret = Some(HamID::SceneID(self.future_scene_id));
        self.future_scene_id += 1;
      },
      Action::CreateText { .. } => {
        ret = Some(HamID::SpriteID(self.next_sprite_id.get() + self.sprite_id_offset));
        self.prioritary.push(ActionPriv::new(self.cur_processed_scene, ret.clone(), action));
        self.sprite_id_offset += 1;
        return ret; // Special case! 
      },
      _ => {}
    }

    self.actions_priv.push(ActionPriv::new(self.cur_processed_scene, ret.clone(), action));
    ret  
  }

  // Method to consume the events (used internally by the engine)
  pub(crate) fn take_all(&mut self) -> Vec<ActionPriv> {
    std::mem::take(&mut self.actions_priv)
  }

  pub(crate) fn take_prioritary(&mut self) -> Vec<ActionPriv> {
    self.sprite_id_offset = 0;
    std::mem::take(&mut self.prioritary)
  }
}
