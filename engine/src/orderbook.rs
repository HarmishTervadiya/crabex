use crate::types::{Order, OrderType, Side, Trade};
use std::collections::{BTreeMap, VecDeque};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct OrderBook {
    pub bids: BTreeMap<u64, VecDeque<Order>>,
    pub asks: BTreeMap<u64, VecDeque<Order>>,
    pub trade_counter: u64,
}

impl OrderBook {
    pub fn new() -> Self {
        Self {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            trade_counter: 0,
        }
    }

    pub fn process_order(&mut self, mut incoming_order: Order) -> Vec<Trade> {
        let mut trades = Vec::new();

        match incoming_order.side {
            Side::Buy => {
                let asks_prices: Vec<u64> = self.asks.keys().cloned().collect();

                for ask_price in asks_prices {
                    if incoming_order.order_type == OrderType::Limit
                        && ask_price > incoming_order.price
                    {
                        break;
                    }

                    if let Some(queue) = self.asks.get_mut(&ask_price) {
                        while let Some(mut resting_ask) = queue.pop_front() {
                            let qty = incoming_order.quantity.min(resting_ask.quantity);

                            incoming_order.quantity -= qty;
                            resting_ask.quantity -= qty;

                            self.trade_counter += 1;
                            let current_timestamp = SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .expect("Time went backwards")
                                .as_secs();

                            trades.push(Trade {
                                trade_id: self.trade_counter,
                                buyer_id: incoming_order.trader_id,
                                seller_id: resting_ask.trader_id,
                                price: ask_price,
                                quantity: qty,
                                timestamp: current_timestamp,
                            });

                            if resting_ask.quantity > 0 {
                                queue.push_front(resting_ask);
                            }

                            if incoming_order.quantity == 0 {
                                break;
                            }
                        }

                        if queue.is_empty() {
                            self.asks.remove(&ask_price);
                        }
                    }

                    if incoming_order.quantity == 0 {
                        break;
                    }
                }

                if incoming_order.order_type == OrderType::Limit && incoming_order.quantity > 0 {
                    self.bids
                        .entry(incoming_order.price)
                        .or_insert_with(VecDeque::new)
                        .push_back(incoming_order);
                }
            }
            Side::Sell => {
                let bid_prices: Vec<u64> = self.bids.keys().rev().cloned().collect();

                for bid_price in bid_prices {
                    if incoming_order.order_type == OrderType::Limit
                        && incoming_order.price > bid_price
                    {
                        break;
                    }

                    if let Some(queue) = self.bids.get_mut(&bid_price) {
                        while let Some(mut resting_bid) = queue.pop_front() {
                            let qty = incoming_order.quantity.min(resting_bid.quantity);

                            incoming_order.quantity -= qty;
                            resting_bid.quantity -= qty;

                            self.trade_counter += 1;
                            let current_timestamps = SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .expect("Time went backwards")
                                .as_secs();

                            trades.push(Trade {
                                trade_id: self.trade_counter,
                                buyer_id: resting_bid.trader_id,
                                seller_id: incoming_order.trader_id,
                                price: bid_price,
                                quantity: qty,
                                timestamp: current_timestamps,
                            });

                            if resting_bid.quantity > 0 {
                                queue.push_front(resting_bid);
                            }

                            if incoming_order.quantity == 0 {
                                break;
                            }
                        }

                        if queue.is_empty() {
                            self.bids.remove(&bid_price);
                        }
                    }

                    if incoming_order.quantity == 0 {
                        break;
                    }
                }

                if incoming_order.order_type == OrderType::Limit && incoming_order.quantity > 0 {
                    self.asks
                        .entry(incoming_order.price)
                        .or_insert_with(VecDeque::new)
                        .push_back(incoming_order);
                }
            }
        }

        trades
    }

    pub fn cancel_order(&mut self, side: Side, price: u64, target_order_id: u64) -> Option<Order> {
        let tree: &mut BTreeMap<u64, VecDeque<Order>> = match side {
            Side::Buy => &mut self.bids,
            Side::Sell => &mut self.asks,
        };

        if let Some(queue) = tree.get_mut(&price) {
            let cancelled_order = queue
                .iter()
                .find(|order| order.order_id == target_order_id)
                .cloned();

            queue.retain(|order| order.order_id != target_order_id);

            if queue.is_empty() {
                tree.remove(&price);
            }

            return cancelled_order;
        }

        None
    }
}
