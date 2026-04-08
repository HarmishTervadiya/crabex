use tokio::sync::broadcast;

use crate::orderbook::OrderBook;
use crate::types::{Account, MarketData, Order, Side, Trade};
use std::collections::HashMap;

pub enum Asset {
    Base,
    Quote,
}

pub struct Engine {
    pub accounts: HashMap<u64, Account>,
    pub orderbook: OrderBook,
    pub bcast_tx: broadcast::Sender<String>,
}

impl Engine {
    pub fn new(bcast_tx: broadcast::Sender<String>) -> Self {
        Self {
            accounts: HashMap::new(),
            orderbook: OrderBook::new(),
            bcast_tx,
        }
    }

    pub fn deposit(&mut self, trader_id: u64, amount: u64, asset: Asset) {
        if amount == 0 {
            eprintln!("Invalid deposit amount");
            return;
        }

        let account = self.accounts.entry(trader_id).or_default();

        match asset {
            Asset::Base => {
                account.base_qty_available += amount;
            }
            Asset::Quote => {
                account.quote_qty_available += amount;
            }
        }

        println!("Deposit successfull of {} to {}", amount, trader_id)
    }

    pub fn lock_funds(&mut self, trader_id: u64, amount: u64, asset: Asset) -> Result<(), String> {
        if amount == 0 {
            eprintln!("Invalid withdraw amount");
            return Err("Invalid amount".to_string());
        }

        let account = self.accounts.entry(trader_id).or_default();

        match asset {
            Asset::Base => {
                if account.base_qty_available >= amount {
                    account.base_qty_locked += amount;
                    account.base_qty_available -= amount;
                    println!("Lock successfull of base qty {} from {}", amount, trader_id);

                    Ok(())
                } else {
                    Err("Not enough base funds".to_string())
                }
            }
            Asset::Quote => {
                if account.quote_qty_available >= amount {
                    account.quote_qty_locked += amount;
                    account.quote_qty_available -= amount;
                    println!(
                        "Lock successfull of quote qty {} from {}",
                        amount, trader_id
                    );

                    Ok(())
                } else {
                    Err("Not enough quote funds".to_string())
                }
            }
        }
    }

    pub fn withdraw(&mut self, trader_id: u64, amount: u64, asset: Asset) -> Result<(), String> {
        if amount == 0 {
            eprintln!("Invalid withdraw amount");
            return Err("Invalid amount".to_string());
        }

        let account = self.accounts.entry(trader_id).or_default();

        match asset {
            Asset::Base => {
                if account.base_qty_available >= amount {
                    account.base_qty_available -= amount;
                    println!("Withdraw successfull of base {} from {}", amount, trader_id);

                    Ok(())
                } else {
                    Err("Not enough base funds".to_string())
                }
            }
            Asset::Quote => {
                if account.quote_qty_available >= amount {
                    account.quote_qty_available -= amount;
                    println!(
                        "Withdraw successfull of quote {} from {}",
                        amount, trader_id
                    );

                    Ok(())
                } else {
                    Err("Not enough quote funds".to_string())
                }
            }
        }
    }

    pub fn place_order(&mut self, incoming_order: Order) -> Result<(), String> {
        if incoming_order.price == 0 || incoming_order.quantity == 0 {
            eprintln!("Invalid price or qunatity in order");
            return Err("Invalid price or qunatity in order".to_string());
        }

        let original_price = incoming_order.price.clone();
        let original_side = incoming_order.side.clone();
        let trader_id = incoming_order.trader_id.clone();

        match incoming_order.side {
            Side::Buy => {
                let cost = incoming_order.price * incoming_order.quantity;
                self.lock_funds(incoming_order.trader_id, cost, Asset::Quote)?;
            }
            Side::Sell => {
                self.lock_funds(
                    incoming_order.trader_id,
                    incoming_order.quantity,
                    Asset::Base,
                )?;
            }
        }

        let trades = self.orderbook.process_order(incoming_order);

        // Refund logic
        if original_side == Side::Buy {
            for trade in &trades {
                let price_improvement = original_price - trade.price;
                let refund_amount = price_improvement * trade.quantity;

                if refund_amount > 0 {
                    if let Some(account) = self.accounts.get_mut(&trader_id) {
                        account.quote_qty_locked -= refund_amount;
                        account.quote_qty_available += refund_amount;
                        println!(
                            "Refunded {} to Buyer {} for price improvement!",
                            refund_amount, trader_id
                        );
                    }
                }
            }
        }
        self.settle_trades(trades);

        let market_data = MarketData {
            best_ask: self.orderbook.best_ask(),
            best_bid: self.orderbook.best_bid(),
        };

        if let Ok(json_string) = serde_json::to_string(&market_data) {
            let _ = self.bcast_tx.send(format!("{}\n", json_string));
        }

        Ok(())
    }

    pub fn place_cancel_order(
        &mut self,
        side: Side,
        price: u64,
        target_order_id: u64,
    ) -> Result<(), String> {
        if price == 0 {
            return Err("Insufficient amount".to_string());
        }

        if let Some(cancelled_order) =
            self.orderbook
                .cancel_order(side.clone(), price, target_order_id)
        {
            let account = self.accounts.get_mut(&cancelled_order.trader_id).unwrap();

            match side {
                Side::Buy => {
                    let cost = cancelled_order.price * cancelled_order.quantity;
                    account.quote_qty_available += cost;
                    account.quote_qty_locked -= cost;
                }
                Side::Sell => {
                    account.base_qty_available += cancelled_order.quantity;
                    account.base_qty_locked -= cancelled_order.quantity;
                }
            }
        }

        Ok(())
    }

    pub fn settle_trades(&mut self, mut trades: Vec<Trade>) {
        for trade in trades {
            let total_cost = trade.price * trade.quantity;

            if let Some(buyer) = self.accounts.get_mut(&trade.buyer_id) {
                buyer.quote_qty_locked -= total_cost;
                buyer.base_qty_available += trade.quantity;
            }

            if let Some(seller) = self.accounts.get_mut(&trade.seller_id) {
                seller.base_qty_locked -= trade.quantity;
                seller.quote_qty_available += total_cost;
            }

            println!(
                "Settled Trade {} between Buyer {} and Seller {}",
                trade.trade_id, trade.buyer_id, trade.seller_id
            );
        }
    }
}
