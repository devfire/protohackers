use std::net::SocketAddr;

use bytes::BytesMut;
use line_reversal::{
    errors::LRCPError,
    state::Db,
    types::{PosDataStruct, Session},
};

pub async fn handle_connect(
    session: Session,
    addr: SocketAddr,
    shared_db: Db,
) -> anyhow::Result<(), LRCPError> {
    let data = BytesMut::new();
    let pos_data = PosDataStruct::new(0, data);
    shared_db.add_session(addr, session, pos_data).await;
    Ok(())
}
