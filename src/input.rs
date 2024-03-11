use sdl2::{keyboard::Scancode, mouse::MouseButton};

pub enum SDLCommand {
    Key(Vec<Scancode>),
    MouseButton(Vec<MouseButton>),
    MousePosition,
    GamepadButton,
    GamepadAxis,
}
