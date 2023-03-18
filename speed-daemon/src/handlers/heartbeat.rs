use log::error;
// use speed_daemon::errors::SpeedDaemonError;
use speed_daemon::message::OutboundMessageType;
use tokio::sync::mpsc;
use tokio::time::sleep;
use tokio::time::Duration;

pub fn handle_want_hearbeat(
    interval: u32,
    tx: mpsc::Sender<OutboundMessageType>,
) -> anyhow::Result<()> {
    let interval = interval / 10;
    tokio::spawn(async move {
        loop {
            match tx.send(OutboundMessageType::Heartbeat).await {
                Ok(_) => {}
                Err(e) => {
                    error!("Client disconnected: {}", e);
                    return;
                }
            }
            // .expect("Unable to send heartbeat");

            sleep(Duration::from_secs(interval as u64)).await;
        }
    });

    Ok(())
}
