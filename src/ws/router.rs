use std::{collections::HashMap, sync::Arc};

use axum::{Router, routing::get};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{
    AppState,
    ws::{handler::ws_handler, room::Room, types::RoomCanvasUpdate},
};

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(ws_handler))
}

pub struct RoomManager {
    rooms: RwLock<HashMap<Uuid, Arc<Room>>>,
    max_connections_per_room: usize,
}

impl RoomManager {
    pub fn initialize(max_connections: usize) -> Self {
        Self {
            rooms: RwLock::new(HashMap::new()),
            max_connections_per_room: max_connections,
        }
    }

    pub async fn broadcast(&self, canvas_id: &Uuid, update: RoomCanvasUpdate) {
        let rooms = self.rooms.read().await;
        if let Some(room) = rooms.get(canvas_id) {
            room.broadcast(update);
        }
    }

    pub async fn get_or_create_room(&self, canvas_id: Uuid) -> Arc<Room> {
        {
            let rooms = self.rooms.read().await;
            if let Some(room) = rooms.get(&canvas_id) {
                return Arc::clone(room);
            }
        }

        let mut rooms = self.rooms.write().await;
        rooms
            .entry(canvas_id)
            .or_insert_with(|| Arc::new(Room::new(canvas_id, self.max_connections_per_room)))
            .clone()
    }

    pub async fn remove_room_if_empty(&self, canvas_id: &Uuid) {
        let mut rooms = self.rooms.write().await;
        if let Some(room) = rooms.get(canvas_id)
            && room.get_connection_count().await == 0
        {
            rooms.remove(canvas_id);
        }
    }
}
