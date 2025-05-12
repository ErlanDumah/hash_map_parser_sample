# sample_Q1

Most assumptions and implementation details are described in src/main.rs.

## Assumptions and Things unsure

### Fixed size hashmap

I implemented the code with a "fixed size" storage in mind as I believed that to be part of the requirements. As a result the hashmap does not extend itself automatically and an insertion can cause an error for lack of space.

The solution could be updated with a possible extension of space in mind: once the amount of occupied entries reaches a certain threshold in relation to its current size, we re-allocate a Vec with double the size and re-insert every currently occupied entry for the new storage. This would under circumstances also ensure that the hashmap related functionality stays O(1) by preventing the hashmap storage occupied to come close to its size.


## Steps taken to arrive at the solution

The driver for the solution chosen are the runtime requirements: As hashmaps typically already involve O(1) for lookups, insertions and removals I was not too worried about that. The O(1) requirement for first and last entries was more interesting:

For the program to be correct, first and last entries must remain stable even through removal, update and possible re-insertion of entries, so a simple memorisation of just the first element and last element don't suffice. In short, there must be a storage of a list that has O(1) for first, last lookup as well as removal of elements. This is where the idea of a "linked list" came to mind as the runtimes match perfectly for it.

As we are coding in rust, we're not going to try and utilize pointer magic as other languages would probably do. Instead, the idea is to "simulate" a linked list by having each (occupied) entry contain information for its state in the linked list. From thereon, all we need to do is ensure that the order of the list gets preserved through mutating operations of the hashmap collection. As the requirements seem to suggest that `get_mut` is not supported and instead `insert` is also used for updates, this seems like a good approach.


## Correctness of the program

### Runtime analysis

Functions `get`, `insert`, `remove` can be argued by amortisation to be O(1). For each one of them the worst case is a scan of the entire fixed size storage array. However this case happens only as the hashtable fills up. It can be assumed given reasonable size that the lookup does not scan the entire hashmap but at most a fraction of it. See assumptions as to why I did not implement a solution where this is ensured.

Functions `get_first`, `get_last` are both a lookup of an array entry per index, thereby O(1).


### Test cases

The test cases used are meant as ensurance that the functions implemented fulfill the requirements given, and as an easy way of debugging the code. In particular use cases where the first and last entry are manipulated were important to ensure correctness of the code.
