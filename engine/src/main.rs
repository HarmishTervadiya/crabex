mod engine;
mod orderbook;
mod types;

use crate::engine::{Asset, Engine};
use crate::types::{Order, OrderType, Side};

fn main() {
    println!("🦀 Booting Crabex Engine...");
    let mut engine = Engine::new();

    engine.deposit(111, 100, Asset::Base);  
    engine.deposit(111, 100, Asset::Quote); 

    engine.deposit(222, 100, Asset::Base);  
    engine.deposit(222, 100, Asset::Quote); 

    println!("\n--- User 111 places a LIMIT SELL ---");
    let sell_order = Order {
        order_id: 1,
        order_type: OrderType::Limit,
        price: 5,
        quantity: 10,
        side: Side::Sell,
        timestamp: 0,
        trader_id: 111,
    };
    engine.place_order(sell_order).unwrap(); 

    println!("\n--- User 222 places a LIMIT BUY ---");
    let buy_order = Order {
        order_id: 2,
        order_type: OrderType::Limit,
        price: 10,
        quantity: 10,
        side: Side::Buy,
        timestamp: 0,
        trader_id: 222,
    };
    engine.place_order(buy_order).unwrap(); 

    println!("\n--- FINAL BALANCES ---");
    println!("User 111 (Seller): {:#?}", engine.accounts.get(&111).unwrap());
    
    println!("User 222 (Buyer): {:#?}", engine.accounts.get(&222).unwrap());
}