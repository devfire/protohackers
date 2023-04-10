use std::net::SocketAddr;

// use bytes::BytesMut;
use line_reversal::{
    errors::LRCPError,
    state::Db,
    types::{Session, SessionPosDataStruct},
};

pub async fn handle_connect(
    session: Session,
    addr: SocketAddr,
    shared_db: Db,
) -> anyhow::Result<(), LRCPError> {
    let session_pos_data = SessionPosDataStruct::new(session, 0, "".to_string());
    shared_db.add_session(addr, session_pos_data).await;

    Ok(())
}
