use std::net::SocketAddr;

use line_reversal::{
    errors::LRCPError,
    message::MessageType,
    state::Db,
    types::{Pos, SessionPosDataStruct},
};
use log::{error, info, warn};

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

/// When you receive a data message
/// If the session is not open: send /close/SESSION/ and stop.
/// If you've already received everything up to POS: unescape "\\" and "\/",
/// find the total LENGTH of unescaped data that you've already received (including the data in this message, if any),
/// send /ack/SESSION/LENGTH/, and pass on the new data (if any) to the application layer.
/// If you have not received everything up to POS: send a duplicate of your previous ack (or /ack/SESSION/0/ if none),
/// saying how much you have received, to provoke the other side to retransmit whatever you're missing.
pub async fn handle_data(
    session_pos_data: &SessionPosDataStruct,
    addr: &SocketAddr,
    tx: Sender<(MessageType, SocketAddr)>,
    shared_db: Db,
) -> anyhow::Result<String, anyhow::Error> {
    // let's first see if the session has been established previously
    if let Some(old_session) = shared_db.get_session(addr).await {
        if let Some(old_pos) = shared_db.get_pos(addr).await {
            info!("Found existing session {old_session} existing pos {old_pos}");

            // Session ok let's unescape the characters
            let data_string = unescape_string(&session_pos_data.data);
            let data_string_length = data_string.len();

            let new_pos: Pos = session_pos_data.pos;

            // Behaviour is undefined if a peer sends payload data that overlaps with payload data you've already received, but differs from it.
            // So we just overwrite the previous value and move on.
            if new_pos - 1 > old_pos {
                warn!("New pos {new_pos} is greater than old pos {old_pos}, missing data.");
                //  Ok(data_string.chars().rev().collect::<String>())
                // uh oh missing acks or data

                let missing_data_msg = MessageType::Ack {
                    session: session_pos_data.session,
                    length: old_pos,
                };
                
                tx.send((missing_data_msg, *addr)).await?;
                // return Err(LRCPError::DataMissing.into())
            } else {
                let new_session = SessionPosDataStruct {
                    session: session_pos_data.session,
                    pos: data_string_length as Pos,
                    data: data_string.clone(),
                };

                shared_db.add_session(*addr, new_session).await;
            }
            Ok(data_string)
        } else {
            error!("Session found but no pos located--very bad!");
            Err(LRCPError::SessionNotFound.into())
        }
    } else {
        // uh-oh, you are sending data but we've never seen this session before, bail.
        error!("{}", LRCPError::SessionNotFound);
        let no_session_reply = MessageType::Close { session: session_pos_data.session };
        tx.send((no_session_reply, *addr)).await?;
        Err(LRCPError::SessionNotFound.into())
    }
}
