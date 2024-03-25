use std::{cell::RefCell, f32::EPSILON, rc::Rc};

use log::{error, warn};
use sdl2::{event::Event, pixels::Color};

use stagehand::{
    app::App,
    draw::DrawType,
    input::{ActionState, ActionType, InputError},
    loading::ResourceError,
    utility::{Update, UpdateInfo, UpdateInstruction},
    StageError,
};

use crate::{
    input::{translate_axis, SDLCommand, SDLGamepadFeature},
    SDLApp,
};

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
                _ => (),
            }
        }

        let keys = events.keyboard_state();
        let mouse = events.mouse_state();

        let mut input = self.input.borrow_mut();
        for command_options in 0..input.commands.len() {
            let mut active = ActionType::Digital(ActionState::Up);

            'commands: for command in input.commands[command_options].commands.iter() {
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
                    SDLCommand::Gamepad(feature, controller) => match controller {
                        Some(index) => {
                            let controller = &self.controllers[*index];

                            match feature {
                                SDLGamepadFeature::Button(buttons) => {
                                    for button in buttons.iter() {
                                        if !controller.button(*button) {
                                            continue;
                                        }
                                    }
                                    active = ActionType::Digital(ActionState::Down);
                                    break 'commands;
                                }
                                SDLGamepadFeature::Axis(axis) => {
                                    let value = translate_axis(controller.axis(*axis));
                                    if value.abs() >= EPSILON {
                                        active = ActionType::Axis(value);
                                        break 'commands;
                                    }
                                }
                                SDLGamepadFeature::Stick(x, y) => {
                                    let (x, y) = (
                                        translate_axis(controller.axis(*x)),
                                        translate_axis(controller.axis(*y)),
                                    );
                                    if x.abs() >= EPSILON || y.abs() >= EPSILON {
                                        active = ActionType::Analog { x, y };
                                        break 'commands;
                                    }
                                }
                            };
                        }
                        None => {
                            'controller: for controller in self.controllers.iter() {
                                match feature {
                                    SDLGamepadFeature::Button(buttons) => {
                                        for button in buttons.iter() {
                                            if !controller.button(*button) {
                                                continue 'controller;
                                            }
                                        }
                                        active = ActionType::Digital(ActionState::Down);
                                        break 'commands;
                                    }
                                    SDLGamepadFeature::Axis(axis) => {
                                        let value = translate_axis(controller.axis(*axis));
                                        if value.abs() >= 0.1 {
                                            active = ActionType::Axis(value);
                                            break 'commands;
                                        }
                                    }
                                    SDLGamepadFeature::Stick(x, y) => {
                                        let (a_x, a_y) = (
                                            translate_axis(controller.axis(*x)),
                                            translate_axis(controller.axis(*y)),
                                        );
                                        if a_x.abs() >= 0.1 || a_y.abs() >= 0.1 {
                                            active = ActionType::Analog { x: a_x, y: a_y };
                                            break 'commands;
                                        }
                                    }
                                };
                            }
                        }
                    },
                };
            }

            let user_index = input.commands[command_options].user_index;
            let action_index = input.commands[command_options].action_index;

            match input.users[user_index].update_action(action_index, active) {
                Err(e) => match e {
                    InputError::ActionIndexOutOfBounds => {
                        error!("Action index not found: {}", action_index)
                    }
                    _ => {}
                },
                _ => {}
            };
        }

        input.set();

        Ok(true)
    }

    fn update(&mut self, delta: f64) {
        self.prepare_info();

        {
            let update = Update::new(
                self.input.clone(),
                self.info.clone(),
                self.u_content.clone(),
            );

            match self.stage.update(&update, delta) {
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

        self.input.borrow_mut().updated();
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
                    DrawType::Texture => {
                        match self.storage.borrow().textures.get_by_ticket(draw.ticket) {
                            Ok(t) => t,
                            Err(e) => {
                                ResourceError::log_failure(e);
                                return;
                            }
                        }
                    }
                    DrawType::Text(s, c) => {
                        match self.storage.borrow().fonts.get_by_ticket(draw.ticket) {
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
                        }
                    }
                };

                self.render_texture(texture, &draw.data);
            }
        }

        self.canvas.present();
    }
}
