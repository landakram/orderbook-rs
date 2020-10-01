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

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::*;

    #[test]
    fn test_new_returns_order() {
        let side = Side::Ask;
        let quantity = dec!(1.0);
        let price = dec!(10.0);
        let time = time::Instant::now();

        let order = Order::new(side, quantity, price, time);

        assert_eq!(order.side, side);
        assert_eq!(order.quantity, quantity);
        assert_eq!(order.price, price);
        assert_eq!(order.timestamp, time);
    }
}
