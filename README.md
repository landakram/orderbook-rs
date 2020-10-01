# orderbook-rs 📖

> An in-memory limit order book and matching engine in Rust.

⚠ **Experimental. Unit tested but never deployed in prod. Use at your own risk.**

## Features

* Market and limit orders 
* Order cancellation
* Price-time priority

## Usage

The order book may be instantiated with `OrderBook::new()`:

```rust
let mut order_book = OrderBook::new();
```

Orders may be submitted with `submit_limit_order` and `submit_market_order`. 
These methods return a struct, `OrderResult`, containing any fills. If the call
results in a resting order on the book, the resting order can be found in 
`OrderResult.partial`.

Example:

```rust
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
```

Time in Force is not implemented, and limit orders are effectively submitted as
Good Till Canceled.

## Motivation

I wrote this to familiarize myself with Rust. The implementation is similar to 
[this orderbook](https://github.com/i25959341/orderbook) for golang, with some minor
differences in naming and the semantics of return values.
