
use log::info;

use speed_daemon::state::SharedState;

use std::sync::Mutex;

use std::sync::Arc;

use speed_daemon::message::InboundMessageType;

use std::net::SocketAddr;

pub fn handle_i_am_camera(
    client_addr: &SocketAddr,
    new_camera: InboundMessageType,
    shared_db: Arc<Mutex<SharedState>>,
) -> anyhow::Result<()> {
    info!("Current camera: {:?}", new_camera);

    // Set the current tokio thread camera so we can look up its details later
    let mut shared_db = shared_db
        .lock()
        .expect("Unable to lock shared db in handle_i_am_camera");

    shared_db.add_camera(*client_addr, new_camera);

    Ok(())
}
