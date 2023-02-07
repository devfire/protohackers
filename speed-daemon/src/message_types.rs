#[derive(Copy, Clone, Debug)]
pub enum MessageType{
    Error(u8),
    Plate(u8),
    Ticket(u8),
    WantHeartbeat(u8),
    Heartbeat(u8),
    IAmCamera(u8),
    IAmDispatcher(u8),
}

struct Message(u8);

impl Message {
    const Error: Self = Self(0x10);
    const Plate: Self = Self(0x20);
}

