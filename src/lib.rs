use loading::{FontLoader, TextureLoader};
use log::{error, warn};
use sdl2::{
    controller::GameController,
    mixer::{InitFlag, AUDIO_S16LSB, DEFAULT_CHANNELS},
    pixels::Color,
    rect::{Point, Rect},
    render::{Canvas, Texture, TextureCreator},
    video::{Window, WindowContext},
    Sdl, TimerSubsystem,
};
use std::{cell::RefCell, rc::Rc};

use stagehand::{
    draw::{Draw, DrawBatch, DrawColor, DrawData, DrawDestination, DrawRect},
    input::InputMap,
    loading::{ResourceError, Ticket},
    scene::Scene,
    utility::{Initialize, Update, UpdateInfo, UpdateInstruction},
    Stage,
};

use {input::SDLCommand, loading::SDLStorage};

mod app;

pub mod input;
pub mod loading;

pub fn initialize_sdl2<'a, 'c>() -> Result<
    (
        Sdl,
        Canvas<Window>,
        TextureLoader<'a, WindowContext>,
        FontLoader<'a, 'c>,
    ),
    String,
> {
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

    let texture_loader = TextureLoader::from_creator(texture_creator);
    let font_loader = FontLoader::from_context(ttf_context);

    Ok((sdl_context, canvas, texture_loader, font_loader))
}

pub struct SDLApp<'a, 'b, 'c, IContent, UContent, Message> {
    stage: Stage<
        'a,
        String,
        Initialize<SDLCommand, SDLStorage<'a, 'b, 'c>, IContent>,
        Update<SDLCommand, UContent>,
        Message,
        UpdateInstruction,
        (),
        DrawBatch<Draw, ()>,
    >,

    sdl: Sdl,
    canvas: Canvas<Window>,
    controllers: Vec<GameController>,

    i_content: Rc<RefCell<IContent>>,
    u_content: Rc<RefCell<UContent>>,
    input: Rc<RefCell<InputMap<SDLCommand>>>,
    info: Rc<RefCell<Vec<UpdateInfo>>>,

    storage: Rc<RefCell<SDLStorage<'a, 'b, 'c>>>,
    texture_creator: &'a TextureCreator<WindowContext>,

    timer: TimerSubsystem,
}

impl<'a, 'b, 'c, IContent, UContent, Message> SDLApp<'a, 'b, 'c, IContent, UContent, Message> {
    pub fn from_loader(
        context: Sdl,
        canvas: Canvas<Window>,
        texture: &'a TextureLoader<'a, WindowContext>,
        input: Rc<RefCell<InputMap<SDLCommand>>>,
        storage: Rc<RefCell<SDLStorage<'a, 'b, 'c>>>,
        i_content: Rc<RefCell<IContent>>,
        u_content: Rc<RefCell<UContent>>,
    ) -> Result<Self, String> {
        let controller_system = context.game_controller()?;
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

        Self::new(
            context,
            canvas,
            controllers,
            input,
            storage,
            &texture.creator,
            i_content,
            u_content,
        )
    }

    pub fn new(
        sdl: Sdl,
        canvas: Canvas<Window>,
        controllers: Vec<GameController>,
        input: Rc<RefCell<InputMap<SDLCommand>>>,
        storage: Rc<RefCell<SDLStorage<'a, 'b, 'c>>>,
        creator: &'a TextureCreator<WindowContext>,
        i_content: Rc<RefCell<IContent>>,
        u_content: Rc<RefCell<UContent>>,
    ) -> Result<Self, String> {
        let timer = sdl.timer()?;

        Ok(SDLApp {
            stage: Stage::new(),

            sdl,
            canvas,
            controllers,

            i_content: i_content,
            u_content: u_content,
            input: input,
            info: Rc::new(RefCell::new(Vec::new())),

            storage,
            texture_creator: creator,

            timer,
        })
    }

    pub fn prepare_info(&mut self) {
        let mut info = self.info.borrow_mut();
        info.clear();

        if !sdl2::mixer::Music::is_playing() {
            info.push(UpdateInfo::MusicStopped);
        }
    }

    pub fn add_scene(
        &mut self,
        key: String,
        scene: Box<
            dyn Scene<
                    Key = String,
                    Initialize = Initialize<SDLCommand, SDLStorage<'a, 'b, 'c>, IContent>,
                    Update = Update<SDLCommand, UContent>,
                    Message = Message,
                    Instruction = UpdateInstruction,
                    Draw = (),
                    DrawBatch = DrawBatch<Draw, ()>,
                > + 'a,
        >,
        active: bool,
    ) {
        self.stage.add_scene(key, scene, active);
    }

    fn volume(v: f32) -> i32 {
        (v * sdl2::mixer::MAX_VOLUME as f32) as i32
    }

    fn play_music(&mut self, ticket: Ticket, loops: i32, volume: f32) {
        match self.storage.borrow().music.get_by_ticket(ticket) {
            Ok(m) => {
                sdl2::mixer::Music::set_volume(Self::volume(volume));
                match m.borrow().play(loops) {
                    Ok(()) => {}
                    Err(e) => error!("Error playing music: {}", e),
                }
            }
            Err(e) => ResourceError::log_failure(e),
        }
    }

    fn play_sound(&mut self, ticket: Ticket, volume: f32) {
        match self.storage.borrow().sounds.get_by_ticket(ticket) {
            Ok(s) => {
                match s.try_borrow_mut() {
                    Ok(mut s_v) => {
                        s_v.set_volume(Self::volume(volume));
                    }
                    Err(e) => warn!(
                        "Cannot set volume on a sound effect already borrowed elsewhere: {}",
                        e
                    ),
                }

                match sdl2::mixer::Channel::all().play(&s.borrow(), 0) {
                    Ok(_c) => {}
                    Err(e) => error!("Error playing sound: {}", e),
                }
            }
            Err(e) => ResourceError::log_failure(e),
        }
    }

    fn render_texture(&mut self, texture: Rc<RefCell<Texture<'_>>>, data: &DrawData) {
        let tex = texture.borrow();
        let query = tex.query();

        let source = match &data.source {
            Some(r) => Some(to_rect(r)),
            None => None,
        };

        let (angle, origin) = match &data.rotation {
            Some(r) => (
                r.angle as f64,
                Point::new(
                    (r.origin.0 * query.width as f32) as i32,
                    (r.origin.1 * query.height as f32) as i32,
                ),
            ),
            None => (0.0, Point::new(0, 0)),
        };

        let dest = match &data.destination {
            Some(d) => match d {
                DrawDestination::Location { x, y } => Some(Rect::new(
                    (*x as i32) - origin.x,
                    (*y as i32) - origin.y,
                    query.width,
                    query.height,
                )),
                DrawDestination::Rect(rect) => Some(to_rect(rect)),
            },
            None => None,
        };

        let (horizontal, vertical) = match &data.flip {
            Some(f) => (f.horizontal, f.vertical),
            None => (false, false),
        };

        if let Err(e) = self
            .canvas
            .copy_ex(&tex, source, dest, angle, origin, horizontal, vertical)
        {
            warn!("SDL2 Texture Rendering failed: {}", e);
        }
    }
}

fn to_rect(r: &DrawRect) -> Rect {
    Rect::new(r.x as i32, r.y as i32, r.width as u32, r.height as u32)
}

fn to_color(c: &DrawColor) -> Color {
    let max = u8::MAX as f32;
    Color::RGBA(
        (max * c.r) as u8,
        (max * c.g) as u8,
        (max * c.b) as u8,
        (max * c.a) as u8,
    )
}
