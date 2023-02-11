use crate::errors::SpeedDaemonError;

#[derive(Copy, Clone, Debug)]
pub enum MessageType{
    Plate,
    Ticket,
    WantHeartbeat,
    Heartbeat,
    IAmCamera,
    IAmDispatcher,
}

impl MessageType {
    pub fn get_message_type(msg_id: u8) -> Result<Self, SpeedDaemonError> {
        match msg_id {
            0x20 => Ok(Self::Plate),
            0x21 => Ok(Self::Ticket),
            0x40 => Ok(Self::WantHeartbeat),
            0x41 => Ok(Self::Heartbeat),
            0x80 => Ok(Self::IAmCamera),
            0x81 => Ok(Self::IAmDispatcher),
            _ => Err(SpeedDaemonError::InvalidMessage),
        }
    }
}
