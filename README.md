A test support library to exacerbated contention in the tested code.

#   Goals

-   Help fuzzing multi-threaded code by exarcebating contention.
-   Help benchmark multi-threaded code under heavy contention.


#   Motivation

When developing multi-threaded code, two questions typically arise:

-   Is it correct?
-   Is it fast enough?

Unfortunately, while it may be possible to exhaustively test all code-paths of serial code, testing all potential
interleavings of concurrent code is not possible without extensive cooperation from the code under test[^1]. Similarly,
benchmarking concurrent code can be quite difficult, _especially measuring latency_.

This is the raison d'etre of `Bursty`: helping crafting tests, and benchmarks, as a serie of steps in a pipeline where
all concurrent threads execute each step in lock-step, thereby maximizing contention at every step.

[^1] _See [`loom`](https://crates.io/crates/loom) for an interesting approach._


#   How does it work?

`Bursty` is a framework for running a pipeline of steps, in lock-step, across a user-configured number of threads, while
accessing both globally shared and thread-local state.

A quick demo -- lifted from `BurstyBuilder` documentation:

```rust
use std::sync::atomic::{AtomicI32, Ordering};
use bursty::BurstyBuilder;

//  Construct a builder with a globally-shared `AtomicI32` and two thread-local "states" (1 and 10).
let mut builder = BurstyBuilder::new(AtomicI32::new(0), vec!(1, 10));

//  Add a step in the pipeline to execute on each thread, which adds the local integer to the global one.
builder.add_simple_step(|| |global: &AtomicI32, local: &mut i32| { global.fetch_add(*local, Ordering::Relaxed); });

//  Launch the execution, with 4 iterations of the pipeline.
let mut bursty = builder.launch(4);

assert!(44 >= bursty.global().load(Ordering::Relaxed));

//  Join all threads, blocking until they are done, or one panicks.
bursty.join();

//  Check the global and local states.
assert_eq!(44, bursty.global().load(Ordering::Relaxed));
assert_eq!(vec!(1, 10), bursty.into_locals());
```

Of import, note that there's no obligation for a user-defined step to have the same behavior on all threads. The step
could perfectly add odd numbers and subtract even ones, for example. This allows testing mixed work loads.


#   How is the execution in lock-step achieved?

By spinning.

Between each step to execute, `Bursty` injects a rendez-vous, a simple atomic integer initialized with the number of
threads to wait for. When reaching this atomic, each thread decrements it, then spins until the atomic is 0, at which
points it proceeds.

This is the most accurate way to ensure that all threads start executing the following step at near the same time,
though does come with the caveat of being extremely CPU intensive.

It is typically not a good idea to run more threads than physical cores on the machine.


#   How to fuzz-test?

For fuzz-testing with Bursty, simply create the pipeline to test and crank up the number of iterations depending on how
much time execution takes and how long you are willing to wait. Each iteration is likely to have a slightly different
intervealing than the others, and over time more and more interleavings will be tested... though there's no magic number
that will guarantee that all will be tested.


#   How to benchmark?

Pick a benchmarking framework which allows the user to report the measurement, then create a pipeline. For efficiency,
it is recommended to create a self-resetting pipeline which can be executed multiple times back-to-back, and to schedule
multiple iterations at a time, so that there's no need to create N threads every time.


#   Similar Projects

I am not aware of any project allowing benchmarking latency under heavy contention.

For testing correctness, as noted, [`loom`](https://crates.io/crates/loom) may be of interest. In particular, `loom` is
more likely to provide a _definitive_ answer, whereas the fuzzing approach exhibited only boosts confidence.
