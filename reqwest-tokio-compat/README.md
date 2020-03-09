This example is part of a larger repository of examples, [async-applied](../README.md).

# reqwest-tokio-compat

[reqwest](https://github.com/seanmonstar/reqwest) is an excellent crate for making HTTP requests in the vein of wget, curl, etc.  [tokio](https://tokio.rs) is the de facto Rust async runtime, especially for io-driven tasks.  This example builds on the [reqwest-tokio](../reqwest-tokio/README.md) example.  In that example we wondered why we could not call [`tokio::io::copy`](https://docs.rs/tokio/0.2.11/tokio/io/fn.copy.html) to copy the body of the [`reqwest::Response`](https://docs.rs/reqwest/0.10.1/reqwest/struct.Response.html) into `outfile`?

For [reasons related to good API design](https://github.com/seanmonstar/reqwest/issues/482) a `request::Response` does not implement `AsyncRead` directly.  It is, however, possible to convert it to a [`futures::io::Stream`](https://docs.rs/futures/0.3.4/futures/stream/trait.Stream.html) using [`bytes_stream()`](https://docs.rs/reqwest/0.10.1/reqwest/struct.Response.html#method.bytes_stream).  A `Stream` is naught but an async iterator, analagous to how we called `while let Some(chunk) = download.chunk().await?` in the [prior example](../reqwest-tokio/README.md).  It is therefore not surprising that there is an [stream extensions trait](https://docs.rs/futures/0.3.4/futures/stream/trait.TryStreamExt.html) that allows us to go from a stream to a [`futures::io::AsyncRead`](https://docs.rs/futures/0.3.4/futures/io/trait.AsyncRead.html).  Note that, because `AsyncRead` uses [`futures::io::Error`](https://docs.rs/futures/0.3.4/futures/io/struct.Error.html), we must [map](https://docs.rs/futures/0.3.4/futures/stream/trait.TryStreamExt.html#method.map_err) from [`request::Error`](https://docs.rs/reqwest/0.10.1/reqwest/struct.Error.html) in the process.

Having turned our `reqwest::Response` into an `AsyncRead` it should now be straightforward to invoke [`tokio::io::copy`](https://docs.rs/tokio/0.2.13/tokio/io/fn.copy.html)... but it is not.  For reasons described [here](https://www.reddit.com/r/rust/comments/enn3ax/strategies_for_futuresioasyncread_vs/) and [here](https://github.com/rust-lang/futures-rs/pull/1826), [`futures::io::AsyncRead`](https://docs.rs/futures/0.3.4/futures/io/trait.AsyncRead.html) is not compatible with [`tokio::io::AsyncRead`](https://docs.rs/tokio/0.2.11/tokio/io/trait.AsyncRead.html).  Bummer.  Fortunately there is a [compatibility layer between tokio and futures](https://docs.rs/tokio-util/0.3.0/tokio_util/compat/index.html)!

The compatibility layer is found in a separate crate, [tokio-util](https://github.com/tokio-rs/tokio/tree/master/tokio-util).  To use it we pull in the appropriate extension trait:

```
use tokio_util::compat::FuturesAsyncReadCompatExt;
```

And then simply call `compat()` in the `futures::io::AsyncRead` to make it into a `tokio::io::AsyncRead`.  Although it is annoying that futures and tokio have chosen not to use the same traits (at least for now), converting from one to the other really is not that hard.

> Note: To use the compatibility layer you will need to use tokio-util version 0.3 or greater in your `Cargo.toml`.