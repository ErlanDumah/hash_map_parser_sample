# sample_Q2

## Assumptions and Things unsure

### Parsing the data received from the request

It is assumed this exercise is about writing a parser for the data, not optimizing the REST access to it or storing it after the fact. To this end I have created a parser that can accept the data received in the form of a standard rust `String`.


## Steps taken to arrive at the solution

The standard way of fetching from an http endpoint and then parsing json data would be to use a library like serde as a parser. In this exercise I presume the point would be defeated by using such a library.

As the assignment is about low latency as well as performance, I propose a parser that is made specifically for the use case. My parser would then support functions that limit the amount of data to be parsed in some way and resuming the parsing at a later time, allowing for in-between updates.

For the performance measurement I started with a simple `std::Instant` implementation to measure the time taken for a single entry to be parsed. This evaluated to about `60 µs`, which is of course with standard debug configuration.

Since performance seems to be the emphasis of this task, I additionally added a benchmark using the crate `criterion`. This benchmark can be found in `benches/bench_parser.rs`. I also refactored the code to use a `lib.rs` file to be able to use its functionality both in the main executable `main.rs` as well as with the benchmarking tool.

Using criterion, I was able to thoroughly test the performance of my "100 parse_single" test. To reproduce this you can call `cargo bench` from the `Q2` folder. The average execution time came out to be around `245 µs` per 100 entries. Parsing the entire data of one GET with its 1436 entries took `3.3384 ms` on average.

The next part would have been to use profiling. At this point I was running out of time and settled for mentioning the idea in this README.


## Correctness of the program

### Runtime analysis

The parser does not use any abstract language trees, rather a direct scan of the data given as characters. Thereby its runtime is O(n) where n is the size of data to be parsed.


### Optimisations for low latency

A few optimization opportunities come to mind:

 - Profiling of code with tools: This will let us break down which part of the code takes the most execution time, indicating where our bottlenecks are and what to work on.
 - Break down the one big request into smaller ones: Currently we are just requesting all available data instead of specifying the request for the respective use case.
 - Try and refactor the code to accept a byte array rather than a `String` and measure possible speedup: it could conceivably be that the algorithm can be sped up by directly accepting byte arrays rather than a `String` iterator.


### Other thoughts

 - The parser made for this exercise is of course an "incomplete" JSON parser, and it would throw an error with otherwise correct JSON. However the current use case seems to allow for this restriction. Obviously for production code you may want a little bit more robustness, rather than code that just stops parsing on the first key it does not recognise.

