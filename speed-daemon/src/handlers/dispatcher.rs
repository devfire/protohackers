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

    // make sure this client hasn't identified itself as a camera before
    if let Some(existing_camera) = shared_db.get_current_camera(client_addr).await {
        error!(
            "{:?} sent a duplicate camera message {:?}",
            client_addr, existing_camera
        );
        return Err(SpeedDaemonError::DuplicateCamera);
    }

    for road in roads.iter() {
        if shared_db.ticket_dispatcher_already_exists(road, client_addr).await {
            error!("Road {road} already has a dispatcher for {client_addr}");
            return Err(SpeedDaemonError::DuplicateDispatcher);
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
