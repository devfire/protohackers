use speed_daemon::{errors::SpeedDaemonError, message::OutboundMessageType};
use tokio::sync::mpsc;

pub fn handle_error(
    error_message: String,
    tx: mpsc::Sender<OutboundMessageType>,
) -> anyhow::Result<(), SpeedDaemonError> {
    tokio::spawn(async move {
        tx.send(OutboundMessageType::Error(error_message))
            .await
            .expect("Unable to send error message");
    });
    Ok(())
}
