#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Clone)]
pub enum OrderType {
    Market,
    Limit,
}

#[derive(Debug, Clone)]
pub struct Order {
    pub order_id: u64,
    pub trader_id: u64,
    pub order_type: OrderType,
    pub side: Side,
    pub quantity: u64,
    pub price: u64,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct Trade {
    pub trade_id: u64,
    pub buyer_id: u64,
    pub seller_id: u64,
    pub quantity: u64,
    pub price: u64,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Default)]
pub struct Account {
    pub base_qty_available: u64,
    pub base_qty_locked: u64,
    pub quote_qty_available: u64,
    pub quote_qty_locked: u64,
}