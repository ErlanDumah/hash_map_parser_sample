# sample_Q2

Most assumptions and implementation details are described in src/main.rs.

## Correctness of the program

### Runtime analysis

The parser does not use any abstract language trees, rather a direct scan of the data given as characters. Thereby its runtime is O(n) where n is the size of data.

### Optimisations for low latency

Obiously the amount of data being parsed on each request is huge and provides significant delays of response if we were to parse all of it in one go. A few optimization opportunities come to mind:

 - Break down the one big request into smaller ones: Currently we are just requesting all available data instead of specifying the request for the respective use case.
 - Support a on-the-fly parsing algorithm: The parser always supports this: have the parser support "spoon-feeding" of batches of bytes and create single entities of parsed data on the fly instead of all of it in one go.

