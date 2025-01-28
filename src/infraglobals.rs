use std::sync::OnceLock;
use std::path::{Path, PathBuf};

static GAME_PATH: OnceLock<PathBuf> = OnceLock::new();

/// Sets the configuration path. Can only be set once.
pub fn set_game_path<P: AsRef<Path>>(path: P) {
  GAME_PATH
    .set(path.as_ref().to_path_buf())
    .expect("Configuration path can only be set once");
}

/// Gets the configuration path, or a default if it hasn't been set.
pub(crate) fn get_game_path() -> &'static Path {
  GAME_PATH.get_or_init(|| {
    dirs::data_local_dir()
      .unwrap_or_else(|| PathBuf::from("."))
      .join("hamgraph_default_invalid_path")
  })
}

pub fn get_conf_path() -> PathBuf {
  get_game_path().join("conf")
}

pub fn get_img_path() -> PathBuf {
  get_game_path().join("img")
}

pub fn get_music_path() -> PathBuf {
  get_game_path().join("music")
}

pub fn get_sfx_path() -> PathBuf {
  get_game_path().join("sfx")
}

pub fn get_ttf_path() -> PathBuf {
  get_game_path().join("font")
}



