use log::info;

use speed_daemon::{message::InboundMessageType, state::Db};
use std::net::SocketAddr;

pub async fn handle_i_am_camera(
    client_addr: &SocketAddr,
    new_camera: InboundMessageType,
    mut shared_db: Db,
) -> anyhow::Result<()> {
    // info!("Adding camera: {:?} to client {}", new_camera, client_addr);

    shared_db.add_camera(*client_addr, new_camera);

    Ok(())
}
