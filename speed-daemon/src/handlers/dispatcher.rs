use std::net::SocketAddr;

use log::{error, info};
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
    info!("From {client_addr} adding a dispatcher for roads {roads:?}");

    // make sure this client hasn't identified itself as a camera before
    if let Some(existing_camera) = shared_db.get_current_camera(client_addr).await {
        error!(
            "{client_addr} previously identified as a camera: {:?}",
            existing_camera
        );
        return Err(SpeedDaemonError::DuplicateDispatcher);
    }

    if shared_db
        .ticket_dispatcher_already_exists(client_addr)
        .await
    {
        error!("{client_addr} previously identified as a dispatcher already.");
        return Err(SpeedDaemonError::DuplicateDispatcher);
    }

    for road in roads.iter() {
        // for every road this dispatcher is responsible for, add the corresponding tx reference
        // info!("Adding dispatcher {} for road {}", client_addr, road);
        let tx = tx.clone();
        shared_db
            .add_ticket_dispatcher(*road, *client_addr, tx)
            .await
    }
    Ok(())
}
