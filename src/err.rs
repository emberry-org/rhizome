use std::{future::Future, io};

#[inline]
pub async fn eprinterr<T, OK>(result: T)
where
    T: Future<Output = io::Result<OK>>,
{
    if let Err(e) = result.await {
        eprintln!("{}", e);
    }
}

#[inline]
pub async fn eprinterr_with<T, OK>(result: T, context: &str)
where
    T: Future<Output = io::Result<OK>>,
{
    if let Err(e) = result.await {
        eprintln!("{}: {}", context, e);
    }
}
