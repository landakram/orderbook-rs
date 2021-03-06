use rust_decimal::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time;
use uuid::Uuid;

use crate::book_side::BookSide;
use crate::order::{Order, Side};
use crate::price_level::PriceLevel;

#[derive(Debug)]
pub struct OrderBook {
    orders: HashMap<Uuid, Order>,
    bids: BookSide,
    asks: BookSide,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum FillStatus {
    Full,
    Partial,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Fill {
    order_id: Uuid,
    status: FillStatus,
    price: Decimal,
    quantity: Decimal,
}

#[derive(Debug)]
pub struct OrderResult {
    done: Vec<Fill>,
    partial: Option<Order>,
    quantity_filled: Decimal,
}

fn iterate_min(side: &BookSide) -> Option<Rc<RefCell<PriceLevel>>> {
    return side.min_price_level();
}

fn iterate_max(side: &BookSide) -> Option<Rc<RefCell<PriceLevel>>> {
    return side.max_price_level();
}

fn greater_than_or_equal(left: Decimal, right: Decimal) -> bool {
    left >= right
}

fn less_than_or_equal(left: Decimal, right: Decimal) -> bool {
    left <= right
}

impl OrderBook {
    pub fn new() -> OrderBook {
        return OrderBook {
            orders: HashMap::new(),
            bids: BookSide::new(),
            asks: BookSide::new(),
        };
    }

    pub fn submit_market_order(&mut self, side: Side, quantity: Decimal) -> OrderResult {
        let iter: fn(&BookSide) -> Option<Rc<RefCell<PriceLevel>>>;

        let mut order_result = OrderResult {
            done: Vec::new(),
            partial: None,
            quantity_filled: Decimal::zero(),
        };
        let mut quantity_left = quantity;

        match side {
            Side::Bid => {
                iter = iterate_min;
            }
            Side::Ask => {
                iter = iterate_max;
            }
        }

        loop {
            if quantity_left <= Decimal::zero() || self.other_book_side(side).num_orders <= 0 {
                break;
            }

            match iter(self.other_book_side(side)) {
                None => break,
                Some(best_price) => {
                    let result = self.fill_at_price_level(best_price, quantity_left);

                    order_result.done.extend(&result.done);
                    order_result.quantity_filled += result.quantity_filled;
                    quantity_left -= result.quantity_filled;
                }
            }
        }

        return order_result;
    }

    pub fn submit_limit_order(
        &mut self,
        side: Side,
        quantity: Decimal,
        price: Decimal,
    ) -> OrderResult {
        let iter: fn(&BookSide) -> Option<Rc<RefCell<PriceLevel>>>;
        let comparator: fn(Decimal, Decimal) -> bool;

        let mut order_result = OrderResult {
            done: Vec::new(),
            partial: None,
            quantity_filled: Decimal::zero(),
        };
        let mut quantity_left = quantity;

        match side {
            Side::Bid => {
                iter = iterate_min;
                comparator = greater_than_or_equal;
            }
            Side::Ask => {
                iter = iterate_max;
                comparator = less_than_or_equal;
            }
        }

        loop {
            match iter(self.other_book_side(side)) {
                None => break,
                Some(best_price) => {
                    if quantity_left <= Decimal::zero()
                        || self.other_book_side(side).num_orders <= 0
                        || !comparator(price, best_price.borrow().price)
                    {
                        break;
                    }

                    let result = self.fill_at_price_level(best_price, quantity_left);

                    order_result.done.extend(&result.done);
                    order_result.quantity_filled += result.quantity_filled;
                    quantity_left -= result.quantity_filled;
                }
            }
        }

        // Add the remaining quantity to the book.
        // Note that we don't implement Time in Force, so the orders are effectively
        // Good Till Canceled (GTC).
        if quantity_left > Decimal::zero() {
            let resting_order = Order::new(side, quantity_left, price, time::Instant::now());

            self.append(resting_order);
            order_result.partial = Some(resting_order);
        }

        order_result
    }

    pub fn get(&self, id: Uuid) -> Option<&Order> {
        return self.orders.get(&id);
    }

    pub fn remove(&mut self, id: Uuid) -> Option<Order> {
        if let Some(order) = self.orders.remove(&id) {
            match order.side {
                Side::Ask => {
                    return self.asks.remove(order);
                }
                Side::Bid => {
                    return self.bids.remove(order);
                }
            }
        }

        return None;
    }

    fn other_book_side(&self, side: Side) -> &BookSide {
        match side {
            Side::Ask => {
                return &self.bids;
            }
            Side::Bid => {
                return &self.asks;
            }
        }
    }

    fn fill_at_price_level(
        &mut self,
        price_level: Rc<RefCell<PriceLevel>>,
        quantity: Decimal,
    ) -> OrderResult {
        let mut order_result = OrderResult {
            done: Vec::new(),
            partial: None,
            quantity_filled: Decimal::zero(),
        };
        let mut quantity_left = quantity;

        while quantity_left > Decimal::zero() && price_level.borrow().len() > 0 {
            let mut remove_id: Option<Uuid> = None;

            {
                let mut price_level = price_level.borrow_mut();
                if let Some(head) = price_level.front() {
                    if quantity_left < head.quantity {
                        let prev_quantity = head.quantity;

                        let mut o = head.clone();
                        o.quantity -= quantity_left;

                        price_level.replace_front(o);
                        self.orders.insert(o.id, o);
                        match o.side {
                            Side::Ask => {
                                self.asks.volume -= prev_quantity;
                                self.asks.volume += o.quantity;
                            }
                            Side::Bid => {
                                self.bids.volume -= prev_quantity;
                                self.bids.volume += o.quantity;
                            }
                        }

                        order_result.done.push(Fill {
                            order_id: o.id,
                            status: FillStatus::Partial,
                            price: o.price,
                            quantity: quantity_left,
                        });
                        order_result.quantity_filled += quantity_left;

                        quantity_left = Decimal::zero();
                    } else {
                        remove_id = Some(head.id);
                    }
                }
            }

            if let Some(id) = remove_id {
                match self.remove(id) {
                    Some(order) => {
                        order_result.done.push(Fill {
                            order_id: order.id,
                            status: FillStatus::Full,
                            price: order.price,
                            quantity: order.quantity,
                        });
                        order_result.quantity_filled += order.quantity;

                        quantity_left -= order.quantity;
                    }
                    None => {
                        println!("this should never happen");
                        // This should never happen
                    }
                }
            }
        }

        return order_result;
    }

    fn append(&mut self, order: Order) {
        self.orders.insert(order.id, order);

        match order.side {
            Side::Ask => {
                self.asks.append(order);
            }
            Side::Bid => {
                self.bids.append(order);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::order::Side;
    use rust_decimal_macros::*;

    #[test]
    fn test_submit_market_order() {
        let mut order_book = OrderBook::new();

        let o1 = order_book.submit_limit_order(Side::Ask, dec!(10.00), dec!(50.00));
        let o2 = order_book.submit_limit_order(Side::Ask, dec!(10.00), dec!(75.00));
        let o3 = order_book.submit_limit_order(Side::Ask, dec!(10.00), dec!(75.00));

        let result = order_book.submit_market_order(Side::Bid, dec!(25.00));
        let mut order_ids = result.done.iter().map(|f| f.order_id);

        // Order was filled with price-time priority
        assert_eq!(result.quantity_filled, dec!(25.00));

        assert_eq!(order_ids.next(), Some(o1.partial.unwrap().id));
        assert_eq!(order_ids.next(), Some(o2.partial.unwrap().id));
        assert_eq!(order_ids.next(), Some(o3.partial.unwrap().id));
        assert_eq!(order_ids.next(), None);

        // Filled orders were taken off the book
        assert!(order_book.get(o1.partial.unwrap().id).is_none());
        assert!(order_book.get(o2.partial.unwrap().id).is_none());

        // Partially filled order is still on the book with updated quantity
        assert_eq!(
            order_book.get(o3.partial.unwrap().id).unwrap().quantity,
            dec!(5.00)
        );
    }

    #[test]
    fn test_submit_market_order_partial() {
        let mut order_book = OrderBook::new();

        let o1 = order_book.submit_limit_order(Side::Ask, dec!(5.00), dec!(50.00));

        let result = order_book.submit_market_order(Side::Bid, dec!(20.00));

        // Order was partially filled
        assert_eq!(result.quantity_filled, dec!(5.00));

        let mut order_ids = result.done.iter().map(|f| f.order_id);
        assert_eq!(order_ids.next(), Some(o1.partial.unwrap().id));
        assert_eq!(order_ids.next(), None);

        // Nothing is left on the book
        assert_eq!(order_book.orders.len(), 0);
    }

    #[test]
    fn test_submit_limit_order() {
        let mut order_book = OrderBook::new();

        let o1 = order_book.submit_limit_order(Side::Ask, dec!(5.00), dec!(50.00));
        let o2 = order_book.submit_limit_order(Side::Ask, dec!(20.00), dec!(51.00));

        let result = order_book.submit_limit_order(Side::Bid, dec!(15.00), dec!(52.00));

        // Order was filled with price-time priority
        assert_eq!(result.quantity_filled, dec!(15.00));

        let mut order_ids = result.done.iter().map(|f| f.order_id);
        assert_eq!(order_ids.next(), Some(o1.partial.unwrap().id));
        assert_eq!(order_ids.next(), Some(o2.partial.unwrap().id));
        assert_eq!(order_ids.next(), None);

        // Filled orders were taken off the book
        assert!(order_book.get(o1.partial.unwrap().id).is_none());

        // Partially filled order is still on the book with updated quantity
        assert_eq!(
            order_book.get(o2.partial.unwrap().id).unwrap().quantity,
            dec!(10.00)
        );
    }

    #[test]
    fn test_submit_limit_order_partial() {
        let mut order_book = OrderBook::new();

        let o1 = order_book.submit_limit_order(Side::Ask, dec!(5.00), dec!(50.00));
        let _o2 = order_book.submit_limit_order(Side::Ask, dec!(20.00), dec!(60.00));

        let result = order_book.submit_limit_order(Side::Bid, dec!(15.00), dec!(55.00));

        // Order was partially filled
        assert_eq!(result.quantity_filled, dec!(5.00));

        let mut order_ids = result.done.iter().map(|f| f.order_id);
        assert_eq!(order_ids.next(), Some(o1.partial.unwrap().id));
        assert_eq!(order_ids.next(), None);

        // Filled orders were taken off the book
        assert!(order_book.get(o1.partial.unwrap().id).is_none());

        // New order for partial quantity was added to the book
        assert_eq!(
            order_book.get(result.partial.unwrap().id).unwrap().quantity,
            dec!(10.00)
        );
    }

    #[test]
    fn test_submit_limit_order_no_fill() {
        let mut order_book = OrderBook::new();

        let _o1 = order_book.submit_limit_order(Side::Ask, dec!(5.00), dec!(50.00));

        let result = order_book.submit_limit_order(Side::Bid, dec!(5.00), dec!(40.00));

        // Order was not filled
        assert_eq!(result.done.len(), 0);

        // Order is now resting on the book
        assert_eq!(
            order_book.get(result.partial.unwrap().id).unwrap().quantity,
            dec!(5.00)
        );
    }

    #[test]
    fn test_remove() {
        let mut order_book = OrderBook::new();

        let _o1 = order_book.submit_limit_order(Side::Ask, dec!(5.00), dec!(50.00));
        let o2 = order_book.submit_limit_order(Side::Bid, dec!(5.00), dec!(40.00));

        let result = order_book.remove(o2.partial.unwrap().id);

        // Order was removed
        assert_eq!(result.unwrap().id, o2.partial.unwrap().id);

        // Order is no longer on the book
        assert_eq!(order_book.get(result.unwrap().id), None);
    }

    #[test]
    fn test_get() {
        let mut order_book = OrderBook::new();

        let result = order_book.submit_limit_order(Side::Ask, dec!(5.00), dec!(50.00));

        // Gets an order on the book
        assert_eq!(
            order_book.get(result.partial.unwrap().id).map(|&o| o),
            result.partial
        );
    }

    #[test]
    fn test_get_no_order() {
        let order_book = OrderBook::new();

        let id = Uuid::new_v4();

        // Returns None for a bogus ID
        assert_eq!(order_book.get(id), None);
    }
}
