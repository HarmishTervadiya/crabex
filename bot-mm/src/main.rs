use std::time::Instant;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

#[tokio::main]
async fn main() {
    println!("🤖 Booting Crabex Stress-Test Bot...");

    let mut stream = TcpStream::connect("127.0.0.1:8000").await.unwrap();
    println!("🟢 Connected to Engine!");

    let num_orders = 10_000;
    println!("🔥 Firing {} orders...", num_orders);

    let start_time = Instant::now();

    for i in 1..=num_orders {
        let side = if i % 2 == 0 { "Buy" } else { "Sell" };
        let price = 10 + (i % 3);

        let json_msg = format!(
            "{{\"PlaceOrder\":{{\"order_id\":{},\"trader_id\":111,\"order_type\":\"Limit\",\"side\":\"{}\",\"quantity\":1,\"price\":{},\"timestamp\":0}}}}\n",
            i, side, price
        );

        stream.write_all(json_msg.as_bytes()).await.unwrap();
    }

    let elapsed = start_time.elapsed();
    
    // Calculate Transactions Per Second (TPS)
    let tps = (num_orders as f64 / elapsed.as_secs_f64()) as u64;

    println!("⚡ Blasted {} orders in {:?}", num_orders, elapsed);
    println!("🚀 Bot TPS (Transactions Per Second): {}", tps);
}
