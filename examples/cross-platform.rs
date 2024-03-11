use sdl2::{
    keyboard::Scancode,
    mixer::{InitFlag, AUDIO_S16LSB, DEFAULT_CHANNELS},
};

use stagehand::{
    app::gameloop,
    example::ExampleScene,
    input::{ActionState, ActionType, InputMap},
    scene::Scene,
    utility2d::Initialize,
};

use stagehand_sdl2::{
    input::SDLCommand,
    loading::{SDLStorage, TextureLoader},
    SDLApp,
};

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    sdl_context.audio()?;
    sdl2::image::init(sdl2::image::InitFlag::PNG)?;
    sdl2::mixer::open_audio(44100, AUDIO_S16LSB, DEFAULT_CHANNELS, 1024)?;
    sdl2::mixer::init(InitFlag::MP3)?;

    sdl2::mixer::allocate_channels(4);

    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem
        .window("Stagehand SDL2 Example", 800, 600)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let texture_creator = canvas.texture_creator();

    let mut input = InputMap::<SDLCommand>::new();
    let player = input.add_user();
    input
        .add_action(
            player,
            "Forward".to_string(),
            vec![
                SDLCommand::Key(vec![Scancode::W]),
                SDLCommand::Key(vec![Scancode::Up]),
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
                SDLCommand::Key(vec![Scancode::Down]),
            ],
            ActionType::Digital(ActionState::Up),
        )
        .unwrap();
    input
        .add_action(
            player,
            "Look".to_string(),
            vec![],
            ActionType::Analog { x: 0.0, y: 0.0 },
        )
        .unwrap();
    input
        .add_action(
            player,
            "Pause".to_string(),
            vec![],
            ActionType::Digital(ActionState::Up),
        )
        .unwrap();

    let texture_loader = TextureLoader::from_creator(texture_creator);
    let mut storage = SDLStorage::new(&texture_loader);
    storage
        .textures
        .load("Logo.png".to_string(), "example-assets/Logo.png")
        .unwrap();

    storage
        .music
        .load("Music.wav".to_string(), "example-assets/Music.wav")
        .unwrap();

    storage.textures.lock();
    storage.music.lock();

    let mut initialize = Initialize::<SDLCommand, SDLStorage>::new(input, storage);

    let mut scene = ExampleScene::new();
    scene.initialize(&mut initialize);

    let mut app = SDLApp::new(sdl_context, canvas, initialize.input, initialize.content)?;

    app.add_scene(Box::new(scene));

    gameloop(&mut app, 60)?;

    Ok(())
}
