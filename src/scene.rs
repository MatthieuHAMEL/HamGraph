use std::collections::HashSet;
use sdl2::{event::Event, rect::Rect};
use taffy::NodeId;
use tracing::{debug, warn};
use crate::{action::{Action, EventKind}, action_bus::{ActionBus, ActionPriv}, Renderer, layout_manager::LayoutManager, utils::is_point_in_rect};

// Unique identifier for each scene.
pub type SceneID = u64;

// TODO
// big optimizations to perform here 
// with from_id
// avoid a linear search all the time
// use a little map while the scenestack is not dirtified ... 

const TRASCENE: &str = "hg::scene";

pub trait Scene {
  fn init(&mut self, _action_bus: &mut ActionBus) {}

  // todo call update and render "at the same time"
  fn update(&mut self, _delta_time: f32, _action_bus: &mut ActionBus) {}

  // TODO : this is private ! CF egui_scene.
  fn immediate(&mut self, _renderer: &mut Renderer, _action_bus: &mut ActionBus) -> Option<Rect> { None }
  fn render(&self, _renderer: &mut Renderer) {}
  fn is_modal(&self) -> bool { false }
  fn handle_action(&mut self, _action: &Action, _origin: Option<SceneID>, _action_bus: &mut ActionBus) -> bool { false }
  fn left_click_zone(&self) -> Option<Rect> { None }

  // TODO. a rectangle position is not enough. The scene may want to know its "border" size for example
  fn pos_changed(&mut self, _pos: Rect) {  }
  fn subscriptions(&self) -> EventKind { EventKind::NotAnEvent }
  fn name(&self) -> &str { "Unknown" }
  fn is_immediate(&self) -> bool { false } // private 
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

  pub fn get_id(&self) -> SceneID {
    self.id
  }

  pub fn get_parent_id(&self) -> SceneID {
    self.parent
  }

  pub fn get_nodeid(&self) -> Option<NodeId> {
    self.taffy_id
  }
}

// TODO this will change (why having a max layer depth ?)
const MAX_LAY: usize = 10;

struct RootScene {}
impl RootScene{fn new() -> Self { Self{} }}
impl Scene for RootScene {}

pub struct SceneStack {
  // Each entry is a list (stack) of scenes for that layer.
  // For example, layers[0] is background scenes, layers[1] game world scenes... etc
  scenes_priv: Vec<Vec<ScenePriv>>, 
  next_scene_id: SceneID,
}

impl SceneStack 
{
  pub fn new(user_root_scene: Box<dyn Scene>, engine_root_node_id: NodeId) -> Self {
    let mut scenes_priv: Vec<Vec<ScenePriv>> = (0..MAX_LAY).map(|_| Vec::new()).collect();
    let mut engine_root = ScenePriv::new(0, u64::MAX, Box::new(RootScene::new()));
    engine_root.taffy_id = Some(engine_root_node_id);

    scenes_priv[0].push(engine_root); // the engine root scene
    scenes_priv[0].push(ScenePriv::new(1, 0, user_root_scene)); // the user root scene
    Self { scenes_priv, next_scene_id: 2 }
  }

  pub(crate) fn next_scene_id(&self) -> SceneID {
    self.next_scene_id
  }

  pub(crate) fn push(&mut self, layer: usize, scene: Box<dyn Scene>, parent: SceneID) // TODO result or anything
  {
    let new_scene_id = self.next_scene_id;
    debug!("New scene requested : id=<{}>, name=<{}>, parent=<{}>", new_scene_id, scene.name(), parent);
    self.scenes_priv[layer].push(ScenePriv::new(new_scene_id, parent, scene));

    // Add children 
     // If detached mode, pass 0 anyway, so, .. ok 
    if let Some(parent_scene) = self.get_scene(parent) {
      parent_scene.children.push(new_scene_id); // If parent scene is dead we are bad 
    }
    self.next_scene_id += 1;
  }

  pub(crate) fn remove_scene(&mut self, id: SceneID) -> bool {
    if id == 0 {
      return false; // Cannot remove the root scene
    }
    let mut ids_to_remove = HashSet::new();
    self.collect_descendants(id, &mut ids_to_remove);

    debug!(target: "hg::scene", "Remove {} -> found descendants : {:#?}", id, ids_to_remove);
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

    if !found {
      // Nothing was found (the scene to remove, nor the descendants of course)
      debug!(target: "hg::scene", "Scene not found in remove_scene.");
    }

    found
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
    self.get_scene(id).unwrap().scene.init(action_bus);
  }

