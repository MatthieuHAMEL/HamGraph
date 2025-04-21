use sdl2::rect::Rect;
use taffy::{prelude::{length, percent, TaffyMaxContent}, Dimension, FlexDirection, FlexWrap, NodeId, Size, Style, TaffyTree};

use crate::scene::{SceneID, SceneStack};

// This module is an abstraction over the taffy layout.

use bitflags::bitflags;
bitflags! 
{
  /// Flags representing the different kinds of actions.
  /// Each bit stands for one type of action.
  #[derive(Clone, PartialEq)]
  pub struct BoxFlags: u64 {
    const DefaultBox        = 0;
    const ItemsInRow        = 0b0000_0001;
    const ItemsInCol        = 0b0000_0010;
    const ItemsDirReversed  = 0b0000_0100;
    const WrapItems         = 0b0000_1000;
    const WrapItemsReversed = 0b0001_0000;
    const NoWrap            = 0b0010_0000;
  }
}

pub type JustifyContent = taffy::JustifyContent;
pub type AlignItems = taffy::AlignItems;
pub type AlignContent = taffy::AlignContent;
pub type AlignSelf = taffy::AlignSelf;


#[derive(Clone)]
/// TODO maybe absolute location won't be needed at all 
/// Why should an absolutely-positioned scene request a layout ? 
///   position can be absolute though the user might still want it to be computed wrt the window size
/// // NB: it has no interest in practice but it'd be useful to centralize that option 
/// in the layout struct, still ...so TODO 
/// TODO play with Padding too
pub struct Layout {
  // Universal option
  pub size: (Option<f32>, Option<f32>), // In percentage of the parent container 

  // Parent (container) options
  pub flex: BoxFlags, 
  pub justify_content: JustifyContent, 
  pub align_items: AlignItems,
  pub align_content: Option<AlignContent>,
  pub gap: (f32, f32), // Idem 

  // Child (item) options
  pub grow: f32, 
  pub align_self: Option<AlignSelf>
}

impl Default for Layout {
  fn default() -> Layout {
    Layout {
      size: (None, None), // means auto  
      flex: BoxFlags::DefaultBox, 
      justify_content:JustifyContent::FlexStart, 
      align_items: AlignItems::Stretch,  // So that if no size is specified on some element, things are still visible 
      align_content: None, // TODO no test for now 
      gap: (0.1, 0.1), 
      grow: 1.0, 
      align_self: None, // No self alignment by default 
    }
  }
}

pub fn taffy_style(opts: &Layout) -> Style {
  // Start with the default taffy style ... 
  let mut taffy_style = taffy::Style { ..Default::default() };

  if let Some(x) = opts.size.0 {
    taffy_style.size.width = Dimension::Percent(x);
  }
  else {
    taffy_style.size.width = Dimension::Auto;
  }

  if let Some(y) = opts.size.1 {
    taffy_style.size.height = Dimension::Percent(y);
  }
  else {
    taffy_style.size.height = Dimension::Auto;
  }

  if opts.flex != BoxFlags::DefaultBox {
    taffy_style.flex_direction = if opts.flex.contains(BoxFlags::ItemsInRow) {
      if opts.flex.contains(BoxFlags::ItemsDirReversed) {
        FlexDirection::RowReverse
      } else {
        FlexDirection::Row
      }
    } 
    else if opts.flex.contains(BoxFlags::ItemsInCol) {
      if opts.flex.contains(BoxFlags::ItemsDirReversed) {
        FlexDirection::ColumnReverse
      } else {
        FlexDirection::Column
      }
    } else { taffy_style.flex_direction};

    // Wrap vs. Nowrap vs. WrapReverse
    taffy_style.flex_wrap = if opts.flex.contains(BoxFlags::WrapItemsReversed) {
      FlexWrap::WrapReverse
    } else if opts.flex.contains(BoxFlags::NoWrap) {
      FlexWrap::NoWrap
    } else { // Default 
      FlexWrap::Wrap
    };
  }
  else { // default 
    taffy_style.flex_wrap = FlexWrap::Wrap;
    taffy_style.flex_direction = FlexDirection::Row;
  }

  // 4. Justify / Align items / content
  taffy_style.justify_content = Some(opts.justify_content);
  taffy_style.align_items = Some(opts.align_items);

  taffy_style.align_content = opts.align_content;

  // 5. Gap in percent
  taffy_style.gap = Size {width: percent(opts.gap.0), height: percent(opts.gap.1)};

  // 6. Child properties: grow, align_self
  taffy_style.flex_grow = opts.grow;

  if let Some(asf) = opts.align_self {
    taffy_style.align_self = Some(asf);
  }
  taffy_style
}

