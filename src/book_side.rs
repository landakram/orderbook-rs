use rbtree::RBTree;
use rust_decimal::prelude::*;
use rust_decimal_macros::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::order::Order;
use crate::price_level::PriceLevel;

#[derive(Debug)]
pub struct BookSide {
    prices: HashMap<Decimal, Rc<RefCell<PriceLevel>>>,
    price_tree: RBTree<Decimal, Rc<RefCell<PriceLevel>>>,
    pub volume: Decimal,
    pub num_orders: u32,
    pub depth: u32,
}

impl BookSide {
    pub fn new() -> Self {
        return BookSide {
            prices: HashMap::new(),
            price_tree: RBTree::new(),
            volume: dec!(0),
            num_orders: 0,
            depth: 0,
        };
    }

    pub fn append(&mut self, order: Order) {
        let price_level: Rc<RefCell<PriceLevel>>;

        if let Some(pl) = self.prices.get(&order.price) {
            price_level = pl.clone();
        } else {
            price_level = Rc::new(RefCell::new(PriceLevel::new(order.price)));
            self.prices.insert(order.price, price_level.clone());
            self.price_tree.insert(order.price, price_level.clone());
            self.depth += 1;
        }

        let mut price_level = price_level.borrow_mut();
        price_level.append(order);
        self.num_orders += 1;
        self.volume += order.quantity;
    }

    pub fn remove(&mut self, order: Order) -> Option<Order> {
        let mut result = None;
        let mut remove_price_level = false;

        if let Some(price_level) = self.prices.get(&order.price) {
            self.num_orders -= 1;
            self.volume -= order.quantity;
            let mut price_level = price_level.borrow_mut();
            result = price_level.remove(order);

            if price_level.len() <= 0 {
                remove_price_level = true;
            }
        }

        if remove_price_level {
            self.prices.remove(&order.price);
            self.price_tree.remove(&order.price);
            self.depth -= 1;
        }

        return result;
    }

    pub fn min_price_level(&self) -> Option<Rc<RefCell<PriceLevel>>> {
        if self.depth > 0 {
            if let Some((&_price, price_level)) = self.price_tree.get_first() {
                return Some(price_level.clone());
            }
        }

        return None;
    }

    pub fn max_price_level(&self) -> Option<Rc<RefCell<PriceLevel>>> {
        if self.depth > 0 {
            if let Some((&_price, price_level)) = self.price_tree.get_last() {
                return Some(price_level.clone());
            }
        }

        return None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::order::Side;
    use std::time;

    #[test]
    fn test_append_with_no_price_levels() {
        let mut side = BookSide::new();

        let price = dec!(10.00);
        let order = Order::new(Side::Ask, dec!(1.0), price, time::Instant::now());

        side.append(order);

        let pl = side.prices.get(&price).unwrap();
        assert_eq!(
            *pl.borrow().front().unwrap(),
            order,
            "Order wasn't appended"
        );

        let pl2 = side.price_tree.get(&price).unwrap();
        assert_eq!(*pl.borrow(), *pl2.borrow(), "Internal data is inconsistent");

        assert_eq!(side.depth, 1);
        assert_eq!(side.volume, order.quantity);
        assert_eq!(side.num_orders, 1);
    }

    #[test]
    fn test_append_with_existing_price_level() {
        let mut side = BookSide::new();

        let price = dec!(10.00);
        let order = Order::new(Side::Ask, dec!(1.0), price, time::Instant::now());
        let order2 = Order::new(Side::Ask, dec!(2.0), price, time::Instant::now());

        side.append(order);
        side.append(order2);

        let pl = side.prices.get(&price).unwrap();
        assert_eq!(
            *pl.borrow().front().unwrap(),
            order,
            "Order wasn't appended"
        );

        let pl2 = side.price_tree.get(&price).unwrap();
        assert_eq!(*pl.borrow(), *pl2.borrow(), "Internal data is inconsistent");

        assert_eq!(side.depth, 1, "Depth should not increment");
        assert_eq!(side.volume, order.quantity + order2.quantity);
        assert_eq!(side.num_orders, 2);
    }

    #[test]
    fn test_append_with_new_price_level() {
        let mut side = BookSide::new();

        let order = Order::new(Side::Ask, dec!(1.0), dec!(5.0), time::Instant::now());
        let order2 = Order::new(Side::Ask, dec!(2.0), dec!(10.0), time::Instant::now());

        side.append(order);
        side.append(order2);

        assert_eq!(side.prices.len(), 2);
        assert_eq!(side.depth, 2);
    }

    #[test]
    fn test_remove() {
        let mut side = BookSide::new();

        let order = Order::new(Side::Ask, dec!(1.0), dec!(10.0), time::Instant::now());
        let order2 = Order::new(Side::Ask, dec!(2.0), dec!(10.0), time::Instant::now());

        side.append(order);
        side.append(order2);

        side.remove(order2);

        assert_eq!(side.depth, 1);
        assert_eq!(side.volume, order.quantity);
        assert_eq!(side.num_orders, 1);
    }

    #[test]
    fn test_remove_with_last_order_at_price_level() {
        let mut side = BookSide::new();

        let order = Order::new(Side::Ask, dec!(1.0), dec!(10.0), time::Instant::now());

        side.append(order);
        side.remove(order);

        assert_eq!(side.depth, 0);
        assert_eq!(side.volume, Decimal::zero());
        assert_eq!(side.num_orders, 0);
    }
}
