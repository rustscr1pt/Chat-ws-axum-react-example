use std::sync::Arc;
use axum::{
    extract::{ws::WebSocket, WebSocketUpgrade},
    response::Response,
    routing::get,
    Router,
};
use axum::extract::State;
use axum::extract::ws::Message;
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::sync::Mutex;

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    let router = Router::new().merge(route());
    Ok(router.into())
}

#[derive(Debug, Clone)]
struct AppState {
    broadcast_tx : Arc<Mutex<Sender<Message>>>
}

fn route() -> Router {
    let (tx, _) = tokio::sync::broadcast::channel(32);
    let app = AppState {
        broadcast_tx : Arc::new(Mutex::new(tx))
    };
    Router::new().route("/", get(handler)).with_state(app)
}

async fn handler(ws: WebSocketUpgrade, State(app) : State<AppState>) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, app))
}

// An example of easy websocket implementation where message sent from client is echoed
// async fn handle_socket(mut ws: WebSocket) {
//     while let Some(msg) = ws.recv().await {
//         let msg = if let Ok(msg) = msg {
//             msg
//         } else {
//             return; // client disconnected
//         };
//         if ws.send(msg).await.is_err() {
//             return; // client disconnected
//         }
//     }
// }

async fn handle_socket(ws : WebSocket, app : AppState) {
    let (ws_tx, ws_rx) = ws.split();
    let ws_tx = Arc::new(Mutex::new(ws_tx));
    {
        let broadcast_rx = app.broadcast_tx.lock().await.subscribe();
        tokio::spawn(async move {
            recv_broadcast(ws_tx, broadcast_rx).await;
        });
    }
    recv_from_client(ws_rx, app.broadcast_tx).await;
}

async fn recv_from_client(mut client_rx : SplitStream<WebSocket>, broadcast_tx : Arc<Mutex<Sender<Message>>>) {
    while let Some(Ok(msg)) = client_rx.next().await {
        if matches!(msg, Message::Close(_)) {
            return;
        }
        if broadcast_tx.lock().await.send(msg).is_err() {
            println!("Failed to broadcast the message");
        }
    }
}

async fn recv_broadcast(client_tx : Arc<Mutex<SplitSink<WebSocket, Message>>>, mut broadcast_rx : Receiver<Message>) {
    while let Ok(msg) = broadcast_rx.recv().await {
        if client_tx.lock().await.send(msg).await.is_err() {
            return;
        }
    }
}