use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Mutex};
use tokio::time::interval;

use crate::warp::{WarpClient, WarpInfo, WarpResult};

#[derive(Debug, Clone)]
#[allow(dead_code)] // Future use for async message-based architecture
pub enum WarpMessage {
    Connect,
    Disconnect,
    Refresh,
    CreateRegistration,
    DeleteRegistration,
    StatusUpdate(WarpInfo),
    Error(String),
}

#[allow(dead_code)] // Future use for async message-based architecture
pub struct WarpManager {
    client: WarpClient,
    sender: mpsc::UnboundedSender<WarpMessage>,
    receiver: Arc<Mutex<mpsc::UnboundedReceiver<WarpMessage>>>,
}

impl WarpManager {
    pub fn new() -> Self {
        let client = WarpClient::new();
        let (sender, receiver) = mpsc::unbounded_channel();
        
        Self {
            client,
            sender,
            receiver: Arc::new(Mutex::new(receiver)),
        }
    }

    #[allow(dead_code)] // Future use for async message-based architecture
    pub fn get_sender(&self) -> mpsc::UnboundedSender<WarpMessage> {
        self.sender.clone()
    }

    #[allow(dead_code)] // Future use for async message-based architecture
    pub async fn start_background_tasks(&self) {
        let client = self.client.clone();
        let sender = self.sender.clone();
        
        // Start periodic status updates
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(5));
            
            loop {
                interval.tick().await;
                
                match client.get_status().await {
                    Ok(info) => {
                        let _ = sender.send(WarpMessage::StatusUpdate(info));
                    }
                    Err(e) => {
                        let _ = sender.send(WarpMessage::Error(format!("Status update failed: {}", e)));
                    }
                }
            }
        });
    }

    #[allow(dead_code)] // Future use for async message-based architecture
    pub async fn handle_message(&self, message: WarpMessage) -> WarpResult<()> {
        match message {
            WarpMessage::Connect => {
                self.client.connect().await?;
                // Send status update after connection attempt
                let info = self.client.get_status().await?;
                let _ = self.sender.send(WarpMessage::StatusUpdate(info));
            }
            WarpMessage::Disconnect => {
                self.client.disconnect().await?;
                // Send status update after disconnection attempt
                let info = self.client.get_status().await?;
                let _ = self.sender.send(WarpMessage::StatusUpdate(info));
            }
            WarpMessage::Refresh => {
                let info = self.client.get_status().await?;
                let _ = self.sender.send(WarpMessage::StatusUpdate(info));
            }
            WarpMessage::CreateRegistration => {
                self.client.create_registration().await?;
                let info = self.client.get_status().await?;
                let _ = self.sender.send(WarpMessage::StatusUpdate(info));
            }
            WarpMessage::DeleteRegistration => {
                self.client.delete_registration().await?;
                let info = self.client.get_status().await?;
                let _ = self.sender.send(WarpMessage::StatusUpdate(info));
            }
            WarpMessage::StatusUpdate(_) | WarpMessage::Error(_) => {
                // These are output messages, no action needed
            }
        }
        
        Ok(())
    }

    #[allow(dead_code)] // Future use for async message-based architecture
    pub async fn process_messages(&self) {
        let receiver = self.receiver.clone();
        
        while let Some(message) = {
            let mut recv = receiver.lock().await;
            recv.recv().await
        } {
            if let Err(e) = self.handle_message(message).await {
                let _ = self.sender.send(WarpMessage::Error(format!("Command failed: {}", e)));
            }
        }
    }
}

impl Default for WarpManager {
    fn default() -> Self {
        Self::new()
    }
}
