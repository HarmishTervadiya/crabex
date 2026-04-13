use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::time::{sleep, Duration}; // Add this!

#[tokio::main]
async fn main() {
    println!("🤖 Booting Infinite Market Simulator...");

    let mut stream = TcpStream::connect("127.0.0.1:8000").await.unwrap();
    println!("🟢 Connected to Engine! Firing continuous orders...");

    let mut i = 0;
    
    loop {
        i += 1;
        let side = if i % 2 == 0 { "Buy" } else { "Sell" };
        
        let price = if side == "Buy" {
            90 + (i % 10) 
        } else {
            101 + (i % 10)
        };

        let json_msg = format!(
            "{{\"PlaceOrder\":{{\"order_id\":{},\"trader_id\":111,\"order_type\":\"Limit\",\"side\":\"{}\",\"quantity\":1,\"price\":{},\"timestamp\":0}}}}\n",
            i, side, price
        );

        stream.write_all(json_msg.as_bytes()).await.unwrap();

        // This gives you 200 orders per second—perfect for human eyes to watch.
        sleep(Duration::from_millis(5)).await; 
    }
}