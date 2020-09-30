mod book_side;
mod order;
mod order_book;
mod price_level;

use rust_decimal_macros::*;
use std::time;

use order::{Order, Side};
use order_book::OrderBook;

fn main() {
    let mut order_book = OrderBook::new();

    let order1 = Order::new(Side::Ask, dec!(10.01), dec!(50.00), time::Instant::now());
    order_book.append(order1);

    let order2 = Order::new(Side::Ask, dec!(10.01), dec!(75.00), time::Instant::now());
    order_book.append(order2);

    let order3 = Order::new(Side::Bid, dec!(10.01), dec!(45.00), time::Instant::now());
    order_book.append(order3);

    println!("Submitting market order...");

    let result = order_book.submit_market_order(Side::Bid, dec!(20.00));

    println!("{:#?}", result);
    println!("{:#?}", order_book);
}
