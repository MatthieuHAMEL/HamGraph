use std::time::{Duration, Instant};

use sdl2::{event::{Event, WindowEvent}, image::Sdl2ImageContext, keyboard::Keycode, mixer::Sdl2MixerContext, mouse::MouseButton, pixels::Color, render::{Canvas, TextureCreator}, ttf::Sdl2TtfContext, video::{Window, WindowContext}, Sdl, VideoSubsystem};
use tracing::{debug, info, warn};
//use taffy::print_tree;
use crate::{action::{Action, EventKind}, action_bus::{ActionBus, ActionPriv}, font::FontStore, infraglobals, init, layout_manager::LayoutManager, mixer_manager::MixerManager, scene::{Scene, SceneID, SceneStack}, sprite::SpriteStore, Renderer};

pub use crate::infraglobals::set_install_path;
pub use crate::infraglobals::set_userdata_path;

#[derive(Debug, Clone)]
pub enum HamID {
  SceneID(SceneID), 
  SpriteID(usize)
}

const TRAINIT: &str = "hg::init";
const TRASCENE: &str = "hg::scene";


/** "Abstraction" of SDL2 for the user. */
pub struct HamSdl2 {
  sdl_context: Sdl,
  _image_context: Sdl2ImageContext,
  ttf_context: Sdl2TtfContext,
  _video_subsystem: VideoSubsystem,
  _mixer_context: Sdl2MixerContext,
  canvas: Canvas<Window>,
  texture_creator: TextureCreator<WindowContext>,
  window_dim: (u32, u32),
}

impl HamSdl2 {
  pub fn new(title: &str, win_width: u32, win_heigt: u32) -> Self {
    let (sdl_context, _image_context, ttf_context, _video_subsystem, _mixer_context, canvas) 
    = init::init_sdl2(title, win_width, win_heigt);

    let texture_creator = canvas.texture_creator();
    Self {
      sdl_context, _image_context, ttf_context, _video_subsystem, _mixer_context, canvas, 
      window_dim: (win_width, win_heigt), texture_creator
    }
  }
}


// For now (TODO) everything is pub -- I'll see later...
pub struct HamGraph<'a> {
  pub renderer: Renderer<'a>,
  pub scene_stack: SceneStack,
  pub action_bus: ActionBus, // TODO not pub !
  pub(crate) layout_manager: LayoutManager,
  pub window_dim: (u32, u32),
  pub mixer_manager: MixerManager<'a>,
}
  
impl<'a> HamGraph<'a> {
  pub fn new(hamsdl2: &'a mut HamSdl2, mut root_scene: Box<dyn Scene>) -> Self {
    let sprite_store = SpriteStore::new(&hamsdl2.texture_creator);

    info!(target: TRAINIT, "Initializing HAMGRAPH...");
    
    let mut action_bus = ActionBus::new(sprite_store.shared_len());
    root_scene.init(&mut action_bus);

    let wdim = hamsdl2.canvas.window().size();
    info!(target: TRAINIT, "Window dimensions: {:?}", wdim);
    
    let layout_manager = LayoutManager::new(wdim);

    let mut font_store = FontStore::new();
    let scene_stack = SceneStack::new(root_scene, layout_manager.root_node_id);
    
    // Todo : this should of course be multiplied by the UI scale. 
    font_store.load_default_sized_fonts(&hamsdl2.ttf_context, &infraglobals::get_ttf_path());
    
    let renderer = Renderer::new(&hamsdl2.sdl_context, &mut hamsdl2._video_subsystem, &mut hamsdl2.canvas, sprite_store, font_store, &hamsdl2.texture_creator);
    Self {
      renderer,
      scene_stack, 
      action_bus, 
      layout_manager, 
      window_dim: hamsdl2.window_dim, 
      mixer_manager: MixerManager::new()
    }
  }

