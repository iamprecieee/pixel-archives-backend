use std::sync::atomic::{AtomicUsize, Ordering};

use tokio::sync::broadcast::{self, Receiver, Sender};
use uuid::Uuid;

use crate::ws::types::RoomCanvasUpdate;

pub struct Room {
    sender: Sender<RoomCanvasUpdate>,
    connection_count: AtomicUsize,
    max_connections: usize,
}

impl Room {
    pub fn new(_canvas_id: Uuid, max_connections: usize) -> Self {
        const BROADCAST_BUFFER_SIZE: usize = 256;

        let (sender, _) = broadcast::channel(BROADCAST_BUFFER_SIZE);
        Self {
            sender,
            connection_count: AtomicUsize::new(0),
            max_connections,
        }
    }

    pub async fn get_connection_count(&self) -> usize {
        self.connection_count.load(Ordering::SeqCst)
    }

    pub fn subscribe(&self) -> Option<Receiver<RoomCanvasUpdate>> {
        loop {
            let count = self.connection_count.load(Ordering::SeqCst);
            if count >= self.max_connections {
                return None;
            }

            // Atomically increment only if the count hasn't changed
            match self.connection_count.compare_exchange(
                count,
                count + 1,
                Ordering::SeqCst,
                Ordering::SeqCst,
            ) {
                Ok(_) => return Some(self.sender.subscribe()),
                Err(_) => continue, // Another thread modified the count, retry
            }
        }
    }

    pub fn unsubscribe(&self) {
        self.connection_count.fetch_sub(1, Ordering::SeqCst);
    }

    pub fn broadcast(&self, update: RoomCanvasUpdate) {
        let _ = self.sender.send(update);
    }
}
