use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OrderType {
    Market,
    Limit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub order_id: u64,
    pub trader_id: u64,
    pub order_type: OrderType,
    pub side: Side,
    pub quantity: u64,
    pub price: u64,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub trade_id: u64,
    pub buyer_id: u64,
    pub seller_id: u64,
    pub quantity: u64,
    pub price: u64,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Account {
    pub base_qty_available: u64,
    pub base_qty_locked: u64,
    pub quote_qty_available: u64,
    pub quote_qty_locked: u64,
}

#[derive(Serialize, Deserialize)]
pub enum EngineMessage {
    PlaceOrder(Order),
    CancelOrder {
        side: Side,
        price: u64,
        target_order_id: u64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketData {
    pub bids: Vec<(u64, u64)>,
    pub asks: Vec<(u64, u64)>,
}
