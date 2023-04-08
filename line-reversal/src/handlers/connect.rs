use std::net::SocketAddr;

use line_reversal::{errors::LRCPError, state::Db, types::Session};

pub fn handle_connect(
    session: Session,
    client_addr: &SocketAddr,
    shared_db: Db,
) -> anyhow::Result<(), LRCPError> {

    
    Ok(())
}
