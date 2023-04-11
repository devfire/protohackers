use std::net::SocketAddr;

use line_reversal::{
    errors::LRCPError, message::MessageType, state::Db, types::SessionPosDataStruct,
};
use log::{error, info};
use nom::{
    branch::alt,
    bytes::complete::{escaped_transform, tag},
    character::complete::alpha1,
    combinator::value,
    IResult,
};
use tokio::sync::mpsc::Sender;

fn parse_data_string(input: &str) -> IResult<&str, String> {
    escaped_transform(
        alpha1,
        '\\',
        alt((
            value("\\", tag("\\")),
            value("\"", tag("\"")),
            value("\n", tag("n")),
        )),
    )(input)
}

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

    // Session ok let's unescape the characters
    let data_string = parse_data_string(&session_pos_data.data);

    
    Ok(())
}
