use std::net::SocketAddr;

// use log::info;
use speed_daemon::{message::OutboundMessageType, state::Db, types::Road};
use tokio::sync::mpsc;

pub async fn handle_i_am_dispatcher(
    roads: Vec<Road>,
    client_addr: &SocketAddr,
    tx: &mpsc::Sender<OutboundMessageType>,
    mut shared_db: Db,
) -> anyhow::Result<()> {
    // info!("Adding a dispatcher for roads {:?}", roads);


    for road in roads.iter() {
        // for every road this dispatcher is responsible for, add the corresponding tx reference
        // info!("Adding dispatcher {} for road {}", client_addr, road);
        let tx = tx.clone();
        shared_db.add_ticket_dispatcher(*road, *client_addr, tx)
    }
    Ok(())
}
