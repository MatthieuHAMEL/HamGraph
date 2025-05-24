This is my attempt to create a simple 2D game engine with Rust and the SDL2 crate.

Philosophy : 

- Object-oriented and Scene-based ;
- Implement the Scene interface, push Actions to interact with the engine or other scenes
  - Direct control on the renderer in every Scene

I use : 
- SDL2 for the windowing system + the SDL2 renderer
- Taffy to express and compute the CSS flexbox display
- EGUI for an immediate mode UI experience,
- Serde to offer a JSON config file describing sprite sheets and serialize it.

See also the HamGraph_Macros repo ...

