use std::net::SocketAddr;

use log::error;
use speed_daemon::{
    errors::SpeedDaemonError, message::OutboundMessageType, state::Db, types::Road,
};
use tokio::sync::mpsc;

pub async fn handle_i_am_dispatcher(
    roads: Vec<Road>,
    client_addr: &SocketAddr,
    tx: &mpsc::Sender<OutboundMessageType>,
    mut shared_db: Db,
) -> anyhow::Result<(), SpeedDaemonError> {
    // info!("Adding a dispatcher for roads {:?}", roads);

    for road in roads.iter() {
        if let Some(existing_dispatcher) = shared_db.get_ticket_dispatcher(road).await {
            error!(
                "{:?} already is a dispatcher {:?}",
                client_addr, existing_dispatcher
            );
            return Err(SpeedDaemonError::DuplicateClient);
        } else {
            // for every road this dispatcher is responsible for, add the corresponding tx reference
            // info!("Adding dispatcher {} for road {}", client_addr, road);
            let tx = tx.clone();
            shared_db
                .add_ticket_dispatcher(*road, *client_addr, tx)
                .await
        }
    }
    Ok(())
}
