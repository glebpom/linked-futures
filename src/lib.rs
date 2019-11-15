#![doc(html_root_url = "https://docs.rs/linked-futures/0.1.1")]
#![warn(
    missing_debug_implementations,
    rust_2018_idioms,
    unreachable_pub
)]
#![deny(intra_doc_link_resolution_failure)]

//! This crate provides the way to "link" futures into a single block,
//! which stops executing once any of these futures complete.
//!
//! Under the hood, it uses [`FuturesUnordered`](https://docs.rs/futures/0.3.1/futures/stream/struct.FuturesUnordered.html)
//! to execute multiple futures efficiently. In order to avoid boxing, custom `one-of` type from
//! [`one-of-futures`](https://crates.io/crates/one-of-futures) crate is generated for
//! each [`link_futures`](macro.link_futures.html) block.

/// Create necessary enums for later usage with [`link_futures`](macro.link_futures.html)
#[macro_export]
macro_rules! linked_block {
    ( $one_of_block:ident, $identifier_enum:ident; $($variants:ident),* ) => {
        one_of_futures::impl_one_of!($one_of_block; $($variants),*);

        #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
        enum $identifier_enum {
            $($variants),*
        }
    }
}

/// Link multiple futures into a single block
///
/// Example:
/// ```rust
/// use std::time::{Duration, Instant};
///
/// use futures::{pin_mut, SinkExt, StreamExt};
/// use futures::channel::mpsc;
/// use futures::executor::block_on;
/// use tokio::clock;
/// use tokio::timer::{delay, Interval};
///
/// use linked_futures::{link_futures, linked_block};
///
/// linked_block!(PeriodicStoppableSender, PeriodicStoppableSenderFutreIdentifier;
///     Forwarder,
///     Reader,
///     Generator,
///     Stop
/// );
///
/// #[tokio::main]
/// async fn main() {
///     let (mut tx1, mut rx1) = mpsc::channel::<Instant>(1);
///     let (mut tx2, mut rx2) = mpsc::channel::<Instant>(1);
///
///     let mut interval = Interval::new(clock::now(), Duration::from_millis(100));
///
///     let generator = async {
///         while let Some(instant) = interval.next().await {
///             tx1.send(instant).await;
///         }
///     };
///     let forwarder = async {
///         while let Some(instant) = rx1.next().await {
///             tx2.send(instant).await;
///         }
///     };
///     let reader = async {
///         while let Some(instant) = rx2.next().await {
///             println!("instant: {:?}", instant);
///         }
///     };
///     let stop = async {
///         delay(clock::now() + Duration::from_secs(1)).await;
///     };
///     let linked = link_futures!(
///        PeriodicStoppableSender,
///        PeriodicStoppableSenderFutreIdentifier;
///        Generator => generator,
///        Forwarder => forwarder,
///        Reader => reader,
///        Stop => stop
///     );
///     block_on(async {
///         pin_mut!(linked);
///         let (completed_future_identifier, _) = linked.await;
///         match completed_future_identifier {
///             PeriodicStoppableSenderFutreIdentifier::Stop =>
///                 println!("linked block stopped normally"),
///             n =>
///                 panic!("linked block unexpectedly terminated by future: {:?}", n),
///         }
///     });
/// }
/// ```
#[macro_export]
macro_rules! link_futures {
    ( $one_of_block:ident, $identifier_enum:ident; $( $key:ident => $value:expr ),* ) => {{
        let mut linked = ::futures::stream::FuturesUnordered::new();
        $( linked.push($one_of_block::$key(async {
            ($identifier_enum::$key, $value.await)
        })); )*
        async move {
            use futures::stream::StreamExt;

            linked.next().await.unwrap()
        }
    }};
}

#[cfg(test)]
mod tests {
    use futures::channel::oneshot;
    use futures::executor::block_on;

    use super::*;

    linked_block!(SimpleBlock, SimpleBlockFutureIdentifier; Never, Stop);

    #[test]
    fn it_works() {
        let (_tx, rx) = oneshot::channel::<()>();
        let block = link_futures!(SimpleBlock, SimpleBlockFutureIdentifier;
            Never => async {
                let _ = rx.await;
            },
            Stop => async { }
        );
        let (stopped_future_name, _) = block_on(async { block.await });
        assert_eq!(stopped_future_name, SimpleBlockFutureIdentifier::Stop);
    }

    #[test]
    fn test_readme_deps() {
        version_sync::assert_markdown_deps_updated!("README.md");
    }

    #[test]
    fn test_html_root_url() {
        version_sync::assert_html_root_url_updated!("src/lib.rs");
    }
}
