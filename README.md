# linked-futures

## Overview

This crate provides the way to "link" futures into a single block,
which stops executing once any of these futures complete.

Under the hood, it uses [`FuturesUnordered`](https://docs.rs/futures/0.3.1/futures/stream/struct.FuturesUnordered.html)
to execute multiple futures efficiently. In order to avoid boxing, custom `one-of` type from
[`one-of-futures`](https://crates.io/crates/one-of-futures) crate is generated for
each [`link_futures`](macro.link_futures.html) block.

License: MIT

## Usage

Add this to your `Cargo.toml`:
```toml
[dependencies]
linked-futures = "0.1"
```

## Example
```rust
use std::time::{Duration, Instant};

use futures::{pin_mut, SinkExt, StreamExt};
use futures::channel::mpsc;
use futures::executor::block_on;
use tokio::clock;
use tokio::timer::{delay, Interval};

use linked_futures::{link_futures, linked_block};

linked_block!(PeriodicStoppableSender, PeriodicStoppableSenderFutreIdentifier; Forwarder, Reader, Generator, Stop);

#[tokio::main]
async fn main() {
    let (mut tx1, mut rx1) = mpsc::channel::<Instant>(1);
    let (mut tx2, mut rx2) = mpsc::channel::<Instant>(1);
    let generator = async {
        while let Some(instant) = Interval::new(clock::now(), Duration::from_millis(100)).take(1).next().await {
            tx1.send(instant).await;
        }
    };
    let forwarder = async {
        while let Some(instant) = rx1.next().await {
            tx2.send(instant).await;
        }
    };
    let reader = async {
        while let Some(instant) = rx2.next().await {
            println!("instant: {:?}", instant);
        }
    };
    let stop = async { delay(clock::now() + Duration::from_secs(1)).await; };
    let linked = link_futures!(PeriodicStoppableSender, PeriodicStoppableSenderFutreIdentifier;
       Generator => generator,
       Forwarder => forwarder,
       Reader => reader,
       Stop => stop
   );
    block_on(async {
        pin_mut!(linked);
        let (completed_future_identifier, _) = linked.await;
        match completed_future_identifier {
            PeriodicStoppableSenderFutreIdentifier::Stop => println!("linked block stopped normally"),
            n => panic!("linked block unexpectedly terminated by future: {:?}", n),
        }
    });
}
```