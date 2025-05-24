Various notes I took to understand this topic better : 

## CSS FlexBox 

It is (one of) the "best current" models to handle layout & responsiveness 

There are alternatives : fixed layout (like grid-based), which is generally horrible, some others are constraint-based and make use of fast constraint solvers like Cassowary ; this was used by Apple in the past but there's no reference lib. 

In the flexbox model, most dimensions are expressed relatively to the dimensions of the parent container (the top container being the window). But I can also give FlexBox absolute units. 

## Absolute units 

In CSS, a pixel (px) is defined as a 96 DPI pixel and is not so absolute: on browsers, it'll be sized accordingly to the system settings. 

-> I must multiply the absolute units I give to Taffy by a scale factor (I get it from the system if possible, or I can guess it based on the screen size and DPI ...)

-> Text and absolute dimensions given to egui must be scaled too. (by the same factor ? Or should I clamp for the text depending on the screen size and view angle // distance ? TODO)

-> The "scale" setting must be overridable by the user because whatever I apply, the distance User--Screen (i.e. the viewing distance, which defines the UI expectations) is unknown!

## The scale factor 

Starting from a physical size (in centimeters) is a good idea. Then convert to pixels. 

## Clamping 

Relative units look cool. Expressing a menu or a button size in pixels doesn't look cool. (with CSS pixels it'll be scaled depending on the DPI to preserve its physical size - but that size will remain the same whether it is displayed on an iPhone or a 4K TV. Which is a problem)

So "let that button be 30% of the screen width" is more promising, but not enough when the screen format changes. (On a PC 30% is fine for a centered menu, but on a smartphone in portrait format it'll be too narrow). To achieve true responsiveness, CSS has clamp() :

```
// Min 200px, ideally 30% of the width of the screen, at most 90%
inline-size: clamp(200px, 30%, 90%); 
```

I can mimic that with Taffy with the properties : min-size, size, max-size using relative or absolute units.

The pixels would be CSS pixels, then multiplied internally by the UI scale. And I could write something to allow specifying millimeters. 

## The problem of integrating egui (or any other framework) in a CSS layout tree (Taffy)

### Why do I integrate an immediate mode UI framework in HAMGRAPH ?

This is how to create a simple "MenuScene" in a retained fashion in HamGraph currently: 
- Create MenuScene
  - Create two children : TopBarScene and BottomBarScene
    - TopBarScene creates 2 children of ButtonScene type
    - BottomBarScene creates 3 children : a CheckBoxScene, a TextBoxScene and a ButtonScene. 

That's 8 scenes => the Scene trait has to be implemented 8 times. Not really light !

Though, the layouting of this menu is perfectly handled by HamGraph Taffy Tree, whether it is "from outside" (where the Menu should be in the screen relatively to other widgets), or "from inside" (how are the different little buttons and checkboxes displayed inside the Menu). Because each of those 8 scenes is a Node in the taffy tree and each scene can request a Layout (Action::RequestLayout), that is, a bunch of CSS properties (flex-direction, align-items, min_size, size, max_size, etc.).

Immediate mode (Egui's) goal is to replace all of this 8-scenes taffy branch by a 10 lines code, basically. 

### Why it's tough 

Though, I want Egui frames (Areas, Windows, etc.) to behave like scenes. First, if I create a menu with Egui, I still want the menu to be correctly positioned relatively to my other widgets in the Taffy tree. Consequently **the EguiScene has to be part of the Taffy tree**.

By definition, the "inside", immediatly-rendered stuff (buttons, etc) in my EguiScene are not part of the Taffy tree and obey to egui's rules. (I looked at egui_taffy but it doesn't allow to merge my tree with their...). It follows that the EguiScene only acts like a **leaf** in the Taffy tree. 

But, even though it happens in its own world, the Egui frame still acts like a container (a node!). Taffy will treat it like a leaf and communicate its size, but the Egui frame may want to grow and shrink, and it also has to communicate a "preffered size" to Taffy. This means I have to implement the "**negociation**" process that happens in CSS implementation. 

## Sources / useful threads and articles

- https://www.reddit.com/r/Windows10/comments/6we5ys/how_does_windows_10_calculate_the_recommended_dpi/
- https://developer.mozilla.org/fr/docs/Web/CSS/clamp