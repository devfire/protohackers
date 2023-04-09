use std::net::SocketAddr;

use bytes::BytesMut;
use line_reversal::{
    codec::MessageCodec,
    errors::LRCPError,
    message::MessageType,
    state::Db,
    types::{Session, SessionPosDataStruct},
};
use tokio_util::udp::UdpFramed;

pub async fn handle_connect(
    session: Session,
    addr: SocketAddr,
    shared_db: Db,
) -> anyhow::Result<(), LRCPError> {
    let data = BytesMut::new();
    let session_pos_data = SessionPosDataStruct::new(0, 0, data);
    shared_db.add_session(addr, session_pos_data).await;

    // Send /ack/SESSION/0/ to let the client know that the session is open
    // NOTE: we always do this even if it is a duplicate connect,
    // because the first ack may have been dropped.

    let ack_msg = MessageType::Ack { session, length: 0 };

    let destination_frame = UdpFramed::new(addr, MessageCodec::new());

    Ok(())
}
