/// Contains a wrapper around CAN communication

use kvaser_sys::{can_initialize_library, can_open_channel, can_write, can_bus_on};
use kvaser_sys::CANHandle as KVHandle;

#[derive(Debug)]
pub enum CANError {

}

pub struct CANMessage {
    pub id: u32,
    pub data: [u8; 8],
    pub dlc: u8,
    pub is_extended: bool,
}

impl CANMessage {
    /// if data is greater than 8 bytes, then only the first 8 bytes will
    /// be used. 
    pub fn new(id: u32, data: &[u8], is_extended: bool) -> Self {
        let mut d = [0u8; 8];
        if data.len() > 8 {
            d.copy_from_slice(&data[..8]);
        } else {
            for (index, i) in data.iter().enumerate() {
                d[index as usize] = *i;
            }
        }
        println!("new message: {:?}", d);
            
        CANMessage {
            id,
            data: d,
            dlc: if data.len() >= 8 {
                8
            } else {
                data.len() as u8
            },
            is_extended,
        }
    }
}

// generic CANHandle helper
pub struct CANHandle {
    handle: KVHandle,
}

impl CANHandle {
    pub fn open(dev: i32) -> Result<Self, CANError> {
        // it is safe to call this multiple times. 
        can_initialize_library();
        let handle = can_open_channel(dev, 0x20).unwrap();
        can_bus_on(handle).unwrap();
        Ok(Self {
            handle
        })
    }

    pub fn write(&self, msg: &CANMessage) -> Result<(), CANError> {
        can_write(self.handle, msg.id, &msg.data, msg.dlc, if msg.is_extended {
            0x4
        } else {
            0x2
        });

        Ok(())
    }
}
    


