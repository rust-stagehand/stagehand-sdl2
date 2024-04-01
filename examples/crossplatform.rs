use sdl2::{
    controller::{Axis, Button},
    keyboard::Scancode,
    mouse::MouseButton,
};

use stagehand::{
    app::gameloop,
    example::{ui::UIScene, ExampleScene},
    input::{ActionState, ActionType, InputMap},
};

use stagehand_sdl2::{
    initialize_sdl2,
    input::{SDLCommand, SDLGamepadFeature},
    loading::SDLStorage,
    SDLApp,
};

fn main() -> Result<(), String> {
    let (context, canvas, texture_loader, font_loader) = initialize_sdl2()?;

    let mut storage = SDLStorage::new(&texture_loader, &font_loader);
    storage
        .textures
        .load("Logo.png".to_string(), "example-assets/Logo.png")
        .unwrap();

    storage
        .music
        .load("Music.wav".to_string(), "example-assets/Music.wav")
        .unwrap();

    storage
        .sounds
        .load("OoB.wav".to_string(), "example-assets/OoB.wav")
        .unwrap();

    storage
        .fonts
        .load(
            "Napalm.ttf".to_string(),
            &("example-assets/OperationNapalm.ttf", 32),
        )
        .unwrap();
    storage.lock();

    let mut input = InputMap::<SDLCommand>::new();
    let player = input.add_user();
    input
        .add_action(
            player,
            "Forward".to_string(),
            vec![
                SDLCommand::Key(vec![Scancode::W]),
                SDLCommand::Key(vec![Scancode::Up, Scancode::LShift]),
                SDLCommand::MouseButton(vec![MouseButton::Left]),
                SDLCommand::Gamepad(SDLGamepadFeature::Button(vec![Button::DPadUp]), None),
            ],
            ActionType::Digital(ActionState::Up),
        )
        .unwrap();
    input
        .add_action(
            player,
            "Backward".to_string(),
            vec![
                SDLCommand::Key(vec![Scancode::S]),
                SDLCommand::Key(vec![Scancode::Down, Scancode::LShift]),
                SDLCommand::MouseButton(vec![MouseButton::Right]),
                SDLCommand::Gamepad(SDLGamepadFeature::Button(vec![Button::DPadDown]), None),
            ],
            ActionType::Digital(ActionState::Up),
        )
        .unwrap();
    input
        .add_action(
            player,
            "Look".to_string(),
            vec![
                SDLCommand::MousePosition,
                SDLCommand::Gamepad(SDLGamepadFeature::Stick(Axis::RightX, Axis::RightY), None),
            ],
            ActionType::Analog { x: 0.0, y: 0.0 },
        )
        .unwrap();
    input
        .add_action(
            player,
            "Pause".to_string(),
            vec![
                SDLCommand::Key(vec![Scancode::Escape]),
                SDLCommand::MouseButton(vec![MouseButton::Middle]),
                SDLCommand::Gamepad(SDLGamepadFeature::Button(vec![Button::A]), None),
            ],
            ActionType::Digital(ActionState::Up),
        )
        .unwrap();

    let mut app = SDLApp::from_loader(context, canvas, &texture_loader, input, storage, (), ())?;

    let scene = ExampleScene::new();
    let ui = UIScene::new();

    app.add_scene("Example".to_string(), Box::new(scene), true, true);
    app.add_scene("UI".to_string(), Box::new(ui), true, true);

    gameloop(&mut app, 60)?;

    Ok(())
}
