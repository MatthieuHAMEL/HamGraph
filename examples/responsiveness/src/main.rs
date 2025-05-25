use std::path::PathBuf;

use hamgraph::{hg, HamGraph, HamSdl2};
use mainframe::RectangleScene;
use sdl2::pixels::Color;
use const_format::concatcp;
mod mainframe;

pub const IS_DEV_ENV: bool = cfg!(debug_assertions); // nb: not good 
pub const DEV_GAME_PATH: &str = concatcp!(env!("CARGO_MANIFEST_DIR"), "/userdata");
pub const DEV_JSON_SPRITE_PATH: &str = concatcp!(env!("CARGO_MANIFEST_DIR"), "/userdata/conf/spritedesc.json");

fn main() {
    // If in a development environment, use the project's folder
  if IS_DEV_ENV {
    hg::set_install_path(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("userdata"));
    hg::set_userdata_path(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("userdata"));
  } else {
    // TODO. Could this be choosed automatically by HAMGRAPH ? Or is it not a good thing ?
    hg::set_install_path(dirs::home_dir()
    .unwrap_or_else(|| { hamgraph::errors::prompt_err_and_panic("WHIP - no home dir!", "", None); })
    .join("/usr/local/whip0")); // TODO (improve on windows/linux etc) (distinguish install path / game path)
  }

  let mut sdl2_ctx = HamSdl2::new("RESPONSIVENESS", 1920, 930);
  
  let mut hg = HamGraph::new(
    &mut sdl2_ctx,
    Box::new(RectangleScene::new(Color::RGB(150, 170, 0), "main", false)));

  hg.run_main_loop();
}
