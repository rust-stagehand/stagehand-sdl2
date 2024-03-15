use std::{cell::RefCell, rc::Rc};

use log::{error, warn};
use sdl2::{event::Event, pixels::Color};

use stagehand::{
    app::App,
    draw::DrawType,
    input::{ActionState, ActionType, InputError},
    loading::ResourceError,
    utility::{UpdateInfo, UpdateInstruction},
    StageError,
};

use crate::{input::SDLCommand, SDLApp};

impl<'a, 'b, 'c, IContent, UContent, Message> App
    for SDLApp<'a, 'b, 'c, IContent, UContent, Message>
{
    type EventError = String;

    fn ticks(&self) -> u64 {
        self.timer.ticks64()
    }

    fn processed_events(&mut self) -> Result<bool, String> {
        let mut events = self.sdl.event_pump()?;

        for event in events.poll_iter() {
            match event {
                Event::Quit { .. } => {
                    return Ok(false);
                }
                _ => {}
            }
        }

        let keys = events.keyboard_state();
        let mouse = events.mouse_state();

        for command_options in self.update.input.commands.iter() {
            let mut active = ActionType::Digital(ActionState::Up);
            'commands: for command in command_options.commands.iter() {
                match command {
                    SDLCommand::Key(c) => 'key: {
                        for key in c.iter() {
                            if !keys.is_scancode_pressed(*key) {
                                break 'key;
                            }
                        }
                        active = ActionType::Digital(ActionState::Down);
                        break 'commands;
                    }
                    SDLCommand::MouseButton(b) => 'button: {
                        for button in b.iter() {
                            if !mouse.is_mouse_button_pressed(*button) {
                                break 'button;
                            }
                        }
                        active = ActionType::Digital(ActionState::Down);
                        break 'commands;
                    }
                    SDLCommand::MousePosition => {
                        active = ActionType::Analog {
                            x: mouse.x() as f32,
                            y: mouse.y() as f32,
                        };
                    }
                    _ => {}
                };
            }

            match self.update.input.users[command_options.user_index]
                .update_action(command_options.action_index, active)
            {
                Err(e) => match e {
                    InputError::ActionIndexOutOfBounds => {
                        error!("Action index not found: {}", command_options.action_index)
                    }
                    _ => {}
                },
                _ => {}
            };
        }

        Ok(true)
    }

    fn update(&mut self, delta: f64) {
        self.update.info.clear();

        if !sdl2::mixer::Music::is_playing() {
            self.update.info.push(UpdateInfo::MusicStopped);
        }

        match self.stage.update(&self.update, delta) {
            Ok(v) => {
                for instruction in v.iter() {
                    match instruction {
                        UpdateInstruction::PlayMusic(ticket, loops, volume) => {
                            self.play_music(*ticket, *loops, *volume)
                        }
                        UpdateInstruction::PlaySound(ticket, volume) => {
                            self.play_sound(*ticket, *volume)
                        }
                    }
                }
            }
            Err(e) => match e {
                StageError::NoScenesToUpdateError => warn!("Stage has no scenes to update."),
                _ => {}
            },
        }
    }

    fn draw(&mut self, interp: f64, _total_time: u64) {
        self.canvas.set_draw_color(Color::RGB(55, 55, 55));
        self.canvas.clear();

        let batches = match self.stage.draw(&(), interp) {
            Ok(b) => b,
            Err(e) => {
                match e {
                    StageError::NoScenesToDrawError => warn!("Stage has no scenes to draw."),
                    _ => {}
                }

                return;
            }
        };

        for batch in batches.iter() {
            for draw in batch.instructions.iter() {
                let texture = match &draw.draw_type {
                    DrawType::Texture => match self.storage.textures.get_by_ticket(draw.ticket) {
                        Ok(t) => t,
                        Err(e) => {
                            ResourceError::log_failure(e);
                            return;
                        }
                    },
                    DrawType::Text(s, c) => match self.storage.fonts.get_by_ticket(draw.ticket) {
                        Ok(f) => {
                            let surface = match f
                                .borrow()
                                .render(&s)
                                .blended(super::to_color(&c))
                                .map_err(|e| e.to_string())
                            {
                                Ok(s) => s,
                                Err(e) => {
                                    error!("Error rendering font: {}", e);
                                    return;
                                }
                            };
                            let texture = match self
                                .texture_creator
                                .create_texture_from_surface(&surface)
                                .map_err(|e| e.to_string())
                            {
                                Ok(t) => t,
                                Err(e) => {
                                    error!("Error transferring text surface to texture: {}", e);
                                    return;
                                }
                            };

                            Rc::new(RefCell::new(texture))
                        }
                        Err(e) => {
                            ResourceError::log_failure(e);
                            return;
                        }
                    },
                };

                self.render_texture(texture, &draw.data);
            }
        }

        self.canvas.present();
    }
}
