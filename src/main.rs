use rust_decimal_macros::*;

use orderbook::order::Side;
use orderbook::OrderBook;

fn main() {
    let mut order_book = OrderBook::new();

    // Fill the book up with some orders.
    order_book.submit_limit_order(Side::Ask, dec!(10.01), dec!(50.00));
    order_book.submit_limit_order(Side::Ask, dec!(10.01), dec!(75.00));
    order_book.submit_limit_order(Side::Ask, dec!(10.00), dec!(75.00));
    order_book.submit_limit_order(Side::Ask, dec!(10.00), dec!(90.00));
    order_book.submit_limit_order(Side::Bid, dec!(10.01), dec!(45.00));

    println!("Submitting market order...");

    let result = order_book.submit_market_order(Side::Bid, dec!(20.00));

    println!("{:#?}", result);
    println!("{:#?}", order_book);

    println!("Submitting limit order...");

    let result = order_book.submit_limit_order(Side::Bid, dec!(20.00), dec!(76.00));

    println!("{:#?}", result);
    println!("{:#?}", order_book);
}
