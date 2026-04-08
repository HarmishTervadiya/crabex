use rand::Rng;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc;

#[derive(Debug, Deserialize)]
pub struct MarketData {
    pub best_bid: Option<u64>,
    pub best_ask: Option<u64>,
}

#[tokio::main]
async fn main() {
    println!("🤖 Booting Arb Bot...");

    let (crabex_tx, mut crabex_rx) = mpsc::channel::<MarketData>(1000);
    let (binance_tx, mut binance_rx) = mpsc::channel::<MarketData>(1000);

    tokio::spawn(async move {
        listen_to_exchange(crabex_tx).await;
    });

    tokio::spawn(async move {
        listen_to_binance(binance_tx).await;
    });

    println!("Waiting for prices...");

    let mut current_crabex = MarketData {
        best_ask: None,
        best_bid: None,
    };
    let mut current_binance = MarketData {
        best_ask: None,
        best_bid: None,
    };

    loop {
        tokio::select! {
         Some(crabex_data) = crabex_rx.recv() =>  {
            current_crabex=crabex_data;
            if let Some((side, price)) = check_for_arbitrage(&current_crabex, &current_binance) {
                execute_trade(side, price).await;
            };
        },
         Some(binance_data) = binance_rx.recv()=> {
            current_binance=binance_data;
            if let Some((side, price)) = check_for_arbitrage(&current_crabex, &current_binance) {
                execute_trade(side, price).await;
            };
         }
        }
    }
}

// This function will listen to the TCP socket and translate JSON into our MarketData struct.
async fn listen_to_exchange(tx: mpsc::Sender<MarketData>) {
    let mut stream = match TcpStream::connect("127.0.0.1:8000").await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("🔴 Eyes failed to connect to Crabex: {}", e);
            return;
        }
    };

    let (reader, _) = stream.split();
    let mut buf_reader = tokio::io::BufReader::new(reader);
    let mut line = String::new();

    while let Ok(bytes_read) = buf_reader.read_line(&mut line).await {
        if bytes_read == 0 {
            println!("Crabex disconnected our Eyes.");
            break;
        }

        match serde_json::from_str::<MarketData>(&line) {
            Ok(market_data) => {
                if let Err(e) = tx.send(market_data).await {
                    eprintln!("Failed to send data to Brain: {}", e);
                }
            }
            Err(_) => {}
        }

        line.clear();
    }
}

// --- THE FAKE BINANCE ---
// This just pumps out fake Market Data every 1 second.
async fn listen_to_binance(tx: mpsc::Sender<MarketData>) {
    let mut counter = 0;
    loop {
        // Make the price bounce between $9, $10, and $11
        let fake_bid = 9 + (counter % 3);
        let fake_ask = fake_bid + 1; // The Ask (sellers) is always $1 higher than the Bid (buyers)

        let data = MarketData {
            best_bid: Some(fake_bid),
            best_ask: Some(fake_ask),
        };

        let _ = tx.send(data).await;

        // Wait 1 second before sending the next price
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        counter += 1;
    }
}
// This function takes the prices from Crabex and Binance and looks for a mathematical gap.
fn check_for_arbitrage<'a>(crabex: &MarketData, binance: &MarketData) -> Option<(&'a str, u64)> {
    if let (Some(c_bid), Some(c_ask), Some(b_bid), Some(b_ask)) = (
        crabex.best_bid,
        crabex.best_ask,
        binance.best_bid,
        binance.best_ask,
    ) {
        if c_ask < b_bid {
            println!(
                "ARBITRAGE! Buy on Crabex (${}), Sell on Binance (${})",
                c_ask, b_bid
            );
            return Some(("Buy", c_ask));
        } else if b_ask < c_bid {
            println!(
                "ARBITRAGE! Buy on Binance (${}), Sell on Crabex (${})",
                b_ask, c_bid
            );

            return Some(("Sell", c_bid));
        } else {
            None
        }
    } else {
        None
    }
}

// This function will build the "PlaceOrder" JSON and shove it down the TCP socket.
async fn execute_trade(side: &str, price: u64) {
    if let Ok(mut stream) = TcpStream::connect("127.0.0.1:8000").await {
        let random_id: u64 = rand::random();
        let json_msg = format!(
            "{{\"PlaceOrder\":{{\"order_id\":{},\"trader_id\":333,\"order_type\":\"Limit\",\"side\":\"{}\",\"quantity\":1,\"price\":{},\"timestamp\":0}}}}\n",
            random_id, side, price
        );

        let _ = stream.write_all(json_msg.as_bytes()).await;
        println!("Order fired on Crabex: {} at ${}", side, price);
    }
}
