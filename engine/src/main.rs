mod engine;
mod orderbook;
mod types;

use crate::engine::{Asset, Engine};
use crate::types::{EngineMessage, Order, OrderType, Side};
use tokio::sync::mpsc;
use tokio::time::{Duration, sleep};

#[tokio::main]
async fn main() {
    println!("🦀 Booting Crabex Engine...");

    let (tx, mut rx) = mpsc::channel::<EngineMessage>(1000);

    tokio::spawn(async move {
        let mut engine = Engine::new();
        engine.deposit(111, 100, Asset::Base);
        engine.deposit(111, 100, Asset::Quote);
        engine.deposit(222, 100, Asset::Base);
        engine.deposit(222, 100, Asset::Quote);
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

    sleep(Duration::from_millis(10)).await;

    let sell_order = Order {
        order_id: 1,
        order_type: OrderType::Limit,
        price: 5,
        quantity: 10,
        side: Side::Sell,
        timestamp: 0,
        trader_id: 111,
    };

    let buy_order = Order {
        order_id: 2,
        order_type: OrderType::Limit,
        price: 10,
        quantity: 10,
        side: Side::Buy,
        timestamp: 0,
        trader_id: 222,
    };

    println!("\n[Bot] Shooting Sell Order down the pipe...");
    tx.send(EngineMessage::PlaceOrder((sell_order)))
        .await
        .unwrap();

    println!("[Bot] Shooting Buy Order down the pipe...");
    tx.send(EngineMessage::PlaceOrder((buy_order)))
        .await
        .unwrap();

    println!("[Bot] Cancelling Order 99...");
    tx.send(EngineMessage::CancelOrder {
        side: Side::Buy,
        price: 10,
        target_order_id: 99,
    })
    .await
    .unwrap();
    sleep(Duration::from_millis(50)).await;

    println!("Program finished successfully!");
}
