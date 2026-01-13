use std::net::SocketAddr;

use axum::{
    extract::{
        ConnectInfo, Query, State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::Response,
};
use axum_extra::TypedHeader;
use futures::{SinkExt, StreamExt};
use headers::Cookie;
use tokio::sync::broadcast::{Receiver, error};
use uuid::Uuid;

use crate::{
    AppState,
    error::AppError,
    services::auth::TokenType,
    ws::types::{ClientMessage, RoomCanvasUpdate, WsQuery},
};

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    ConnectInfo(_addr): ConnectInfo<SocketAddr>,
    Query(query): Query<WsQuery>,
    cookies: Option<TypedHeader<Cookie>>,
) -> Result<Response, AppError> {
    let token = cookies
        .as_ref()
        .and_then(|c| c.get("access_token"))
        .map(|s| s.to_string())
        .ok_or(AppError::Unauthorized)?;

    let user_id = state
        .jwt_service
        .validate_token(&token, TokenType::Access)
        .map_err(|_| AppError::Unauthorized)?
        .sub;

    Ok(ws.on_upgrade(move |socket| handle_socket(socket, state, query, user_id)))
}

async fn handle_socket(socket: WebSocket, state: AppState, query: WsQuery, user_id: Uuid) {
    let canvas_id = query.canvas_id;
    tracing::info!("WebSocket connection for canvas {canvas_id} from user {user_id}");

    let room = state.ws_rooms.get_or_create_room(canvas_id).await;

    let receiver = match room.subscribe() {
        Some(value) => value,
        None => {
            tracing::warn!("Room full for canvas {canvas_id}");
            return;
        }
    };

    room.broadcast(RoomCanvasUpdate::UserJoined { user_id });
    handle_connection(socket, receiver).await;

    room.unsubscribe();
    room.broadcast(RoomCanvasUpdate::UserLeft { user_id });
    state.ws_rooms.remove_room_if_empty(&canvas_id).await;

    tracing::info!("WebSocket disconnected for canvas {canvas_id}");
}

async fn handle_connection(socket: WebSocket, mut ws_receiver: Receiver<RoomCanvasUpdate>) {
    let (mut sender, mut receiver) = socket.split();

    loop {
        tokio::select! {
            // Handle incoming messages
            msg = receiver.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        if let Ok(ClientMessage::Ping) = serde_json::from_str::<ClientMessage>(&text)
                            && sender.send(Message::Text("pong".into())).await.is_err() {
                                break;
                            }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Err(e)) => {
                        tracing::error!("WebSocket error: {e}");
                        break;
                    }
                    _ => {}
                }
            }

            // Handle broadcasts
            update = ws_receiver.recv() => {
                match update {
                    Ok(update) => {
                        match serde_json::to_string(&update) {
                            Ok(json) => {
                                if sender.send(Message::Text(json.into())).await.is_err() {
                                    break;
                                }
                            }
                            Err(e) => {
                                tracing::error!("Failed to serialize update: {e}");
                            }
                        }
                    }
                    Err(error::RecvError::Lagged(n)) => {
                        tracing::warn!("Lagged {n} messages");
                    }
                    Err(_) => break,
                }
            }
        }
    }
}
