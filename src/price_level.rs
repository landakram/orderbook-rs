use rust_decimal::prelude::*;
use std::collections::VecDeque;

use crate::order::Order;

#[derive(Debug)]
pub struct PriceLevel {
    volume: Decimal,
    price: Decimal,
    orders: VecDeque<Order>,
}

impl PriceLevel {
    pub fn new(price: Decimal) -> Self {
        return PriceLevel {
            volume: Decimal::new(0, 0),
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

    pub fn back(&self) -> Option<&Order> {
        return self.orders.back();
    }

    pub fn replace_front(&mut self, order: Order) {
        let mut quantity = Decimal::zero();

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

    pub fn front_mut(&mut self) -> Option<&mut Order> {
        return self.orders.front_mut();
    }

    pub fn back_mut(&mut self) -> Option<&mut Order> {
        return self.orders.back_mut();
    }
}