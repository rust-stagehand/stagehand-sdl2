use log::{error, warn};
use sdl2::{
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
    utility::{Initialize, Update, UpdateInstruction},
    Stage,
};

use {input::SDLCommand, loading::SDLStorage};

mod app;

pub mod input;
pub mod loading;

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

    update: Update<SDLCommand, UContent>,

    storage: SDLStorage<'a, 'b, 'c>,
    texture_creator: &'a TextureCreator<WindowContext>,

    timer: TimerSubsystem,
}

impl<'a, 'b, 'c, IContent, UContent, Message> SDLApp<'a, 'b, 'c, IContent, UContent, Message> {
    pub fn new(
        sdl: Sdl,
        canvas: Canvas<Window>,
        input: InputMap<SDLCommand>,
        storage: SDLStorage<'a, 'b, 'c>,
        creator: &'a TextureCreator<WindowContext>,
        content: UContent,
    ) -> Result<Self, String> {
        let timer = sdl.timer()?;

        Ok(SDLApp {
            stage: Stage::new(),

            sdl,
            canvas,

            update: Update {
                input,
                info: Vec::new(),
                content,
            },

            storage,
            texture_creator: creator,

            timer,
        })
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
        match self.storage.music.get_by_ticket(ticket) {
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
        match self.storage.sounds.get_by_ticket(ticket) {
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
