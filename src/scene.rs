use std::{collections::HashSet, u64};
use sdl2::{event::Event, rect::Rect, video::Window};
use taffy::NodeId;
use crate::{action::{Action, ActionKind}, action_bus::{ActionBus, ActionPriv}, layout_manager::LayoutManager, sprite::SpriteStore, utils::is_point_in_rect};

// Unique identifier for each scene.
pub type SceneID = u64;

// TODO
// big optimizations to perform here 
// with from_id
// avoid a linear search all the time
// use a little map while the scenestack is not dirtified ... 

pub trait Scene {
  fn init(&mut self, _action_bus: &mut ActionBus) {}

  // todo call update and render "at the same time"
  fn update(&mut self, _delta_time: f32, _action_bus: &mut ActionBus) {}
  fn render(&self, _renderer: &mut sdl2::render::Canvas<Window>, _sprites: &mut SpriteStore) -> Result<(), String> {
    Ok(())
  }
  fn is_modal(&self) -> bool { false }
  fn handle_action(&mut self, _action: &Action, _origin: Option<SceneID>, _action_bus: &mut ActionBus) -> bool { false }
  fn left_click_zone(&self) -> Option<Rect> { None }
  fn pos_changed(&mut self, _pos: Rect) {  }
  fn susbcriptions(&self) -> ActionKind { ActionKind::NoSubscription }
  fn name(&self) -> &str { "Unknown" }
}

pub(crate) struct ScenePriv {
  id: SceneID,
  pub(crate) taffy_id: Option<NodeId>,
  pub(crate) parent: SceneID,
  children: Vec<SceneID>,
  pub(crate) scene: Box<dyn Scene>,
}

impl ScenePriv {
  pub fn new(id: SceneID, parent: SceneID, scene: Box<dyn Scene>) -> Self {
    Self { id, taffy_id: None, parent, children: vec![], scene }  
  }
}

// TODO this will change (why having a max layer depth ?)
const MAX_LAY: usize = 10;

struct RootScene {}
impl RootScene{fn new() -> Self { Self{} }}
impl Scene for RootScene {}

pub struct SceneStack {
  // Each entry is a list (stack) of scenes for that layer.
  // For example, is background scenes, layers[1] game world scenes... etc
  scenes_priv: Vec<Vec<ScenePriv>>, 
 // pub(crate)next_id: SceneID
}

impl SceneStack 
{
  pub fn new(user_root_scene: Box<dyn Scene>, engine_root_node_id: NodeId) -> Self {
    let mut scenes_priv: Vec<Vec<ScenePriv>> = (0..MAX_LAY).map(|_| Vec::new()).collect();
    let mut engine_root = ScenePriv::new(0, u64::MAX, Box::new(RootScene::new()));
    engine_root.taffy_id = Some(engine_root_node_id);

    scenes_priv[0].push(engine_root); // the engine root scene
    scenes_priv[0].push(ScenePriv::new(1, 0, user_root_scene)); // the user root scene
    Self { scenes_priv }
  }

  pub(crate) fn push(&mut self, layer: usize, scene: Box<dyn Scene>, id: SceneID, parent: SceneID) -> SceneID
  {
    println!("New scene requested : id=<{}>, name=<{}>", id, scene.name());
    self.scenes_priv[layer].push(ScenePriv::new(id, parent, scene));

    // Add children 
     // If detached mode, pass 0 anyway, so, .. ok 
    println!("Pushing scene id=<{}>, parent=<{}>", id, parent);
    if let Some(parent_scene) = self.from_id(parent) {
      parent_scene.children.push(id); // If parent scene is dead we are bad 
    }
    id
  }

  pub(crate) fn remove_scene(&mut self, scene_id: SceneID) {
    let mut ids_to_remove = HashSet::new();
    self.collect_descendants(scene_id, &mut ids_to_remove);

    println!("Found descendants : {:#?}", ids_to_remove);
    let mut found = false;
    for layer_vec in &mut self.scenes_priv {
      layer_vec.retain(|sc_p| {
        if ids_to_remove.contains(&sc_p.id) {
          found = true;
          false // Remove the scene
        } else {
          true // Keep the scene
        }
      });
    }

    // Not even the parent scene was found
    if !found {
      return; // Maybe on some debug mode, raise something ... 
      // hey your scene doesnt exist man
    }
  }

  // Recursive function to collect all descendants using the `children` vector
  fn collect_descendants(&self, scene_id: SceneID, descendants: &mut HashSet<SceneID>) 
  {
    descendants.insert(scene_id);

    for layer_vec in &self.scenes_priv {
      for sc_p in layer_vec {
        if sc_p.id == scene_id {
          // Recurse for all children
          for child_id in &sc_p.children {
            self.collect_descendants(*child_id, descendants);
          }
          return; // Once the scene is found, no need to keep searching
        }
      }
    }
  }

  pub fn init(&mut self, id: SceneID, action_bus: &mut ActionBus) {
    self.from_id(id).unwrap().scene.init(action_bus);
  }

