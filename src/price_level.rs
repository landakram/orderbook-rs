use rust_decimal::prelude::*;
use rust_decimal_macros::*;
use std::collections::VecDeque;

use crate::order::Order;

#[derive(Debug, Eq, PartialEq)]
pub struct PriceLevel {
    pub volume: Decimal,
    pub price: Decimal,
    orders: VecDeque<Order>,
}

impl PriceLevel {
    pub fn new(price: Decimal) -> Self {
        return PriceLevel {
            volume: dec!(0),
            price: price,
            orders: VecDeque::new(),
        };
    }

    pub fn append(&mut self, order: Order) {
        self.volume += order.quantity;
        self.orders.push_back(order);
    }

    pub fn remove(&mut self, order: Order) -> Option<Order> {
        self.volume -= order.quantity;
        if let Some(pos) = self.orders.iter().position(|&o| o == order) {
            return self.orders.remove(pos);
        }

        return None;
    }

    pub fn len(&self) -> usize {
        return self.orders.len();
    }

    pub fn front(&self) -> Option<&Order> {
        return self.orders.front();
    }

    pub fn replace_front(&mut self, order: Order) {
        let mut quantity = dec!(0);

        if let Some(o) = self.front_mut() {
            quantity = o.quantity;

            o.id = order.id;
            o.price = order.price;
            o.quantity = order.quantity;
            o.side = order.side;
            o.timestamp = order.timestamp;
        }

        self.volume -= quantity;
        self.volume += order.quantity;
    }

    fn front_mut(&mut self) -> Option<&mut Order> {
        return self.orders.front_mut();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::order::Side;
    use std::time;

    #[test]
    fn test_append() {
        let mut price_level = PriceLevel::new(dec!(10.00));
        let order = Order::new(Side::Ask, dec!(1.0), dec!(10.00), time::Instant::now());

        price_level.append(order);

        assert_eq!(price_level.volume, order.quantity);
        assert_eq!(*price_level.front().unwrap(), order);
    }

    #[test]
    fn test_remove() {
        let mut price_level = PriceLevel::new(dec!(10.00));
        let order = Order::new(Side::Ask, dec!(1.0), dec!(10.00), time::Instant::now());
        let order2 = Order::new(Side::Ask, dec!(2.0), dec!(10.00), time::Instant::now());

        price_level.append(order);
        price_level.append(order2);

        price_level.remove(order);

        assert_eq!(price_level.volume, order2.quantity);
        assert_eq!(*price_level.front().unwrap(), order2);
    }

    #[test]
    fn test_len() {
        let mut price_level = PriceLevel::new(dec!(10.00));
        let order = Order::new(Side::Ask, dec!(1.0), dec!(10.00), time::Instant::now());
        let order2 = Order::new(Side::Ask, dec!(2.0), dec!(10.00), time::Instant::now());

        price_level.append(order);
        price_level.append(order2);

        assert_eq!(price_level.len(), 2);
    }

    #[test]
    fn test_front() {
        let mut price_level = PriceLevel::new(dec!(10.00));
        let order = Order::new(Side::Ask, dec!(1.0), dec!(10.00), time::Instant::now());
        let order2 = Order::new(Side::Ask, dec!(2.0), dec!(10.00), time::Instant::now());

        price_level.append(order);
        price_level.append(order2);

        assert_eq!(*price_level.front().unwrap(), order);
    }

    #[test]
    fn test_replace_front() {
        let mut price_level = PriceLevel::new(dec!(10.00));
        let order = Order::new(Side::Ask, dec!(1.0), dec!(10.00), time::Instant::now());
        let order2 = Order::new(Side::Ask, dec!(2.0), dec!(10.00), time::Instant::now());

        price_level.append(order);
        price_level.append(order2);

        let mut new_order = order.clone();
        new_order.quantity = dec!(0.1);

        price_level.replace_front(new_order);
        assert_eq!(*price_level.front().unwrap(), new_order);
        assert_eq!(price_level.volume, new_order.quantity + order2.quantity);
    }
}
