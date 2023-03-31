use log::{error, info};
use speed_daemon::{
    errors::SpeedDaemonError,
    message::InboundMessageType,
    state::Db,
    types::{Plate, PlateRoadStruct, Timestamp, TimestampCameraStruct},
};
use tokio::sync::mpsc;

use std::net::SocketAddr;

pub async fn handle_plate(
    client_addr: &SocketAddr,
    new_plate: Plate,
    new_timestamp: Timestamp,
    plate_tx: mpsc::Sender<PlateRoadStruct>,
    shared_db: Db,
) -> anyhow::Result<(), SpeedDaemonError> {
    // Get the camera that reported this plate
    if let Some(current_camera) = shared_db.get_current_camera(client_addr).await {
        // Init an empty struct
        let mut new_plate_road = PlateRoadStruct::new(String::from(""), 0);

        // deconstruct the IAmCamera message to get the plate+road combo
        if let InboundMessageType::IAmCamera {
            road,
            mile: _,
            limit: _,
        } = current_camera
        {
            new_plate_road = PlateRoadStruct {
                plate: new_plate,
                road,
            };
        };

        let new_ts_camera = TimestampCameraStruct {
            timestamp: new_timestamp,
            camera: current_camera,
        };

        // info!("Adding {:?} {:?}", new_plate_road, new_ts_camera);

        // add the newly observed plate:road combo to the shared db
        shared_db
            .add_plate_road_timestamp_camera(new_plate_road.clone(), new_ts_camera)
            .await;

        // send it off to the ticket_manager for processing
        plate_tx
            .send(new_plate_road)
            .await
            .expect("Unable to send new plate");
    } else {
        // It wasn't a camera that reported the plate!
        error!("Plate message did not come from camera {client_addr}");
        return Err(SpeedDaemonError::WrongMessageClient);
    }

    Ok(())
}
