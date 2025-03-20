use std::{collections::HashMap, ffi::OsStr, fs, path::Path};

use sdl2::ttf::{Font, Sdl2TtfContext};

pub struct FontStore<'a> {
  fonts: HashMap<String, Font<'a, 'static>>,
}

impl<'a> FontStore<'a> {
  pub fn new() -> Self {
    Self { fonts: HashMap::new() }
  }

  pub fn get(&'a self, name: String) -> &'a Font<'a, 'static> {
    &self.fonts.get(&name).unwrap_or_else(|| {
      println!();
      panic!("{}", format!("No such font !... {}", &name))
    })
  }

  pub fn load_from_folder(
    &mut self, 
    ttf_context: &'a Sdl2TtfContext,
    fonts_folder: &Path,
    font_size: u16,
    category: &str, // the engine will load by default : "small", "medium", "big"
  )
  {
    // Read the directory entries
    let entries = fs::read_dir(fonts_folder)
      .unwrap_or_else(|e| panic!("{}", format!("Failed to read directory: {}", e)));

    for entry in entries 
    {
      let entry = entry
        .unwrap_or_else(|e| panic!("{}", format!("Failed to read entry: {}", e)));
      let path = entry.path();

      // Check if it's a file ending in .ttf
      if path.is_file() 
      {
        if let Some(ext) = path.extension() 
        {
          if ext == OsStr::new("ttf") {
            // The file stem is the name without .ttf
            if let Some(stem) = path.file_stem() 
            {
              let key = stem.to_string_lossy().to_string() + "_" + category;

              // Load the font
              let font = ttf_context
                .load_font(&path, font_size)
                .unwrap_or_else(|e| panic!("{}", format!("Failed to load font: {}", e)));

              // Insert into the HashMap
              self.fonts.insert(key, font);
            }
          }
        }
      }
    }
  }
}
