use log::warn;
use sdl2::{
    controller::{Axis, Button},
    keyboard::Scancode,
    mixer::{InitFlag, AUDIO_S16LSB, DEFAULT_CHANNELS},
    mouse::MouseButton,
};

use stagehand::{
    app::gameloop,
    example::{ui::UIScene, ExampleScene},
    input::{ActionState, ActionType, InputMap},
    scene::Scene,
    utility::Initialize,
};

use stagehand_sdl2::{
    input::{SDLCommand, SDLGamepadFeature},
    loading::{FontLoader, SDLStorage, TextureLoader},
    SDLApp,
};

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    sdl_context.audio()?;

    sdl2::image::init(sdl2::image::InitFlag::PNG)?;

    sdl2::mixer::open_audio(44100, AUDIO_S16LSB, DEFAULT_CHANNELS, 1024)?;
    sdl2::mixer::init(InitFlag::MP3)?;
    sdl2::mixer::allocate_channels(4);

    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;

    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("Stagehand SDL2 Example", 800, 600)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let texture_creator = canvas.texture_creator();

    let controller_system = sdl_context.game_controller()?;
    let num_joysticks = controller_system.num_joysticks()?;

    let mut controllers = Vec::new();
    for index in 0..num_joysticks {
        if !controller_system.is_game_controller(index) {
            continue;
        }

        match controller_system.open(index) {
            Ok(c) => {
                controllers.push(c);
            }
            Err(e) => {
                warn!("Problem opening controller: {}", e);
            }
        };
    }

    let texture_loader = TextureLoader::from_creator(texture_creator);
    let font_loader = FontLoader::from_context(ttf_context);

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

    storage.textures.lock();
    storage.music.lock();
    storage.sounds.lock();
    storage.fonts.lock();

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

    let mut initialize = Initialize::<SDLCommand, SDLStorage, ()>::new(input, storage, ());

    let mut scene = ExampleScene::new();
    scene.initialize(&mut initialize);

    let mut ui = UIScene::new();
    ui.initialize(&mut initialize);

    let mut app = SDLApp::<(), (), String>::new(
        sdl_context,
        canvas,
        controllers,
        initialize.input,
        initialize.storage,
        &texture_loader.creator,
        (),
    )?;

    app.add_scene("Example".to_string(), Box::new(scene), true);
    app.add_scene("UI".to_string(), Box::new(ui), true);

    gameloop(&mut app, 60)?;

    Ok(())
}
