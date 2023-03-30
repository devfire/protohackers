use log::{error, info};

use speed_daemon::{errors::SpeedDaemonError, message::InboundMessageType, state::Db};
use std::net::SocketAddr;

pub async fn handle_i_am_camera(
    client_addr: &SocketAddr,
    new_camera: InboundMessageType,
    mut shared_db: Db,
) -> anyhow::Result<(), SpeedDaemonError> {
    info!("Adding camera: {:?} to client {}", new_camera, client_addr);

    if let Some(existing_camera) = shared_db.get_current_camera(client_addr).await {
        error!(
            "{:?} sent a duplicate camera message {:?}",
            client_addr, existing_camera
        );
        return Err(SpeedDaemonError::DuplicateCamera);
    } else {
        shared_db.add_camera(*client_addr, new_camera).await;
    }

    Ok(())
}
