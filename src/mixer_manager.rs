use sdl2::mixer::{Music, Chunk};
use std::collections::HashMap;

use crate::{errors::prompt_err_and_panic, infraglobals};

// TODO ! enum generation, sounddesc.json and so on.
// But for now this'd be really useless

pub struct MixerManager<'a> {
  music_store: HashMap<String, Music<'a>>,
  sfx_store: HashMap<String, Chunk>,
}

impl MixerManager<'_>
{
  // (TODO?) clippy "you should consider adding a `Default` implementation for `MixerManager<'_>`"
  pub fn new() -> Self {
    Self {
      music_store: HashMap::new(),
      sfx_store: HashMap::new(),
    }
  }

  pub fn load_music(&mut self, name: &str) {
    let music = Music::from_file(infraglobals::get_music_path().join(name))
      .unwrap_or_else(|err| { prompt_err_and_panic("load_music failed", &err, None) });
 
    self.music_store.insert(name.to_string(), music);
  }

  pub fn load_sfx(&mut self, name: &str) {
    let chunk = Chunk::from_file(infraglobals::get_sfx_path().join(name))
      .unwrap_or_else(|err| { prompt_err_and_panic("load_sfx failed", &err, None) });
    self.sfx_store.insert(name.to_string(), chunk);
  }

  pub fn play_music(&mut self, name: &str, loops: i32) {
    if let Some(music) = self.music_store.get(name) {
      music.play(loops).unwrap(); // `loops = -1` for infinite loop
    } else {
      // Attempt to load the music if not found
      self.load_music(name);
      let music = self.music_store.get(name).unwrap(); // Legitimate (load_music panics if problem)
      music.play(loops).unwrap();
    }
  }

  pub fn stop_music(&self) {
    Music::halt();
  }

  pub fn play_sfx(&mut self, name: &str, channel: i32) {
    if let Some(chunk) = self.sfx_store.get(name) {
      sdl2::mixer::Channel(channel).play(chunk, 0).unwrap();
    } else {
      // Attempt to load the sfx if not found
      self.load_sfx(name);
      let chunk = self.sfx_store.get(name).unwrap(); // Legitimate (load_sfx panics if problem)
      sdl2::mixer::Channel(channel).play(chunk, 0).unwrap();
    }
  } 
}
