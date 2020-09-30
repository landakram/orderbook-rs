mod book_side;
mod order;
mod order_book;
mod price_level;

use rust_decimal::prelude::*;
use std::time;

use order::{Order, Side};
use order_book::OrderBook;

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

    println!("Submitting market order...");

    let result = order_book.submit_market_order(Side::Bid, Decimal::new(2000, 2));

    println!("{:#?}", result);
    println!("{:#?}", order_book);
}
