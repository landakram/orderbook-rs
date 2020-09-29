use rbtree::RBTree;
use rust_decimal::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::rc::Rc;
use std::time;
use uuid::Uuid;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum Side {
    Bid,
    Ask,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
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
        if let Some(pos) = self.orders.iter().position(|&o| {
            println!("{:#?}", o);
            println!("{:#?}", order);
            println!("{}", o == order);
            return o == order;
        }) {
            println!("pos {}", pos);
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
        price_level.append(order);
        self.num_orders += 1;
    }

    pub fn remove(&mut self, order: Order) -> Option<Order> {
        let mut result = None;
        let mut remove_price_level = false;

        if let Some(price_level) = self.prices.get(&order.price) {
            self.num_orders -= 1;
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

#[derive(Debug)]
struct OrderBook {
    orders: HashMap<Uuid, Order>,
    bids: BookSide,
    asks: BookSide,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum FillStatus {
    Full,
    Partial,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
struct Fill {
    order_id: Uuid,
    status: FillStatus,
    price: Decimal,
    quantity: Decimal,
}

#[derive(Debug)]
struct OrderResult {
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

impl OrderBook {
    pub fn new() -> OrderBook {
        return OrderBook {
            orders: HashMap::new(),
            bids: BookSide::new(),
            asks: BookSide::new(),
        };
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

                    println!("a fill");
                    println!("{:#?}", result);

                    order_result.done.extend(&result.done);
                    order_result.quantity_filled += result.quantity_filled;
                    quantity_left -= result.quantity_filled;
                }
            }
        }

        return order_result;
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

        println!("starting quantity {}", quantity_left);
        println!("price level {:#?}", price_level.borrow());

        while quantity_left > Decimal::zero() && price_level.borrow().len() > 0 {
            println!("quantity_left {}", quantity_left);

            let mut remove_id: Option<Uuid> = None;

            {
                let mut price_level = price_level.borrow_mut();
                if let Some(head) = price_level.front() {
                    if quantity_left < head.quantity {
                        println!("filling");

                        let mut o = head.clone();
                        o.quantity -= quantity_left;

                        price_level.replace_front(o);
                        self.orders.insert(o.id, o);

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

            println!("quantity left end {}", quantity_left);
            println!("price level len end {}", price_level.borrow().len());
        }

        return order_result;
    }

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
        println!("remove");
        println!("{}", id);
        println!("{:#?}", self.orders);
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

    println!("{:#?}", order_book);

    println!("Submitting market order...");

    let result = order_book.submit_market_order(Side::Bid, Decimal::new(2000, 2));

    println!("{:#?}", result);
    println!("{:#?}", order_book);
}
