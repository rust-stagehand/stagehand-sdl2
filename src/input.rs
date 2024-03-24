use sdl2::{
    controller::{Axis, Button},
    keyboard::Scancode,
    mouse::MouseButton,
    sys::{SDL_JOYSTICK_AXIS_MAX, SDL_JOYSTICK_AXIS_MIN},
};

pub enum SDLCommand {
    Key(Vec<Scancode>),
    MouseButton(Vec<MouseButton>),
    MousePosition,
    Gamepad(SDLGamepadFeature, Option<usize>),
}

pub enum SDLGamepadFeature {
    Button(Vec<Button>),
    Axis(Axis),
    Stick(Axis, Axis),
}

pub fn translate_axis(axis: i16) -> f32 {
    if axis >= 0 {
        axis as f32 / SDL_JOYSTICK_AXIS_MAX as f32
    } else {
        -(axis as f32 / SDL_JOYSTICK_AXIS_MIN as f32)
    }
}
