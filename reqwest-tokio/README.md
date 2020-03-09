This example is part of a larger repository of examples, [async-applied](../README.md).

# reqwest-tokio

[reqwest](https://github.com/seanmonstar/reqwest) is an excellent crate for making HTTP requests in the vein of wget, curl, etc.  [tokio](https://tokio.rs) is the de facto Rust async runtime, especially for io-driven tasks.  This example demonstrates the simplest possible use of these two crates together to download a picture of [the Rust mascot, Ferris](https://rustacean.net/).

If you have done much reading in the [tokio documentation](https://docs.rs/tokio) you may wonder why we use a `while let` loop to drive the download in "chunks" rather than calling [`tokio::io::copy`](https://docs.rs/tokio/0.2.13/tokio/io/fn.copy.html)?  The answer is in the [reqwest-tokio-compat](../reqwest-tokio-compat/README.md) example.