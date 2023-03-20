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
    let interval = interval as f32 / 10.0 * 1000.0;
    tokio::spawn(async move {
        loop {
            match tx.send(OutboundMessageType::Heartbeat).await {
                Ok(_) => {}
                Err(e) => {
                    error!("Unable to send heartbeat, client {} disconnected", e);
                    return;
                }
            }

            sleep(Duration::from_millis(interval as u64)).await;
        }
    });

    Ok(())
}
