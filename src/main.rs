use rbtree::RBTree;
use rust_decimal::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::rc::Rc;
use std::time;
use uuid::Uuid;

#[derive(Copy, Clone, PartialEq, Debug)]
enum Side {
    Bid,
    Ask,
}

#[derive(Copy, Clone, PartialEq, Debug)]
struct Order {
    id: Uuid,
    side: Side,
    timestamp: time::Instant,
    price: Decimal,
    quantity: Decimal,
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

#[derive(Debug)]
struct PriceLevel {
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
}

#[derive(Debug)]
struct BookSide {
    prices: HashMap<Decimal, Rc<RefCell<PriceLevel>>>,
    price_tree: RBTree<Decimal, Rc<RefCell<PriceLevel>>>,
    volume: Decimal,
    num_orders: u32,
    depth: u32,
}

impl BookSide {
    pub fn new() -> Self {
        return BookSide {
            prices: HashMap::new(),
            price_tree: RBTree::new(),
            volume: Decimal::new(0, 0),
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
        price_level.append(order)
    }

    pub fn remove(&mut self, order: Order) -> Option<Order> {
        if let Some(price_level) = self.prices.get(&order.price) {
            let mut price_level = price_level.borrow_mut();
            return price_level.remove(order);
        }

        return None;
    }
}

#[derive(Debug)]
struct OrderBook {
    orders: HashMap<Uuid, Order>,
    bids: BookSide,
    asks: BookSide,
}

// #[derive(Debug)]
// struct OrderResult {
//     fills: Vec<Order>,
//     quantityFilled,
// }

impl OrderBook {
    pub fn new() -> OrderBook {
        return OrderBook {
            orders: HashMap::new(),
            bids: BookSide::new(),
            asks: BookSide::new(),
        };
    }

    // pub fn submit_market_order(side: Side, quantity: Decimal) ->

    // This will go away eventually since we really just want to process market and limit orders.
    pub fn append(&mut self, order: Order) {
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
}

fn main() {
    let mut order_book = OrderBook::new();

    let order1 = Order::new(
        Side::Ask,
        Decimal::new(1001, 2),
        Decimal::new(5000, 2),
        time::Instant::now(),
    );
    order_book.append(order1);

    let order2 = Order::new(
        Side::Ask,
        Decimal::new(1001, 2),
        Decimal::new(7500, 2),
        time::Instant::now(),
    );
    order_book.append(order2);

    let order3 = Order::new(
        Side::Bid,
        Decimal::new(1001, 2),
        Decimal::new(4500, 2),
        time::Instant::now(),
    );
    order_book.append(order3);

    println!("{:?}", order_book);

    order_book.remove(order3.id);

    println!("{:?}", order_book);
}
