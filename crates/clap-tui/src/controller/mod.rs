mod keyboard;
mod mouse;
pub(crate) mod navigation;

pub(crate) use keyboard::handle_key_event;
pub(crate) use mouse::handle_mouse_event;
