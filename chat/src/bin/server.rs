use futures_util::sink::SinkExt;
use futures_util::stream::StreamExt;
use serde_json::{json, Value};
use std::error::Error;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast::{channel, Sender};
use tokio_websockets::{Message, ServerBuilder, WebSocketStream};


type UserList = Arc<Mutex<Vec<String>>>;

async fn handle_connection(
    addr: SocketAddr,
    mut ws_stream: WebSocketStream<TcpStream>,
    bcast_tx: Sender<String>,
    users: UserList,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut bcast_rx = bcast_tx.subscribe();
    let mut username = String::new();

    loop {
        tokio::select! {
            incoming = ws_stream.next() => {
                match incoming {
                    Some(Ok(msg)) => {
                        if let Some(text) = msg.as_text() {
                            println!("From YewClient {addr:?} : {text}");
                            
                            if let Ok(parsed) = serde_json::from_str::<Value>(text) {
                                if parsed["messageType"] == "register" {
                                    if let Some(name) = parsed["data"].as_str() {
                                        username = name.to_string();
                                        

                                        {
                                            let mut users_lock = users.lock().unwrap();
                                            if !users_lock.contains(&username) {
                                                users_lock.push(username.clone());
                                            }
                                            

                                            let response = json!({
                                                "messageType": "users",
                                                "dataArray": *users_lock
                                            }).to_string();
                                            bcast_tx.send(response)?;
                                        }
                                    }
                                } else if parsed["messageType"] == "message" {
                                    if let Some(msg_text) = parsed["data"].as_str() {

                                        let inner_data = json!({
                                            "from": username,
                                            "message": msg_text
                                        }).to_string();
                                        
                                        let response = json!({
                                            "messageType": "message",
                                            "data": inner_data
                                        }).to_string();
                                        
                                        bcast_tx.send(response)?;
                                    }
                                }
                            }
                        }
                    }
                    Some(Err(err)) => return Err(err.into()),
                    None => break,
                }
            }
            msg = bcast_rx.recv() => {
                ws_stream.send(Message::text(msg?)).await?;
            }
        }
    }


    if !username.is_empty() {
        let mut users_lock = users.lock().unwrap();
        users_lock.retain(|u| u != &username);
        let response = json!({
            "messageType": "users",
            "dataArray": *users_lock
        }).to_string();
        let _ = bcast_tx.send(response);
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let (bcast_tx, _) = channel(16);

    let users: UserList = Arc::new(Mutex::new(Vec::new()));

    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("listening on port 8080");

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("New connection from {addr:?}");
        
        let bcast_tx = bcast_tx.clone();
        let users = users.clone();
        
        tokio::spawn(async move {
            let (_req, ws_stream) = ServerBuilder::new().accept(socket).await.unwrap();
            if let Err(e) = handle_connection(addr, ws_stream, bcast_tx, users).await {
                println!("Error processing connection: {e}");
            }
        });
    }
}