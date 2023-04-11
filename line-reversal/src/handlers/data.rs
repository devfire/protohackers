use std::net::SocketAddr;

use line_reversal::{errors::LRCPError, state::Db, types::SessionPosDataStruct};
use log::{error, info};

pub async fn handle_data(
    session_pos_data: SessionPosDataStruct,
    addr: &SocketAddr,
    shared_db: Db,
) -> anyhow::Result<(), LRCPError> {
    // let's first see if the session has been established previously
    if let Some(session) = shared_db.get_session(addr).await {
        info!("found {session}, proceeding ");
    } else {
        error!("Error: {}", LRCPError::SessionNotFound);
        return Err(LRCPError::SessionNotFound);
    }

    Ok(())
}
