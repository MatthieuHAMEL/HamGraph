use std::time::{Duration, Instant};

use sdl2::{event::{Event, WindowEvent}, keyboard::Keycode, mouse::MouseButton, pixels::Color, render::{Canvas, TextureCreator}, ttf::Sdl2TtfContext, video::{Window, WindowContext}, Sdl};
//use taffy::print_tree;
use crate::{action::Action, action_bus::{ActionBus, ActionPriv}, font::FontStore, infraglobals, layout_manager::LayoutManager, mixer_manager::MixerManager, scene::{Scene, SceneID, SceneStack}, sprite::SpriteStore};

#[derive(Debug, Clone)]
pub enum HamID {
  SceneID(SceneID), 
  SpriteID(usize)
}

pub struct HamGraph<'a> {
  pub sdl_context: Sdl,
  pub canvas: Canvas<Window>,
  pub sprite_store: SpriteStore<'a>,
  pub font_store: FontStore<'a>,
  pub scene_stack: SceneStack,
  pub(crate) layout_manager: LayoutManager,
  pub window_dim: (u32, u32),
  pub action_bus: ActionBus, // TODO not pub !
  pub mixer_manager: MixerManager<'a>,
  pub ttf_context: &'a Sdl2TtfContext,
}
  
impl<'a> HamGraph<'a> {
  pub fn new(
    sdl_context: Sdl,
    texture_creator: &'a mut TextureCreator<WindowContext>, 
    canvas: Canvas<Window>,
    ttf_ctx: &'a Sdl2TtfContext,
      mut root_scene: Box<dyn Scene>) -> Self 
  {
    let sprite_store = SpriteStore::new(texture_creator);

    let mut action_bus = ActionBus::new(sprite_store.shared_len());
    root_scene.init(&mut action_bus);

    let wdim = canvas.window().size();
    println!("Window dimensions: {:?}", wdim);
    let layout_manager = LayoutManager::new(wdim);

    let mut font_store = FontStore::new();
    // To do, this should of course be multiplied by the UI scale. 
    font_store.load_from_folder(ttf_ctx,  &infraglobals::get_ttf_path(), 12u16, "small");
    font_store.load_from_folder(ttf_ctx,  &infraglobals::get_ttf_path(), 20u16, "medium");
    font_store.load_from_folder(ttf_ctx,  &infraglobals::get_ttf_path(), 30u16, "big");
    Self { 
      sdl_context,
      canvas,
      sprite_store, 
      font_store,
      scene_stack: SceneStack::new(root_scene, layout_manager.root_node_id), 
      layout_manager,
      window_dim: wdim,
      action_bus, 
      mixer_manager: MixerManager::new(), 
      ttf_context: &ttf_ctx, 
    }
  }

  // Push a scene onto the stack
  fn register_scene(&mut self, layer: usize, scene: Box<dyn Scene>, id: SceneID, parent: SceneID) {
    let mut real_parent: SceneID = 0;
    // If parent is already dead, just parent the scene to 0
    if let Some(_) = self.scene_stack.from_id(parent) {
      real_parent = parent;
    }
    println!("Registering id=<{}>, parent=<{}>", id, real_parent);
    let id2 = self.scene_stack.push(layer, scene, id, real_parent);
      // assert id == id2 todo 
    if id != id2 {
      panic!("IDs inconsistency");
    }
    self.action_bus.prepare(id);
   // self.action_bus.next_sprite_id = self.sprite_store.size();
    self.scene_stack.init(id, &mut self.action_bus);
  }

