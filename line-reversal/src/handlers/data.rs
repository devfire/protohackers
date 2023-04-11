use std::net::SocketAddr;

use line_reversal::{errors::LRCPError, state::Db, types::SessionPosDataStruct, message::MessageType};
use log::{error, info};
use tokio::sync::mpsc::Sender;

pub async fn handle_data(
    session_pos_data: SessionPosDataStruct,
    addr: &SocketAddr,
    tx: Sender<(MessageType, SocketAddr)>,
    shared_db: Db,
) -> anyhow::Result<(), anyhow::Error> {
    // let's first see if the session has been established previously
    let session = session_pos_data.session;

    if let Some(session) = shared_db.get_session(addr).await {
        info!("found {session}, proceeding ");
    } else {
        error!("{}", LRCPError::SessionNotFound);
        let no_session_reply = MessageType::Close { session };
        tx.send((no_session_reply, *addr)).await?;
    }

    Ok(())
}
