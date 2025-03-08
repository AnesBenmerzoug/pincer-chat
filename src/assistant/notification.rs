use relm4::spawn;
use tokio::sync::broadcast;

use super::ollama::types::Message;

pub struct Notifier {
    broadcast_sender: broadcast::Sender<NotifierMessage>,
}

#[derive(Debug, Clone)]
pub enum NotifierMessage {
    NewMessage(Message),
}

impl Notifier {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(100);
        Self {
            broadcast_sender: sender,
        }
    }
    /// Sends a message to all the subscribers.
    pub fn notify(&self, message: NotifierMessage) {
        self.broadcast_sender
            .send(message)
            .expect("Message should be sent");
    }

    /// Subscribe to a [`Notifier`].
    /// Any subscriber will be notified with a message every time
    /// `notify` is called.
    pub fn subscribe<Msg, F>(&self, sender: &relm4::Sender<Msg>, f: F)
    where
        F: Fn(NotifierMessage) -> Msg + 'static + Send + Sync,
        Msg: Send + 'static,
    {
        let sender = sender.clone();
        let mut receiver = self.broadcast_sender.subscribe();

        spawn(async move {
            while let Ok(input) = receiver.recv().await {
                sender.emit(f(input));
            }
        });
    }
}
