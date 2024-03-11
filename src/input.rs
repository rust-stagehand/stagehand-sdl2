use sdl2::keyboard::Scancode;

pub enum SDLCommand {
    Key(Vec<Scancode>),
    MouseButton,
    MousePosition,
    GamepadButton,
    GamepadAxis,
}
