mod engine;
mod orderbook;
mod types;

use crate::engine::{Asset, Engine};
use crate::types::{EngineMessage};
use tokio::sync::broadcast;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    println!("🦀 Booting Crabex Engine...");
    let (tx, mut rx) = mpsc::channel::<EngineMessage>(1000);

    let (bcast_tx, _) = broadcast::channel::<String>(1000);

    let engine_bcast_tx = bcast_tx.clone();
    tokio::spawn(async move {
        let mut engine = Engine::new(engine_bcast_tx);

        engine.deposit(111, 1_000_000_000, Asset::Base);
        engine.deposit(111, 1_000_000_000, Asset::Quote);
        engine.deposit(222, 1_000_000_000, Asset::Base);
        engine.deposit(222, 1_000_000_000, Asset::Quote);

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
        let mut bcast_rx = bcast_tx.subscribe();

        tokio::spawn(async move {
            let (reader, mut writer) = socket.split();
            let mut buf_reader = tokio::io::BufReader::new(reader);
            let mut line = String::new();

            loop {
                tokio::select! {
                   result=tokio::io::AsyncBufReadExt::read_line(&mut buf_reader, &mut line) => {
                       match result {
                           Ok(0) => {
                               println!("Client disconnected.");
                               break;
                           },
                           Ok(_)=> {
                            if let Ok(msg) = serde_json::from_str::<EngineMessage>(&line) {
                                   let _ = tx_clone.send(msg).await;
                               }
                               line.clear();
                           }
                           Err(_) => break,
                       }
                   },
                   result = bcast_rx.recv() => {
                        match result {
                            Ok(market_data_json) => {
                                if tokio::io::AsyncWriteExt::write_all(&mut writer, market_data_json.as_bytes()).await.is_err() {
                                    break;
                                }
                            }
                            Err(_) => break,
                        }
                    }

                }
            }
        });
    }
}
