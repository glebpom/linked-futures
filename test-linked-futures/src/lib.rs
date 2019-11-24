#[cfg(test)]
mod tests {
    use linked_futures::{link_futures, linked_block};

    use futures::channel::oneshot;
    use futures::executor::block_on;

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
}
