use sdl2::rect::Rect;

pub fn is_point_in_rect(rect: &Rect, x: i32, y: i32) -> bool 
{
  x >= rect.x() &&
  x < rect.x() + rect.width() as i32 &&
  y >= rect.y() && 
  y < rect.y() + rect.height() as i32
}