  fn handle_user_action(&mut self, action_p: ActionPriv) { // TODO err mgt 
    match action_p.action {
      // Some actions are intended to be handled by HAMGRAPH :
      Action::CreateScene { scene, layer, .. } => {
        if let Some(HamID::SceneID(scid)) = action_p.back_id {
          self.register_scene(layer, scene, scid, action_p.source_scene );
        }
        else { panic!("Contract error [82]"); } 
                                 // error mgt for unwrap ..
        // If not "detached mode" -- TODO :
      },    
      Action::CreateText { font, size, text } => {
        self.sprite_store.create_ttf_texture(&self.font_store, font + "_" + &size, text);
        // TODO MATOU 
      },                          
      Action::CloseCurrentScene => {
        // Remove from layout if applicable 
        if let Some(nodeid) = self.scene_stack.nodeid(action_p.source_scene) {
          self.layout_manager.remove_layout(nodeid);
        }
        // Remove scene from scene stack
        self.scene_stack.remove_scene(action_p.source_scene);
      },
      Action::StartMusic { track, loops } => {
        self.mixer_manager.play_music(&track, loops);
      },
      Action::StartSfx { track, channel } => {
        self.mixer_manager.play_sfx(&track, channel);
      },
      Action::StopMusic { } => {
        self.mixer_manager.stop_music();
      }
      // Meant to be sent back to another scene 
      Action::ButtonPressed => {
        self.scene_stack.propagate_ham_to_subscribers(&mut self.action_bus, action_p);
      },
      Action::RequestLayout (lay) => {
        self.layout_manager.set_layout(action_p.source_scene, &mut self.scene_stack, lay);
      },
      _ => { println!("warning : user action left unhandled!"); }
    }
  }


  pub fn run_main_loop(&mut self)  // TODO interface
  {
    let mut event_pump = self.sdl_context.event_pump().unwrap_or_else(|e| {
      crate::errors::prompt_err_and_panic("SDL initialization error, no event pump", &e, None);
    });
  
    let target_frame_duration = Duration::from_secs_f32(1.0 / 60.0); // Targeting 60 FPS
    let mut last_update = Instant::now(); 

    'hamloop: loop 
    {
      // Debug temp todo 
      //println!("At this frame: TaffyTree: {:#?}", print_tree(&self.layout_manager.taffy_tree, self.layout_manager.root_node_id));
      //println!("[DEBUG] In Main Loop");
      // 1. HANDLE EVENTS
      for event in event_pump.poll_iter() 
      {
        match event 
        {
          Event::Quit {..} |
          Event::KeyDown { keycode: Some(Keycode::Escape), .. } => { break 'hamloop }, // LEGACY TODO 
            
            // this won't be needed once the weird stuff will have been filtered.  TODO 
          Event::MouseButtonDown {mouse_btn: MouseButton::Left, ..} 
          | Event::MouseButtonUp {mouse_btn: MouseButton::Left, ..} => {
            let action = Action::SdlEvent(event);
            self.scene_stack.propagate_sdl2_to_subscribers(&mut self.action_bus, action);
          }, 
          Event::Window { win_event: WindowEvent::Resized(w, h), ..} => {
            // Window has been resized : update the UI tree 
            self.layout_manager.set_new_window_size((w as u32, h as u32)); // TODO important manage min 
          }
          _ => { /* Nothing for now */ }
        }
      }

      // 2. PROCESS ACTIONS that were ordered by the input handlers 
      let actions = self.action_bus.take_all();
      for a in actions {
        self.handle_user_action(a);
      }
      loop {
        let actions_prio = self.action_bus.take_prioritary();
        if actions_prio.is_empty()  { break; }
        for a in actions_prio {
          self.handle_user_action(a);
        }
      }

      // 3. UPDATE GAME LOGIC
      let now = Instant::now();
      let delta_time = now.duration_since(last_update).as_secs_f32();
      last_update = now;
      self.scene_stack.update_all(delta_time, &mut self.action_bus);

        // Handle only prioritary actions here 
      loop {
        let actions_prio = self.action_bus.take_prioritary();
        if actions_prio.is_empty()  { break; }
        for a in actions_prio {
          self.handle_user_action(a);
        }
      }
      // maybe we should handle all actions after the render() ...  TODO 
        // so that the update() is as close to render() as possible ... 
        // anyway it will be close in permanent regime ... 

      // Update layout 
      let layout_changed = self.layout_manager.update_layout();
      if layout_changed {
        self.scene_stack.update_layout(&self.layout_manager);
      }


      // 4. DRAW
      self.canvas.set_draw_color(Color::RGB(0, 0, 0));
      self.canvas.clear();

        // TODO!!!
      
      self.scene_stack.render_all(&mut self.canvas, &mut self.sprite_store);

      // 5. UPDATE SCREEN
      self.canvas.present();

      // Maintain a consistent frame rate
      let frame_duration = now.elapsed();
      if frame_duration < target_frame_duration { // TODO not needed with VSYNC ?
        std::thread::sleep(target_frame_duration - frame_duration);
      } // else application is quite overwhelmed! ... 
      else { // temp 
        println!("HAMGRAPH is overwhelmed!");
      }
    }
  }
}
  