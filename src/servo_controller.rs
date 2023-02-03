/// this module provides a wrapper around a controller module that can
/// manupulate various number of servos.
/// note: the ServoController does not control a single servo, but rather
/// expects a single module that can control multiple entries.
use crate::can::{CANHandle, CANMessage};
use crate::ControllerInput;

use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ServoState {
    pub id: u8,
    pub angle: u8,
    // each servo should have its own "up"/"down"
    pub up_pressed: bool,
    pub down_pressed: bool,
}

impl ServoState {
    pub fn new(id: u8) -> Self {
        Self {
            id, angle: 0,
            up_pressed: false,
            down_pressed: false,
        }
    }
}

#[derive(Debug)]
pub enum ControllerError {
    InvalidMessage,
}

pub struct ServoController {
    handle: Box<dyn CANHandle>,
    servos: Vec<ServoState>,
}

impl ServoController {
    pub fn new(handle: Box<dyn CANHandle>, servo_count: u32) -> Self {
        // todo: got to be a one liner for this.
        let mut servos = vec![];
        for i in 0..servo_count {
            servos.push(ServoState::new(i as u8));
        }

        Self { handle, servos }
    }

    pub fn handle_command(&mut self, command: ControllerInput) -> Result<(), ControllerError> {

        self.servos[0].up_pressed = command.up;
        self.servos[0].down_pressed = command.down;

        self.servos[1].up_pressed = command.left;
        self.servos[1].down_pressed = command.right;

        Ok(())
    }

    //
    pub fn update(&mut self) -> Result<(), ControllerError> {
        // read in can message and handle incoming can messages.
        if let Ok(Some(msg)) = self.handle.read() {
            if msg.dlc < 1 {
                println!("Message must be of atleast length 1");
                return Err(ControllerError::InvalidMessage);
            }

            let servo_base = 0x200;
            for (index, i) in self.servos.iter_mut().enumerate() {
                println!("Servo entry: {}", servo_base + index as u32);
                if msg.id == servo_base + index as u32 {
                    println!("Setting servo: {}", index);
                    i.angle = msg.data[0];
                    break;
                }
            }
        }

        let mut servo_command: u16 = 0x0;
        // send out servo input values
        if self.servos[0].up_pressed {
            servo_command |= 0x1;
        } else if self.servos[0].down_pressed {
            servo_command |= 0x2;
        }

        if self.servos[1].up_pressed {
            servo_command |= 0x4;
        } else if self.servos[1].down_pressed {
            servo_command |= 0x8;
        }
            
        self.handle.write(&CANMessage::new(0x200, &servo_command.to_le_bytes(), false));

        Ok(())
    }

    pub fn get_servo_state<'a>(&'a self) -> &'a Vec<ServoState> {
        &self.servos
    }
}