  // Push a scene onto the stack
  fn register_scene(&mut self, layer: usize, scene: Box<dyn Scene>, id: SceneID, parent: SceneID) {
    let mut real_parent: SceneID = 0;
    // If parent is already dead, just parent the scene to 0
    if self.scene_stack.get_scene(parent).is_some() {
      real_parent = parent;
    }
    debug!(target: TRASCENE, "Registering id=<{}>, parent=<{}>", id, real_parent);
    self.scene_stack.push(layer, scene, real_parent); 
    self.action_bus.prepare(id, self.scene_stack.next_scene_id());
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
        self.layout_manager.update_layout(); 
        let (_id_layout, nodeid_layout) = self.scene_stack.get_first_with_layout(action_p.source_scene);
        let style = self.layout_manager.get_style(nodeid_layout);

        let max_size = style.max_size;
        let max_width = if let taffy::style::Dimension::Length(width) = max_size.width {
          width as u32
        } else {
          0 // ... TODO 
        };
        
        // TODO : get max size of the parent container of the text 
        // Adapt the text to that max width 
        // how to adapt to max height ? Not so easy. See EGUI efforts instead.
        let fontfont = font + "_" + &size;
        let (w, h) = self.renderer.sprite_store.try_ttf_texture(
          &self.renderer.font_store,
          &fontfont,
          text,
          max_width,
        );

        self.renderer.sprite_store.commit_ttf_texture();
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
      _ => { 
        warn!(target: "hg::action", "!! User action left unhandled!");
      }
    }
  }

  pub fn run_main_loop(&mut self)
  {
    let mut event_pump = self.renderer.sdl_context.event_pump().unwrap_or_else(|e| {
      crate::errors::prompt_err_and_panic("SDL initialization error, no event pump", &e, None);
    });
  
    let target_frame_duration = Duration::from_secs_f32(1.0 / 60.0); // Targeting 60 FPS
    let mut last_update = Instant::now(); 

    'hamloop: loop {
      let mut v_sdl_events : Vec<(Action, EventKind)> = Vec::new();
      
      // 1. HANDLE EVENTS
      for event in event_pump.poll_iter() {
        let event_kind;
        match event {
          Event::Quit {..} |
          Event::KeyDown { keycode: Some(Keycode::Escape), .. } => { break 'hamloop }, // LEGACY TODO 
            
            // this won't be needed once the weird stuff will have been filtered.  TODO 
          Event::MouseButtonDown {mouse_btn: MouseButton::Left, ..} 
          | Event::MouseButtonUp {mouse_btn: MouseButton::Left, ..} => {
            event_kind = EventKind::SdlMouseClick;
          }, 
          Event::Window { win_event: WindowEvent::Resized(w, h), ..} => {
            // Window has been resized : update the UI tree 
            self.layout_manager.set_new_window_size((w as u32, h as u32)); // TODO important manage min 
            continue;
          }
          _ => { continue; /* Nothing for now */ }
        }
        // Propagate to egui 
        self.renderer.egui_platform.handle_event(&event, &self.renderer.sdl_context, &self.renderer.sdl_video);

        // Here we really want to propagate the event e.g. MouseButtonDown
        let action = Action::SdlEvent(event);
        v_sdl_events.push((action, event_kind));
      }

      // Begin "User Callback Zone"
      self.renderer.begin_egui_pass();

      // Propagate SDL actions 
      for (action, event_kind) in v_sdl_events {
        self.scene_stack.propagate_sdl2_to_subscribers(&mut self.action_bus, action, event_kind);
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
      let now = Instant::now(); // todo ... where should it be?
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
      self.renderer.canvas.set_draw_color(Color::RGB(0, 0, 0));
      self.renderer.canvas.clear();
      
      self.scene_stack.render_all(&mut self.renderer);
      self.renderer.end_egui_pass_and_paint();

      // 5. UPDATE SCREEN
      self.renderer.canvas.present();

      // Maintain a consistent frame rate
      let frame_duration = now.elapsed();
      if frame_duration < target_frame_duration { // TODO not needed with VSYNC ?
        std::thread::sleep(target_frame_duration - frame_duration);
      } // else application is quite overwhelmed! ... 
    }
  }
}
  