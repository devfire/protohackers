use std::net::SocketAddr;

use line_reversal::{
    errors::LRCPError, message::MessageType, state::Db, types::SessionPosDataStruct,
};
use log::{error, info};

use tokio::sync::mpsc::Sender;

fn unescape_string(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => result.push('\n'),
                Some('r') => result.push('\r'),
                Some('t') => result.push('\t'),
                Some('\\') => result.push('\\'),
                Some('"') => result.push('"'),
                Some('\'') => result.push('\''),
                Some(c) => {
                    result.push('\\');
                    result.push(c);
                }
                None => result.push('\\'),
            }
        } else {
            result.push(c);
        }
    }
    result
}

pub async fn handle_data(
    session_pos_data: SessionPosDataStruct,
    addr: &SocketAddr,
    tx: Sender<(MessageType, SocketAddr)>,
    shared_db: Db,
) -> anyhow::Result<(), anyhow::Error> {
    // let's first see if the session has been established previously
    let session = session_pos_data.session;
    let pos = session_pos_data.pos;

    if let Some(session) = shared_db.get_session(addr).await {
        info!("found {session}, proceeding ");
    } else {
        error!("{}", LRCPError::SessionNotFound);
        let no_session_reply = MessageType::Close { session };
        tx.send((no_session_reply, *addr)).await?;
    }

    

    // Session ok let's unescape the characters
    let data_string = unescape_string(&session_pos_data.data);

    let data_string_length = data_string.len();
    
    Ok(())
}
