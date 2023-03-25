use log::{error, info};
// use speed_daemon::errors::SpeedDaemonError;
use speed_daemon::message::OutboundMessageType;
use tokio::sync::mpsc;
use tokio::time;
// use tokio::time::sleep;
use tokio::time::Duration;

pub fn handle_want_hearbeat(
    interval: u32,
    tx: mpsc::Sender<OutboundMessageType>,
) -> anyhow::Result<()> {
    let interval = interval as f32 / 10.0;
    let mut tick_interval = time::interval(Duration::from_secs_f32(interval));
    info!("Setting tick interval to {}", interval);
    tokio::task::spawn(async move {
        loop {
            match tx.send(OutboundMessageType::Heartbeat).await {
                Ok(_) => {}
                Err(e) => {
                    error!("Unable to send heartbeat, client {} disconnected", e);
                    return;
                }
            }

            // sleep(Duration::from_millis(interval as u64)).await;
            

            tick_interval.tick().await;
        }
    });

    Ok(())
}