  pub fn update_all(&mut self, delta_time: f32, action_bus: &mut ActionBus) {
    // Update bottom to top, just like rendering
    for layer in &mut self.scenes_priv {
      for sc_p in layer.iter_mut() {
        action_bus.prepare(sc_p.id);
        sc_p.scene.update(delta_time, action_bus);
      }
    }
  }

  // Paint the scenes from the lowest to the highest in the stack
  pub fn render_all(&self, renderer: &mut sdl2::render::Canvas<Window>, sprites: &mut SpriteStore) {
    for layer in &self.scenes_priv {
      for scene_priv in layer.iter() {
        scene_priv.scene.render(renderer, sprites).unwrap();
      }
    }
  }

  pub fn propagate_sdl2_to_subscribers(&mut self, action_bus: &mut ActionBus, action: Action)
  {
    // if nobody subscribed to that event, just return (TODO)
    let ak = action.kind();

    // Starting from the top layer to the bottom (reverse order)
    for layer_index in (0..self.scenes_priv.len()).rev() 
    {
      let layer = &mut self.scenes_priv[layer_index];
      for sc_idx in (0..layer.len()).rev()
      {
        let scene_priv = &mut layer[sc_idx];

        // No need to do anything if the scene didn't subscribe to that action.
        if !scene_priv.scene.susbcriptions().intersects(ak.clone()) {
          continue;
        }
        // Filter unwanted clicks if out of the clickable zone.
        // It also filters if there is NO clickable zone, but it is rarer 
        // Since a scene with no clickable zone shouldn't subscribe to click events!...
        if let Action::SdlEvent (Event::MouseButtonDown{x, y, ..}) = action {
          let clickable = scene_priv.scene.left_click_zone()
          .map_or(false, |rect| is_point_in_rect(&rect, x, y));

          if !clickable && scene_priv.scene.is_modal() { // x, y not in clickable zone
            return; // If x, y not in current scene and scene is modal, just return.
          }
          else if !clickable { // Just fall through the next scenes.
            continue;
          }
        }

        // Call the user handler
        action_bus.prepare(scene_priv.id);
        if scene_priv.scene.handle_action(&action, None, action_bus) {
          return;
        } // If the event was consumed or the scene is modal, I stop traversing
        
        if scene_priv.scene.is_modal() {
         // Stop looping on first modal scene even if it did not handle the event
          return;
        }
      }
    }
  }

  pub(crate) fn propagate_ham_to_subscribers(&mut self, action_bus: &mut ActionBus, action_p: ActionPriv) 
  {
    // if nobody subscribed to that event, just return (TODO)
    // Starting from the top layer to the bottom (reverse order)
    for layer_index in (0..self.scenes_priv.len()).rev() 
    {
      let layer = &mut self.scenes_priv[layer_index];
      for sc_idx in (0..layer.len()).rev()
      {
        let scene_priv = &mut layer[sc_idx];

        // CALL THE USER HANDLER
        action_bus.prepare(scene_priv.id);
        if scene_priv.scene.handle_action(&action_p.action, Some(action_p.source_scene), action_bus) {
          return;
        } // If the event was consumed or the scene is modal, I stop traversing
        
        if scene_priv.scene.is_modal() {
          // Stop looping on first modal scene even if it did not handle the event
          return;
        }
      }
    }
  }

  pub(crate) fn from_id(&mut self, id: SceneID) -> Option<&mut ScenePriv> {
    for layer_vec in &mut self.scenes_priv 
    {
      if let Some(pos) = layer_vec.iter().position(|sc_p| sc_p.id == id) {
        return Some(&mut layer_vec[pos]);
      }
    }
    None
  }
   // todo optimise with indexes 
  pub(crate) fn nodeid(&mut self, id: SceneID) -> Option<NodeId> {
    if let Some(scene_p) = self.from_id(id) {
      scene_p.taffy_id
    }
    else {
      println!("Warning! No scene found for id {}", id);
      None
    }
  }

  pub(crate) fn set_nodeid(&mut self, id: SceneID, node_id: NodeId) {
    self.from_id(id).unwrap().taffy_id = Some(node_id);
  }

  pub(crate) fn parent(&mut self, id: SceneID) -> SceneID {
    self.from_id(id).unwrap().parent
  }

  pub(crate) fn update_layout(&mut self, layout_mgr: &LayoutManager) {
    // if nobody subscribed to that event, just return (TODO)
    // cf subscription system.
    // Starting from the top layer to the bottom (reverse order)
    for layer_index in (0..self.scenes_priv.len()).rev() 
    {
      let layer = &mut self.scenes_priv[layer_index];
      for sc_idx in (0..layer.len()).rev()
      {
        let scene_priv = &mut layer[sc_idx];
        if let Some(taffy_id) = scene_priv.taffy_id {
          scene_priv.scene.pos_changed(layout_mgr.abs_layout(taffy_id));
        } 
      }
    }
  }
}
