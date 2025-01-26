use sdl2::video::Window;

#[cfg(test)]
pub fn prompt_err_and_panic(message: &str, error: &str, _window: Option<&Window>) -> ! 
{
  panic!("{}: {}", message, error);
}

#[cfg(not(test))]
pub fn prompt_err_and_panic(message: &str, error: &str, window: Option<&Window>) -> ! 
{
  use sdl2::messagebox::*;
  // (in a real application I'd log the error before trying to prompt the msg box, TODO.
  show_simple_message_box(
    MessageBoxFlag::ERROR,
    "FATAL ERROR",
    &format!("{}: {}", message, error),
    window,
  ).unwrap(); 

  panic!("{}: {}", message, error);
}

// Prompt err, without panicking. 
pub fn prompt_err(message: &str, window: Option<&Window>) 
{
  use sdl2::messagebox::*;
  // (in a real application I'd log the error before trying to prompt the msg box, TODO.
  show_simple_message_box(
    MessageBoxFlag::ERROR,
    "FATAL ERROR",
    &format!("ERROR: {}", message),
    window,
  ).unwrap(); 
}