pub(crate) struct LayoutManager {
  pub(crate) taffy_tree: TaffyTree,
  pub(crate) root_node_id: NodeId,
}

impl LayoutManager {
  pub fn new(wdim: (u32, u32)) -> Self {
    let mut taffy_tree: TaffyTree<()> = TaffyTree::new();
    let root_node_id = taffy_tree.new_leaf(
      Style {
        size: Size { width: length(wdim.0 as f32), height: length(wdim.1 as f32) },
          flex_grow: 1.0,
          ..Default::default()
        },
    ).unwrap();

    Self {taffy_tree, root_node_id}
  }

  pub fn update_layout(&mut self) -> bool {
    if self.taffy_tree.dirty(self.root_node_id).unwrap() {
      self.taffy_tree.compute_layout(self.root_node_id, Size::MAX_CONTENT).unwrap();
      return true;
    }
    false
  }

  pub fn abs_layout(&self, node: NodeId) -> Rect {  
    let layout = self.taffy_tree.layout(node).unwrap();
    let mut res = Rect::new(layout.location.x as i32, 
      layout.location.y as i32, 
      layout.size.width as u32, 
      layout.size.height as u32);
    
    let mut par = self.taffy_tree.parent(node);
    while par.is_some() {
      res.x += self.taffy_tree.layout(par.unwrap()).unwrap().location.x as i32;
      res.y += self.taffy_tree.layout(par.unwrap()).unwrap().location.y as i32;

      par = self.taffy_tree.parent(par.unwrap());
    }
    res
  }

  pub fn set_layout(&mut self, requesting: SceneID, scene_stack: &mut SceneStack, lay: Layout) {
    // Convert the HAMGRAPH layout to a TAFFY layout : 
    let style = taffy_style(&lay); 

    // Maybe there is already a layout : 
    if let Some(nodeid_requesting) = scene_stack.nodeid(requesting) {
      let _ = self.taffy_tree.set_style(nodeid_requesting, style);
      return;
    }
    // Else we have to add the node corresponding to that scene :
    let sceneid_parent = scene_stack.parent(requesting);
    if let Some(nodeid_parent) = scene_stack.nodeid(sceneid_parent) {
      let new_nodeid = self.taffy_tree.new_leaf(style).unwrap();
      let _ = self.taffy_tree.add_child(nodeid_parent, new_nodeid);
      scene_stack.set_nodeid(requesting, new_nodeid);
    }
    else {
      panic!("set_layout requested on a scene id=<{}> whose parent id=<{}> did not request a layout !!!", requesting, sceneid_parent)
    }
  }

  pub fn remove_layout(&mut self, node_id: NodeId) {
    self.taffy_tree.remove(node_id).unwrap();
  }

  pub fn set_new_window_size(&mut self, wdim: (u32, u32)) {
    self.taffy_tree.set_style(self.root_node_id,
      Style {
        size: Size { width: length(wdim.0 as f32), height: length(wdim.1 as f32) },
          flex_grow: 1.0,
          ..Default::default()
        },
    ).unwrap();
  }

  pub fn get_style(&self, node_id: NodeId) -> &Style {
    self.taffy_tree.style(node_id).unwrap()
  }
}
