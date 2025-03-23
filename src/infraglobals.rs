use std::sync::OnceLock;
use std::path::{Path, PathBuf};

static USERDATA_PATH: OnceLock<PathBuf> = OnceLock::new();
static INSTALL_PATH: OnceLock<PathBuf> = OnceLock::new();

/// Sets the configuration path. Can only be set once.
pub fn set_userdata_path<P: AsRef<Path>>(path: P) {
  USERDATA_PATH
    .set(path.as_ref().to_path_buf())
    .expect("Configuration path can only be set once");
}

/// Gets the configuration path, or a default if it hasn't been set.
pub(crate) fn get_userdata_path() -> &'static Path {
  USERDATA_PATH.get_or_init(|| {
    dirs::data_local_dir()
      .unwrap_or_else(|| PathBuf::from("."))
      .join("hamgraph_default_invalid_path") // TODO throw or anything instead
  })
}

/// Same for install path
pub fn set_install_path<P: AsRef<Path>>(path: P) {
  INSTALL_PATH
    .set(path.as_ref().to_path_buf())
    .expect("Configuration path can only be set once");
}

pub(crate) fn get_install_path() -> &'static Path {
  INSTALL_PATH.get_or_init(|| {
    dirs::data_local_dir()
      .unwrap_or_else(|| PathBuf::from("."))
      .join("hamgraph_default_invalid_path")
  })
}

pub fn get_conf_path() -> PathBuf {
  get_install_path().join("conf")
}

pub fn get_img_path() -> PathBuf {
  get_install_path().join("img")
}

pub fn get_music_path() -> PathBuf {
  get_install_path().join("music")
}

pub fn get_sfx_path() -> PathBuf {
  get_install_path().join("sfx")
}

pub fn get_ttf_path() -> PathBuf {
  get_install_path().join("font")
}

pub fn get_logger_path() -> PathBuf {
  get_userdata_path().join("log")
}
