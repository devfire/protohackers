use log::{error, info};
// use speed_daemon::errors::SpeedDaemonError;
use speed_daemon::message::OutboundMessageType;
use tokio::sync::mpsc;
// use tokio::time;
use tokio::time::sleep;
use tokio::time::Duration;

pub async fn handle_want_hearbeat(interval: u32, tx: mpsc::Sender<OutboundMessageType>) {
    let interval = interval as f32 / 10.0;
    // let mut tick_interval = time::interval(Duration::from_secs_f32(interval));
    info!("Setting tick interval to {}", interval);
    loop {
        let tx = tx.clone();
        tokio::task::spawn_blocking(move || {
            // match tx.send(OutboundMessageType::Heartbeat) {
            //     Ok(_) => {}
            //     Err(e) => {
            //         error!("Unable to send heartbeat, client {} disconnected", e);
            //         return;
            //     }
            // }

            tx.blocking_send(OutboundMessageType::Heartbeat)
                .expect("Unable to send a heartbeat.");

            // tick_interval.tick();
        });
        sleep(Duration::from_secs_f32(interval)).await;
    }
}
