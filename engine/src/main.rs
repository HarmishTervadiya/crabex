mod engine;
mod orderbook;
mod types;

use crate::engine::{Asset, Engine};
use crate::types::EngineMessage;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    println!("🦀 Booting Crabex Engine...");
    let (tx, mut rx) = mpsc::channel::<EngineMessage>(1000);

    tokio::spawn(async move {
        let mut engine = Engine::new();
        engine.deposit(111, 1_000_000_000, Asset::Base);
        engine.deposit(111, 1_000_000_000, Asset::Quote);

        println!("🟢 Engine is now listening for orders in the background...");

        while let Some(command) = rx.recv().await {
            match command {
                EngineMessage::CancelOrder {
                    side,
                    price,
                    target_order_id,
                } => {
                    if let Err(e) = engine.place_cancel_order(side, price, target_order_id) {
                        eprintln!("🔴 Order cancellation rejected: {}", e);
                    }
                }
                EngineMessage::PlaceOrder(order) => {
                    if let Err(e) = engine.place_order(order) {
                        eprintln!("🔴 Order rejected: {}", e);
                    }
                }
            };
        }

        println!("Engine shutting down (pipe closed).");
    });

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8000")
        .await
        .unwrap();
    println!("TCP Gateway open! Listening for connections on port 8080...");

    while let Ok((mut socket, addr)) = listener.accept().await {
        println!("New connection from: {}", addr);

        let tx_clone = tx.clone();

        tokio::spawn(async move {
            let (reader, _) = socket.split();
            let mut buf_reader = tokio::io::BufReader::new(reader);
            let mut line = String::new();

            while let Ok(bytes_read) =
                tokio::io::AsyncBufReadExt::read_line(&mut buf_reader, &mut line).await
            {
                if bytes_read == 0 {
                    println!("Client disconnected.");
                    break;
                }

                match serde_json::from_str::<EngineMessage>(&line) {
                    Ok(msg) => {
                        if let Err(e) = tx_clone.send(msg).await {
                            eprintln!("Failed to send message to engine: {}", e);
                        }
                    }
                    Err(e) => {
                        eprintln!("🔴 Invalid JSON received: {}", e);
                    }
                }

                line.clear();
            }
        });
    }
}
