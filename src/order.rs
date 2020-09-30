use rust_decimal::prelude::*;
use std::time;
use uuid::Uuid;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Side {
    Bid,
    Ask,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Order {
    pub id: Uuid,
    pub side: Side,
    pub timestamp: time::Instant,
    pub price: Decimal,
    pub quantity: Decimal,
}

impl Order {
    pub fn new(side: Side, quantity: Decimal, price: Decimal, timestamp: time::Instant) -> Order {
        return Order {
            id: Uuid::new_v4(),
            side,
            price,
            quantity,
            timestamp,
        };
    }
}