  pub fn update_all(&mut self, delta_time: f32, action_bus: &mut ActionBus) {
    // Update bottom to top, just like rendering
    for layer in &mut self.scenes_priv {
      for sc_p in layer.iter_mut() {
        action_bus.prepare(sc_p.id, self.next_scene_id);
        sc_p.scene.update(delta_time, action_bus);
      }
    }
  }

  // Paint the scenes from the lowest to the highest in the stack
  pub fn render_all(&mut self, renderer: &mut Renderer, action_bus: &mut ActionBus) {
    
    for layer in &mut self.scenes_priv {
      for scene_priv in layer.iter_mut() {
        scene_priv.scene.render(renderer);

        if scene_priv.scene.is_immediate() {
          let real_rect = Some(scene_priv.scene.immediate(renderer, action_bus));
          println!("Found real rect for UI {:?}", real_rect);
        }
      }
    }

  }

  // We are already doing a match {} on sdl events in the main loop. So we directly give the event kind here
  // Otherwise if we call action.event_kind we're back traversing every existing action.
  pub fn propagate_sdl2_to_subscribers(&mut self, action_bus: &mut ActionBus, action: Action, event_kind: EventKind)
  {
    // if nobody subscribed to that event, just return (TODO)
    // Starting from the top layer to the bottom (reverse order)
    for layer_index in (0..self.scenes_priv.len()).rev() 
    {
      let layer = &mut self.scenes_priv[layer_index];
      for sc_idx in (0..layer.len()).rev()
      {
        let scene_priv = &mut layer[sc_idx];

        // No need to do anything if the scene didn't subscribe to that action.
        if !scene_priv.scene.subscriptions().intersects(event_kind.clone()) {
          continue;
        }
        // Filter unwanted clicks if out of the clickable zone.
        // It also filters if there is NO clickable zone, but it is rarer 
        // Since a scene with no clickable zone shouldn't subscribe to click events!...
        if let Action::SdlEvent (Event::MouseButtonDown{x, y, ..}) = action {
          let clickable = scene_priv.scene.left_click_zone()
          .is_some_and(|rect| is_point_in_rect(&rect, x, y));

          if !clickable && scene_priv.scene.is_modal() { // x, y not in clickable zone
            return; // If x, y not in current scene and scene is modal, just return.
          }
          else if !clickable { // Just fall through the next scenes.
            continue;
          }
        }

        // Call the user handler
        action_bus.prepare(scene_priv.id, self.next_scene_id);
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
        let next_scene_id = self.next_scene_id;
        let scene_priv = &mut layer[sc_idx];

        // CALL THE USER HANDLER
        action_bus.prepare(scene_priv.id, next_scene_id);
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

  pub(crate) fn get_scene(&mut self, id: SceneID) -> Option<&mut ScenePriv> {
    for layer_vec in &mut self.scenes_priv {
      if let Some(pos) = layer_vec.iter().position(|sc_p| sc_p.id == id) {
        return Some(&mut layer_vec[pos]);
      }
    }
    None
  }

   // todo optimise with indexes 
  pub(crate) fn nodeid(&mut self, id: SceneID) -> Option<NodeId> {
    if let Some(scene_p) = self.get_scene(id) {
      scene_p.taffy_id
    }
    else {
      warn!(target: TRASCENE, "Warning! No scene found for id {}", id);
      None
    }
  }

  pub(crate) fn set_nodeid(&mut self, id: SceneID, node_id: NodeId) {
    self.get_scene(id).unwrap().taffy_id = Some(node_id);
  }

  pub(crate) fn get_first_with_layout(&mut self, id: SceneID) -> (SceneID, NodeId) {
    let mut current_id = id;
    while let Some(scene) = self.get_scene(current_id) {
      if scene.taffy_id.is_some() {
        return (current_id, scene.get_nodeid().unwrap());
      }
      current_id = scene.parent;
    }

    (0, NodeId::new(0)) // Default to root scene if no layout is found
  }

  pub(crate) fn parent(&mut self, id: SceneID) -> SceneID {
    self.get_scene(id).unwrap().parent
  }

  pub(crate) fn update_layout(&mut self, layout_mgr: &LayoutManager) {
    // if nobody subscribed to that event, just return (TODO optimization)
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

////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
  use super::*;

  struct TestScene {}
  impl TestScene{fn new() -> Self { Self{} }}
  impl Scene for TestScene {}

  #[test]
  fn test_push_and_get_scene() {
    let mut stack = SceneStack::new(Box::new(TestScene::new()), NodeId::new(1));

    // Test there are 2 scenes in the stack
    assert_eq!(stack.next_scene_id, 2);
    assert!(stack.get_scene(0).is_some());
    assert!(stack.get_scene(1).is_some());
    assert!(stack.get_scene(2).is_none());
    assert!(stack.get_scene(327).is_none());

    // Push a new scene on layer 0
    stack.push(0, Box::new(TestScene::new()), 1);

    assert_eq!(stack.next_scene_id, 3);

    // Now it should be there.
    assert!(stack.get_scene(2).is_some());
  }

  #[test]
  fn test_remove_scene_same_layer() {
    let mut stack = SceneStack::new(Box::new(TestScene::new()), NodeId::new(1));
    stack.push(0, Box::new(TestScene::new()), 1); // Id should be 2
    assert_eq!(stack.next_scene_id, 3);
    assert!(stack.get_scene(2).is_some());
    let mut removed = stack.remove_scene(2);
    assert!(removed);
    assert!(stack.get_scene(2).is_none());

    assert!(stack.get_scene(1).is_some());
    removed = stack.remove_scene(1);
    assert!(removed);
    assert!(stack.get_scene(1).is_none());

    assert!(stack.get_scene(0).is_some());
    removed = stack.remove_scene(0);
    // We shouldn't be able to remove engine root scene
    assert!(!removed);
    assert!(stack.get_scene(0).is_some());
  }

  #[test]
  fn test_parent() {
    let mut stack = SceneStack::new(Box::new(TestScene::new()), NodeId::new(1));
    assert_eq!(stack.parent(1), 0);

    stack.push(0, Box::new(TestScene::new()), 1); // Id should be 2
    assert_eq!(stack.parent(2), 1);

    stack.push(0, Box::new(TestScene::new()), 2); // 3
    assert_eq!(stack.parent(3), 2);

    // Let's try on a different layer, it shouldn't change anything
    stack.push(1, Box::new(TestScene::new()), 1); // 4
    assert_eq!(stack.parent(4), 1);
  }

  #[test]
  fn test_prune() {
    let mut stack = SceneStack::new(Box::new(TestScene::new()), NodeId::new(1));
    stack.push(3, Box::new(TestScene::new()), 1); // id 2
    stack.push(4, Box::new(TestScene::new()), 2); // id 3
    stack.push(2, Box::new(TestScene::new()), 1); // id 4
    stack.push(6, Box::new(TestScene::new()), 1); // id 5
    stack.push(6, Box::new(TestScene::new()), 2); // id 6
    
    // Assert all the scenes are there ...
    for i in 0..=6 {
      assert!(stack.get_scene(i).is_some());
    }
    // ... Not more!
    assert!(stack.get_scene(7).is_none());

    // Now prune id 2
    let mut removed = stack.remove_scene(2);
    assert!(removed);

    // Check that childs of node 2, as long as 2, have disappeared
    assert!(stack.get_scene(2).is_none());
    assert!(stack.get_scene(3).is_none());
    assert!(stack.get_scene(6).is_none());

    // Check that the others are still there
    assert!(stack.get_scene(0).is_some());
    assert!(stack.get_scene(1).is_some());
    assert!(stack.get_scene(4).is_some());
    assert!(stack.get_scene(5).is_some());

    // Now add some more scenes under surviving scenes ...
    stack.push(6, Box::new(TestScene::new()), 4); // id 7
    stack.push(6, Box::new(TestScene::new()), 5); // id 8
    stack.push(6, Box::new(TestScene::new()), 5); // id 9
    assert!(stack.get_scene(7).is_some());
    assert!(stack.get_scene(8).is_some());
    assert!(stack.get_scene(9).is_some());

    // ... And prune from ID 1 : removal depth is higher this time.
    removed = stack.remove_scene(1);
    assert!(removed);

    // Everything should have disappeared except the root scene
    assert!(stack.get_scene(0).is_some());
    for i in 1..=10 {
      assert!(stack.get_scene(i).is_none());
    }
  }
}
