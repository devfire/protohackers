use log::info;
use speed_daemon::{
    message::{InboundMessageType, OutboundMessageType},
    state::Db,
    types::{Mile, Plate, Road, Timestamp},
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
    // Get the current road speed limit
    let mut speed_limit: u16 = 0;
    let mut observed_mile_marker: Mile = 0;
    let mut current_road: Road = 0;

    // At this point, current_camera contains the InboundMessageType::IAmCamera enum with the current tokio task values
    // let new_camera = shared_db.current_camera.clone();
    let current_camera = shared_db.get_current_camera(client_addr);

    // Get the details of the camera that obseved this plate.
    // NOTE: this came from handle_i_am_camera
    if let InboundMessageType::IAmCamera { road, mile, limit } = current_camera {
        current_road = road;
        observed_mile_marker = mile;
        speed_limit = limit;
    }

    info!(
        "Speed limit is: {} mile marker: {}",
        current_road, observed_mile_marker
    );

    let mut mile1: u16 = 0;
    let mut mile2: u16 = 0;
    let mut timestamp1: u32 = 0;
    let mut timestamp2: u32 = 0;
    // Check if this plate has been observed before
    if let Some(previously_seen_camera) = shared_db.check_camera_plate(&new_plate) {
        let time_traveled: u32;
        let mut distance_traveled: u16 = 0;
        // Messages may arrive out of order, so we need to figure out what to subtract from what.
        // NOTE: previously_seen_camera is a (timestamp, InboundMessageType::IAmCamera) tuple,
        // so 0th entry refers to the timestamp.
        if new_timestamp > previously_seen_camera.0 {
            time_traveled = new_timestamp - previously_seen_camera.0;
            if let InboundMessageType::IAmCamera {
                road: _,
                mile,
                limit: _,
            } = previously_seen_camera.1
            {
                distance_traveled = observed_mile_marker - mile;
                mile1 = mile;
                mile2 = observed_mile_marker;
                timestamp1 = previously_seen_camera.0;
                timestamp2 = new_timestamp;
            }
        } else {
            time_traveled = previously_seen_camera.0 - new_timestamp;
            if let InboundMessageType::IAmCamera {
                road: _,
                mile,
                limit: _,
            } = previously_seen_camera.1
            {
                distance_traveled = mile - observed_mile_marker;
                mile1 = observed_mile_marker;
                mile2 = mile;
                timestamp1 = new_timestamp;
                timestamp2 = previously_seen_camera.0;
            }
        }

        let observed_speed: f64 = distance_traveled as f64 / time_traveled as f64 * 3600.0;
        info!(
            "Plate: {} seen by camera: {:?} distance traveled: {} in time: {} speed: {}",
            new_plate, previously_seen_camera, distance_traveled, time_traveled, observed_speed
        );

        // check if the car exceeded the speed limit
        if observed_speed > speed_limit as f64 {
            info!(
                "Plate {} exceeded the speed limit, issuing ticket",
                new_plate
            );
            let new_ticket = OutboundMessageType::Ticket {
                plate: new_plate,
                road: current_road,
                mile1,
                timestamp1,
                mile2,
                timestamp2,
                speed: observed_speed as u16,
            };

            // let tx = tx.clone();
            info!(
                "Plate handler forwarding ticket {:?} to ticket manager",
                new_ticket
            );
            ticket_tx.send(new_ticket).await?;
            // shared_db.add_ticket(new_ticket);
            //issue_ticket(new_ticket, tx);
        }
    } else {
        // Add the newly observed plate to the shared db of plate -> camera hash
        // NOTE: subsequent inserts will override the value because the plate key is the same.
        // But that's OK since we only ever need the last two values.
        shared_db.add_camera_plate(new_plate, new_timestamp, current_camera);
    }
    Ok(())
}
