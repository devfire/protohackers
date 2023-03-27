use log::info;
use speed_daemon::{
    message::{InboundMessageType, OutboundMessageType},
    state::Db,
    types::{Plate, PlateRoadStruct, Timestamp, TimestampCameraStruct},
};
use tokio::sync::mpsc;

use std::net::SocketAddr;

pub async fn handle_plate(
    client_addr: &SocketAddr,
    new_plate: Plate,
    new_timestamp: Timestamp,
    ticket_tx: mpsc::Sender<OutboundMessageType>,
    shared_db: Db,
) -> anyhow::Result<()> {
    // Get the camera that reported this plate
    let current_camera = shared_db.get_current_camera(client_addr).await;

    // Init an empty struct
    let mut new_plate_road = PlateRoadStruct::new(String::from(""), 0);

    if let InboundMessageType::IAmCamera {
        road,
        mile: _,
        limit: _,
    } = current_camera
    {
        new_plate_road = PlateRoadStruct {
            road,
            plate: new_plate,
        };
    };

    let new_ts_camera = TimestampCameraStruct {
        timestamp: new_timestamp,
        camera: current_camera,
    };

    info!("Adding {:?} {:?}", new_plate_road, new_ts_camera);

    shared_db.add_plate_road_timestamp_camera(new_plate_road.clone(), new_ts_camera).await;

    tokio::spawn(async move {
        if let Some(ticket) = shared_db.get_ticket_for_plate(&new_plate_road).await {
            info!(
                "Plate handler forwarding ticket {:?} to ticket manager",
                ticket
            );

            // Send the ticket to the ticket dispatcher
            ticket_tx.send(ticket).await.expect("Unable to send ticket");
        }
    });

    Ok(())
}
