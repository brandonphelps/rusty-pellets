


#[derive(Debug)]
pub enum CANError {
    ComErr,
}

#[derive(Debug)]
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

        CANMessage {
            id,
            data: d,
            dlc: if data.len() >= 8 { 8 } else { data.len() as u8 },
            is_extended,
        }
    }
}

pub trait CANHandle: Send + Sync {
    // fn open(dev: i32) -> Result<Self, CANError>;
    // non blocking write
    fn write(&self, msg: &CANMessage) -> Result<(), CANError>;

    // non blocking read
    fn read(&self) -> Result<Option<CANMessage>, CANError>;
}

#[cfg(target_os = "windows")]
pub use win::*;

pub use mock::*;

#[cfg(not(target_os = "window"))]
pub mod mock {
    use super::*;

    pub struct MockHandle {}

    impl MockHandle {
        pub fn open(_dev: i32) -> Result<Self, CANError> {
            Ok(Self {})
        }
    }

    impl CANHandle for MockHandle {
        fn write(&self, _msg: &CANMessage) -> Result<(), CANError> {
            // todo: maybe store messages internally so that they can be inspected later?
            Ok(())
        }

        fn read(&self) -> Result<Option<CANMessage>, CANError> {
            Ok(Some(CANMessage::new(0x200, &[1, 2, 3, 4], false)))
        }
    }
}

#[cfg(target_os = "windows")]
pub mod win {

    use super::*;

    use kvaser_sys::CANHandle as KVHandle;
    /// Contains a wrapper around CAN communication
    use kvaser_sys::{
        can_bus_on, can_initialize_library, can_open_channel, can_read, can_write, CANStatus,
    };

    // generic CANHandle helper
    pub struct WindowsCANHandle {
        handle: KVHandle,
    }

    impl WindowsCANHandle {
        pub fn open(dev: i32) -> Result<Self, CANError> {
            // it is safe to call this multiple times.
            can_initialize_library();
            let handle = can_open_channel(dev, 0x20).unwrap();
            can_bus_on(handle).unwrap();
            Ok(Self { handle })
        }
    }

    impl CANHandle for WindowsCANHandle {
        // non blocking write.
        fn write(&self, msg: &CANMessage) -> Result<(), CANError> {
            can_write(
                self.handle,
                msg.id,
                &msg.data,
                msg.dlc,
                if msg.is_extended { 0x4 } else { 0x2 },
            );

            Ok(())
        }

        // non block read.
        fn read(&self) -> Result<Option<CANMessage>, CANError> {
            match can_read(self.handle) {
                Ok(msg_info) => {
                    let data = msg_info.1;
                    let _flags = msg_info.3;

                    // todo: do something with flags.
                    Ok(Some(CANMessage::new(msg_info.0, &data, false)))
                }
                Err(CANStatus::CanERR_NOMSG) => Ok(None),
                Err(e) => {
                    println!("unknown can error: {:?}", e);
                    Err(CANError::ComErr)
                }
            }
        }
    }
}
