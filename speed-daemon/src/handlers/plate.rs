use log::info;
use speed_daemon::{
    message::{InboundMessageType, OutboundMessageType},
    state::Db,
    types::{Plate, PlateRoadStruct, PlateTimestamp, Timestamp, TimestampCameraStruct},
};
use tokio::{sync::mpsc, task};

use std::net::SocketAddr;

pub async fn handle_plate(
    client_addr: &SocketAddr,
    new_plate: Plate,
    new_timestamp: Timestamp,
    ticket_tx: mpsc::Sender<OutboundMessageType>,
    shared_db: Db,
) -> anyhow::Result<()> {
    let current_camera = shared_db.get_current_camera(client_addr);

    if let InboundMessageType::IAmCamera { road, mile, limit } = current_camera {
        let new_plate_road = PlateRoadStruct {
            road,
            plate: new_plate,
        };

        let new_ts_camera = TimestampCameraStruct {
            timestamp: new_timestamp,
            camera: current_camera,
        };

        shared_db.add_plate_road_timestamp_camera(new_plate_road, new_ts_camera);
    };

    task::spawn_blocking(move || {
        // At this point, current_camera contains the InboundMessageType::IAmCamera enum with the current tokio task values

        if let Some(tickets_vec) = shared_db.get_tickets_for_plate(&new_plate) {
            for ticket in tickets_vec.iter() {
                // info!(
                //     "Plate handler forwarding ticket {:?} to ticket manager",
                //     ticket
                // );

                // Send the ticket to the ticket dispatcher
                ticket_tx
                    .blocking_send(ticket.clone())
                    .expect("Unable to send ticket");
            }
        }
    });

    // info!(
    //     "From {}: adding plate-timestamp struct {:?} from camera {:?}",
    //     client_addr, new_plate_ts, current_camera
    // );

    Ok(())
}
