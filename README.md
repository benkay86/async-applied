# Async Applied

The Rust programming language supports the powerful concurrency concept of _tasks_, non-blocking computations that model cooperative concurrency.  Tasks are implemented in rust using [futures](https://doc.rust-lang.org/std/future/trait.Future.html), [async](https://doc.rust-lang.org/std/keyword.async.html), and [await](https://doc.rust-lang.org/std/keyword.await.html).  These features were [stabilized quite recently](https://blog.rust-lang.org/2019/11/07/Async-await-stable.html) on 11/7/2019.  Consequently, the ecosystem of crates that depend on them is still very much in flux, and there are not a lot of examples of how to use async/await in practice.

This repository is _not_ intended as yet another tutorial or primer on async/await.  This repository _is_ intended to showcase examples demonstrating practical use of async/await applied to common, real-world problems.  You should familiarize yourself with async/await by reading one of the [excellent](https://rust-lang.github.io/async-book/01_getting_started/01_chapter.html) [existing](https://book.async.rs/) [tutorials](https://tokio.rs/docs/getting-started/hello-world/) before you attempt the examples herein.

Please note that this repository is a work in progress.  Examples may change as the underlying ecosystem changes.  I intend to add more examples as time allows.  You are welcome to contribute to examples to or steal examples from this repository.

## Examples

This repository contains examples of practical async code use that you can download, build, and experiment with.  There are admittedly not many examples right now, but I hope to gradually add more as time permits.

* [reqwest-tokio](./reqwest-tokio/README.md) download a file using [reqwest](https://github.com/seanmonstar/reqwest) and [tokio](https://tokio.rs).
* [reqwest-tokio-compat](./reqwest-tokio-compat/README.md) download a file using [reqwest](https://github.com/seanmonstar/reqwest) and [tokio](https://tokio.rs) using [`tokio_util::compat`](https://github.com/tokio-rs/tokio/blob/master/tokio-util/src/compat.rs) to harmonize traits between [futures](https://github.com/rust-lang/futures-rs) and tokio.

## Async is Not Threads

For those who inevitably are not going to familiarize themselves with async/await before reading further, it is important to understand that **async is not threads**.

Async is _related to_ threads.  When most programmers think of concurrency they think of threads.  Threads are lightweight processes managed by the operating system that share memory and may run concurrently on multiple physical processing units (i.e. cpu cores).  Async, await, and futures are about _tasks_.  Tasks are abstract units of computation that may be run concurrently.  Tasks may be scheduled to run on different threads, or they may run interleaved on the same thread, or some combination of the above.

The most important distinction between _tasks_ and _threads_ is that tasks written with **async should not block**.  Underneath the hood, tasks (modeled as [futures](https://doc.rust-lang.org/std/future/trait.Future.html)) implement a method `poll()`.  Each time `poll()` is called the task performs a little bit of computation.  Something called the _runtime_ will call `poll()` repeatedly until the task completes all of its computations.  Each call to `poll()` will take some small amount of time to return, but the task should not _block_ other tasks by making `poll()` take a really long time to return.

For example, if writing an async task to compute the first one million numbers in the fibonnaci sequence, each call to `poll()` should perform just a little bit of the computation (e.g. compute just the next number).  The task should not block by having `poll()` compute all one million numbers all at once.

**If you write tasks that block your async programs will not behave as expected.**  The Rust compiler cannot know if your task is going to block or not, so it is up to you the programmer to write non-blocking tasks.  If you do not understand the difference between tasks and threads, or if you do not understand the concept of an asynchronous, non-blocking task, **stop**.  Go back and read an async/await tutorial before you proceed.

## The Async Ecosystem

Because you have familiarized yourself with async/await by reading other tutorials, you know that async/await syntax requires the use of a runtime (a.k.a. executor) to schedule async tasks and drive them to completion.  You also know that writing an async/await task that performs io (input/output) is impossible with the primitives in [std::io](https://doc.rust-lang.org/std/io/) because the standard library primitives block.  Therefore, in addition to a runtime, any non-trivial async task that performs io needs non-blocking, asynchronous io primitives.  At the time of this writing _none_ of these features are provided by the Rust standard library.  They all exist in crates within the async ecosystem.

### The Vocabulary

At the bottom of the async ecosystem is a set of traits that serve as a common vocabulary to describe how the different parts of the ecosystem interact with each other.  Currently these are in:

* [`std::future`](https://doc.rust-lang.org/std/future), the parts of the async ecosystem that have been stabilized, most notably:
	- [`Future`](https://doc.rust-lang.org/std/future/trait.Future.html), the fundamental trait describing an asynchronous unit of computation.
* [futures](https://github.com/rust-lang/futures-rs) 0.3, a crate that defines all the traits which _haven't_ been stabilized... yet.  Expect that many of the traits in futures will eventually find their way into `std::future`.  These include:
	- [`AsyncRead`](https://docs.rs/futures/0.3.4/futures/io/trait.AsyncRead.html), an asynchronous version of [`std::io::Read`](https://doc.rust-lang.org/std/io/trait.Read.html).
	- [`AsyncRead`](https://docs.rs/futures/0.3.4/futures/io/trait.AsyncWrite.html), an asynchronous version of [`std::io::Write`](https://doc.rust-lang.org/std/io/trait.Write.html).
	- [`Stream`](https://docs.rs/futures/0.3.4/futures/stream/trait.Stream.html), an asynchronous analog to [`std::iter::Iterator`](https://doc.rust-lang.org/std/iter/trait.Iterator.html).

Note that in the beginning, before async/await was stabilized, the [futures](https://github.com/rust-lang/futures-rs) crate defined everything async and even provided macros like `await!` that are mostly not needed now that async and await have been stabilized.  To avoid encountering such fossils you should make sure to use futures version 0.3 or greater.

### The Foundation

You are free to implement the async traits yourself, and in fact it is not that hard to implement a `Future` to perform an asynchronous computation (e.g. fibbonaci sequence).  Asynchronous input/output (io) operations are much more complicated to implement because they require support from the operating system.  The foundation of async is a set of io-related crates upon which the rest of the async ecosystem rests.  They are important and so you should know they exist -- but it is unlikely you will ever need to use them directly.

* [mio](https://github.com/tokio-rs/mio) interfaces with the operating system to define asynchronous versions of file, socket, etc primitives.
* [hyper](https://github.com/hyperium/hyper) is an HTTP protocol implementation with an asynchronous design.

### The Building Blocks

The foundational crates above are quite primitive, and it would not be very ergonomic to use them directly.  The following crates build upon that foundation to implement traits like `AsyncRead` on, for example, a file.  They define the modules and structures you will interact with most often in the async ecosystem.  Note that some of these crates are listed again under Runtimes below.

* [async-std](https://async.rs) 1.5 implements async traits on files, sockets, etc.  It aims to be easy to learn and hew closely to the the style of its synchronous counterparts in `std::io`.  It is compatible with all the foundational traits in [futures](https://github.com/rust-lang/futures-rs).
* [tokio](https://tokio.rs) 0.2 implements async traits on files, sockets, etc, making it very similar to async-std.  Although it is "only" version 0.2, tokio has actually been around for longer than async-std, is used in more projects, and is arguably more stable.  However, the async ecosystem is young and it's not clear which crate is "better."  One current disadvantage of using tokio is that [`tokio::io::AsyncRead`](https://docs.rs/tokio/0.2.13/tokio/io/trait.AsyncRead.html) are not compatible with [`futures::io::AsyncRead`](https://docs.rs/futures/0.3.4/futures/io/trait.AsyncRead.html) et al, although there is a compatibility layer.  The reasons for this difference are summarized [here](https://www.reddit.com/r/rust/comments/enn3ax/strategies_for_futuresioasyncread_vs/) and [here](https://github.com/rust-lang/futures-rs/pull/1826).
* [tokio-util](https://github.com/tokio-rs/tokio/tree/master/tokio-util) contains some additional functionality built around tokio.  As of version 0.3.0 it includes a [compatibility layer between tokio and futures](https://docs.rs/tokio-util/0.3.0/tokio_util/compat/index.html).
* [reqwest](https://github.com/seanmonstar/reqwest) is a crate for async HTTP/TLS (think curl, wget, etc).  Don't let the strange name fool you; it is the best and only choice for async web requests.
* [warp](https://github.com/seanmonstar/warp) is like reqwest but for the server side.
* [rusoto](https://github.com/rusoto/rusoto) 0.43 is an async API for Amazon Web Services (AWS) including S3.  You will need at least version 0.43, which is [currently beta](https://linuxwit.ch/blog/2020/02/the-future-of-rusoto/).

### The Runtimes

The runtime (also called executor or reactor) is what schedules your async tasks and drives them to completion.  Although it may seem counterintuitive at first, just as there are multiple building blocks that implement the same trait different use cases (e.g. file access vs web access), there are also multiple runtimes with slightly different performance objective and use cases.  These include:

* `std::runtime` -- just kidding!  There is no runtime in the Rust standard library.  That's right, even though async/await are stabilized you will still need to use one of the runtime crates below to actually run any tasks concurrently.  This is because, as per above, there is no unifying runtime use case or obviously best runtime to be standardized.
* [async-std](https://async.rs) 1.5 is a runtime bundled with async-std.  On the backend, the runtime has to keep track of the state of its various io operations to work efficiently, so it makes sense for the runtime and io drivers to live in the same crate. 
* [tokio](https://tokio.rs) 0.2 is a runtime bundled with tokio.  You can use it with futures-compatible traits via [`tokio_util::compat`](https://github.com/tokio-rs/tokio/blob/master/tokio_util/src/compat.rs).
* [futures](https://github.com/rust-lang/futures-rs) provides its own runtime too, but at least for now it is meant more for trivially simple tasks and internal testing.  You will likely want to pick one of async-std or tokio to use in production code.
* [rayon](https://github.com/rayon-rs/rayon) is **not** part of the async ecosystem but deserves honorable mention for being an excellent thread scheduling pool for implementing parallel iterators on CPU-bound, blocking computations.

### The High-Level Stuff

As Rusts's async ecosystem matures we are starting to see high-level frameworks built on top of tokio and other mid-level crates.  Chief among these is the appropriately-named:

* [tower](https://github.com/tower-rs/tower), a network services framework.

## License

This project is licensed under either of

* Apache License, Version 2.0, (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)

at your option.