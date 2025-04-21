use std::{collections::HashMap, ffi::OsStr, fs, path::Path};
use sdl2::ttf::{Font, Sdl2TtfContext};

pub struct FontStore<'a> {
  fonts: HashMap<String, Font<'a, 'static>>,
}

impl<'a> FontStore<'a> {
  // (TODO?) clippy "you should consider adding a `Default` implementation for `FontStore<'a>`"
  pub fn new() -> Self {
    Self { fonts: HashMap::new() }
  }

  pub fn get(&'a self, name: &str) -> &'a Font<'a, 'static> {
    self.fonts.get(name).unwrap_or_else(|| {
      panic!("{}", format!("No such font !... {}", &name))
    })
  }

  // Loads all the fonts from the folder in the 3 sizes 
  pub fn load_default_sized_fonts(&mut self, ttf_context: &'a Sdl2TtfContext, fonts_folder: &Path) {
    self.load_from_folder(ttf_context, fonts_folder, 12u16, "small");
    self.load_from_folder(ttf_context, fonts_folder, 20u16, "medium");
    self.load_from_folder(ttf_context, fonts_folder, 30u16, "big");
  }
  
  pub fn load_from_folder(
    &mut self, 
    ttf_context: &'a Sdl2TtfContext,
    fonts_folder: &Path,
    font_size: u16,
    category: &str, // the engine will load by default : "small", "medium", "big" 
    // (TODO: I'll just have to decide what "small", "medium" or "big" means depending on the DPI & screen size)
  )
  {
    // Read the directory entries
    let entries = fs::read_dir(fonts_folder)
      .unwrap_or_else(|e| panic!("{}", format!("Failed to read directory: {}", e)));

    for entry in entries {
      let entry = entry.unwrap_or_else(|e| panic!("{}", format!("Failed to read entry: {}", e)));
      let path = entry.path();

      // Check if it's a file ending in .ttf
      if path.is_file() {
        if let Some(ext) = path.extension() {
          if ext == OsStr::new("ttf") {
            if let Some(stem) = path.file_stem() { // filename without .ttf
              let key = stem.to_string_lossy().to_string() + "_" + category;

              // Load the font and put it in the hashmap
              let font = ttf_context
                .load_font(&path, font_size)
                .unwrap_or_else(|e| panic!("{}", format!("Failed to load font: {}", e)));
              self.fonts.insert(key, font);
            }
          }
        }
      }
    }
  }
}
