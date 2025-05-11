# sample_Q2

Most assumptions and implementation details are described in src/main.rs.

## Steps taken to arrive at the solution

The standard way of fetching from an http endpoint and then parsing json data would be to use a library like serde as a parser. That library would allow for parsing through simple `#[derive]` statements. However, standard simple parsing practices normally assume that performance of parsing is not the bottleneck in your code and not a requirement.

As the assignment is about low latency as well as performance, I propose a parser that is made specifically for the use case. Instead of using a library that would parse the complete data at once, my parser would then support functions that limit the amount of data to be parsed in some way and resuming the parsing at a later time, allowing for in-between updates.

For the performance measurement I started with a simple `std::Instant` implementation to measure the time taken for a single entry to be parsed.


## Correctness of the program

### Runtime analysis

The parser does not use any abstract language trees, rather a direct scan of the data given as characters. Thereby its runtime is O(n) where n is the size of data.

### Optimisations for low latency

Obviously the amount of data being parsed on each request is huge and provides significant delays of response if we were to parse all of it in one go. A few optimization opportunities come to mind:

 - Break down the one big request into smaller ones: Currently we are just requesting all available data instead of specifying the request for the respective use case.
 - Support a on-the-fly parsing algorithm: The parser always supports this: have the parser support "spoon-feeding" of batches of bytes and create single entities of parsed data on the fly instead of all of it in one go.

