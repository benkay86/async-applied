This example is part of a larger repository of examples, [async-applied](../README.md).

# indicatif-tokio

[indicatif](https://github.com/mitsuhiko/indicatif) is a crate for rendering progress bars in the terminal.  Indicatif does not support async/await syntax per se, but it does support concurrency well enough that you can use it to display the progress of async tasks.