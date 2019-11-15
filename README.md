# linked-futures

This crate provides the way to "link" futures into a single block,
which stops executing once any of these futures complete.

Under the hood, it uses `FuturesUnordered` to execute multiple futures efficiently.
To avoid boxing, custom `one-of` type from [`one-of-futures`](https://crates.io/crates/one-of-futures)
crate is generated for each [`link_futures`](macro.link_futures.html) block.
