use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use tokio::sync::broadcast;

use crate::features::webrtc_signaling::dto::SignalMessage;

const SIGNAL_CHANNEL_CAPACITY: usize = 256;

#[derive(Clone, Default)]
pub struct SignalHub {
    /// Clave: "swarm:{resource_id}" o "peer:{peer_id}"
    channels: Arc<RwLock<HashMap<String, broadcast::Sender<String>>>>,
}

impl SignalHub {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn subscribe(&self, topic: &str) -> broadcast::Receiver<String> {
        let sender = {
            let mut guard = self.channels.write().expect("SignalHub lock poisoned");
            guard
                .entry(topic.to_string())
                .or_insert_with(|| {
                    let (tx, _) = broadcast::channel(SIGNAL_CHANNEL_CAPACITY);
                    tx
                })
                .clone()
        };
        sender.subscribe()
    }

    pub fn publish(&self, topic: &str, msg: &SignalMessage) {
        let sender_opt = {
            let guard = self.channels.read().expect("SignalHub lock poisoned");
            guard.get(topic).cloned()
        };

        if let Some(tx) = sender_opt {
            if let Ok(json) = serde_json::to_string(msg) {
                let _ = tx.send(json);
            }
        }
    }
}