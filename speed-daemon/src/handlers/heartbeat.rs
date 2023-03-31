use speed_daemon::message::OutboundMessageType;
use tokio::sync::mpsc;
use tokio::task;
// use tokio::time::sleep;
use tokio::time::Duration;

pub async fn handle_want_hearbeat(
    interval: u32,
    tx: mpsc::Sender<OutboundMessageType>,
) -> anyhow::Result<()> {
    let interval = interval as f32 / 10.0;
    task::spawn_blocking(move || {
        loop {
            tx.blocking_send(OutboundMessageType::Heartbeat)
                .expect("Unable to send heartbeat");

            // sleep(Duration::from_millis(interval as u64)).await;
            // let mut tick_interval = time::interval(Duration::from_secs_f32(interval));
            std::thread::sleep(Duration::from_secs_f32(interval));
        }
    });

    Ok(())
}
