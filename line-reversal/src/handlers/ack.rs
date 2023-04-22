use line_reversal::{errors::LRCPError, message::MessageType, state::Db, types::{Session, Pos}};
use log::{error, info};
use std::net::SocketAddr;
use tokio::sync::mpsc::Sender;

pub async fn handle_ack(
    session: Session,
    client_addr: &SocketAddr,
    tx: Sender<(MessageType, SocketAddr)>,
    shared_db: Db,
) -> anyhow::Result<(), anyhow::Error> {
    // let old_pos: Pos = 0;
    
    if let Some(session) = shared_db.get_session(client_addr).await {
        info!("found {session}, proceeding ");
        if let Some(old_pos) = shared_db.get_pos(client_addr).await {
            info!("Found old pos {old_pos}");
        }
    } else {
        // uh-oh, you are sending acks but we've never seen this session before, bail.
        error!("{}", LRCPError::SessionNotFound);
        let no_session_reply = MessageType::Close { session };
        tx.send((no_session_reply, *client_addr)).await?;
        return Err(LRCPError::SessionNotFound.into());
    }

    

    Ok(())
}
