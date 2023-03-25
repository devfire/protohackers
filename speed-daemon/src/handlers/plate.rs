use log::info;
use speed_daemon::{
    message::OutboundMessageType,
    state::Db,
    types::{Plate, PlateTimestamp, Timestamp},
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

    let new_plate_ts = PlateTimestamp {
        plate: new_plate.clone(),
        timestamp: new_timestamp,
    };

    shared_db.add_plate_timestamp_camera(new_plate_ts, current_camera);
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

        // tx.blocking_send(OutboundMessageType::Heartbeat)
        //     .expect("Unable to send heartbeat");

        // // sleep(Duration::from_millis(interval as u64)).await;
        // // let mut tick_interval = time::interval(Duration::from_secs_f32(interval));
        // std::thread::sleep(Duration::from_secs_f32(interval));
    });

    // info!(
    //     "From {}: adding plate-timestamp struct {:?} from camera {:?}",
    //     client_addr, new_plate_ts, current_camera
    // );

    // Let's check if this observation resulted in any tickets.
    // NOTE: Really should only ever get one ticket back but just in case, let's run through the vec

    Ok(())
}
