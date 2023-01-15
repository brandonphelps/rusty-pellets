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
}

impl ServoState {
    pub fn new(id: u8) -> Self {
        Self { id, angle: 0 }
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

    pub fn up(&mut self) -> Result<(), ControllerError> {
        println!("up");
        let _response = self.handle.write(&CANMessage::new(0x200, &[0x0], false));
        Ok(())
    }

    pub fn down(&mut self) -> Result<(), ControllerError> {
        println!("down");
        let _response = self.handle.write(&CANMessage::new(0x200, &[0x3], false));
        Ok(())
    }
    pub fn left(&mut self) -> Result<(), ControllerError> {
        println!("left");
        let _response = self.handle.write(&CANMessage::new(0x200, &[0x2], false));
        Ok(())
    }

    pub fn right(&mut self) -> Result<(), ControllerError> {
        println!("right");
        let _response = self.handle.write(&CANMessage::new(0x200, &[0x1], false));
        Ok(())
    }

    pub fn handle_command(&mut self, command: ControllerInput) -> Result<(), ControllerError> {
        match command {
            ControllerInput::Left => self.left(),
            ControllerInput::Right => self.right(),
            ControllerInput::Up => self.up(),
            ControllerInput::Down => self.down(),
        }
    }

    //
    pub fn update(&mut self) -> Result<(), ControllerError> {
        // read in can message and handle incoming can messages.
        if let Ok(Some(msg)) = self.handle.read() {
            println!("Got a message: {:?}", msg);

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

        Ok(())
    }

    pub fn get_servo_state<'a>(&'a self) -> &'a Vec<ServoState> {
        &self.servos
    }
}
