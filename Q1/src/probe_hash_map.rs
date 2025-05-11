


// We define a struct Entry that will serve as the return type for first, last gets:
pub struct Entry<K, V> {
    pub key: K,
    pub value: V,
}

// Then we use a Storage struct that points to the previous and next element
// by the keys of either.
enum Storage<K, V> {
    UnOccupied,
    Occupied(Entry<K, V>),
    OccupiedDeleted,
}
struct Linkage {
    previous: Option<usize>,
    next: Option<usize>,
}
struct ProbeHashMapEntry<K, V> {
    storage: Storage<K,V>,
    linkage: Linkage,
}

/// Since we are using a fixed size hashtable, it can become full
/// In this case we want to return an error on trying to insert entries
#[derive(Debug)]
pub enum InsertionError {
    ContainerFull,
}

// Pretty printing for our InsertionError
impl std::fmt::Display for InsertionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            &InsertionError::ContainerFull => {
                write!(f, "The container is full.")
            },
        }
    }
}

// We create a implementation with a fixed size array given at initialization,
// and implement a standard hash table that utilizes this array as its storage.
// As a Hasher we are using the standard hasher and modify the result as the 
// remainder of our Size.
pub struct ProbeHashMap<K, V, const Size: usize> {
    random_state: std::hash::RandomState, // Use the standard hasher
    first_index: Option<usize>, // Key to least recent key-value pair inserted / updated
    last_index: Option<usize>, // Key to most recent key-value pair inserted / updated
    entry_array: Vec<ProbeHashMapEntry<K, V>>,
}

// Declaring a trait for convenience; you could simply impl ProbeHashMap as well
// I personally like doing it this way because it keeps the relevant signatures easy to read
// Notice the where K: Borrow<Q>; this is a nifty little trait requirement that allows us to
//  use &str types for our fetch functions rather than only &String types.

trait ProbeHashMapTrait<K, V> {
    /// Updates a currently existing value with key equal to given key, or
    /// alternatively creates a new entry if not yet existing.
    /// @return Ok(()) if insertion or update was successful, Err(InsertionError) otherwise
    fn insert(&mut self, key: K, value: V) -> Result<(), InsertionError>;
    /// Removes an entry with key equal to given key
    fn remove<Q>(&mut self, key: &Q)
    where K: std::borrow::Borrow<Q>, Q: std::hash::Hash + Eq + ?Sized;
    /// Returns the value of the entry with key equal to given key.
    /// @return None if no such entry was found, the value of the entry otherwise.
    fn get<Q>(&self, key: &Q) -> Option<&V>
    where K: std::borrow::Borrow<Q>, Q: std::hash::Hash + Eq + ?Sized;
    /// @return None if the map is empty, otherwise the last added or updated entry.
    fn get_last(&self) -> Option<&Entry<K, V>>;
    /// @return None if the map is empty, otherwise the most recent 
    fn get_first(&self) -> Option<&Entry<K, V>>;
}


impl<K, V> ProbeHashMapEntry<K, V> {
    pub fn new() -> Self {
        ProbeHashMapEntry { 
            storage: Storage::UnOccupied, 
            linkage: Linkage { 
                previous: None, 
                next: None, 
            } 
        }
    }
}


impl<K, V, const Size: usize> ProbeHashMap<K, V, Size> {
    pub fn new() -> Self {
        // Allocate vector with capacity in mind to avoid resizing
        let mut entry_array = Vec::with_capacity(Size);
        entry_array.resize_with(Size, || { return ProbeHashMapEntry::new(); });
        ProbeHashMap {
            random_state: std::hash::RandomState::new(),
            first_index: None,
            last_index: None,
            entry_array,
        }
    }
}

// Let's define some private functions for convenience
// For our helper functions we work with the resolution of keys, resulting hashes and indices of storage
enum FindResult {
    None,
    Entry(usize),
    UnOccupied(usize),
}

impl<K: std::hash::Hash + Eq, V, const Size: usize> ProbeHashMap<K, V, Size> {
    /// Calculates the hash of the given key, cropped to our storage size
    /// @return the hash of the key cropped to [0, Size - 1]
    fn hash<Q>(&self, key: &Q) -> usize
    where K: std::borrow::Borrow<Q>, Q: std::hash::Hash + Eq + ?Sized {
        use std::hash::{BuildHasher, Hasher};

        let mut state = self.random_state.build_hasher();
        key.hash(&mut state);
        let hash = state.finish();
        return hash as usize % Size;
    }

    /// Attempts to find an entry or alternatively an unoccupied space for given key
    /// @return Entry(index) if there was such an entry, Unoccupied(index) if there was an unoccupied space, None if the hashtable is full.
    fn find_entry_or_unoccupied<Q>(&self, key: &Q) -> FindResult
    where K: std::borrow::Borrow<Q>, Q: std::hash::Hash + Eq + ?Sized {
        let hash = self.hash(key);
        let mut index = hash;
        // Possible unoccupied entries from [hash, Size-1]
        while index < Size {
            match &self.entry_array[index].storage {
                &Storage::UnOccupied => return FindResult::UnOccupied(index),
                &Storage::Occupied(ref entry) => {
                    if entry.key.borrow() == key {
                        return FindResult::Entry(index);
                    }
                },
                _ => continue,
            }
            index+=1;
        }

        // Possible unoccupied entries from [0, hash - 1]
        index = 0;
        while index < hash {
            match &self.entry_array[index].storage {
                &Storage::UnOccupied => return FindResult::UnOccupied(index),
                &Storage::Occupied(ref entry) => {
                    if entry.key.borrow() == key {
                        return FindResult::Entry(index);
                    }
                },
                _ => continue,
            }
            index+=1;
        }

        return FindResult::None;
    }

    fn find_index_of<Q>(&self, key: &Q) -> Option<usize>
    where K: std::borrow::Borrow<Q>, Q: std::hash::Hash + Eq + ?Sized {
        let hash = self.hash(key);
        let mut index = hash;
        // Possible entries from [hash, Size-1]
        while index < Size {
            match &self.entry_array[index].storage {
                &Storage::Occupied(ref entry) 
                  => { if entry.key.borrow() == key { return Some(index) } },
                &Storage::UnOccupied => return None,
                _ => {},
            }
            index+=1;
        }

        // Possible entries from [0, hash - 1]
        index = 0;
        while index < Size {
            match &self.entry_array[index].storage {
                &Storage::Occupied(ref entry) 
                  => { if entry.key.borrow() == key { return Some(index) } },
                &Storage::UnOccupied => return None,
                _ => {},
            }
            index+=1;
        }

        return None;
    }

    fn find_entry<Q>(&self, key: &Q,) -> Option<&Entry<K, V>>
    where K: std::borrow::Borrow<Q>, Q: std::hash::Hash + Eq + ?Sized {
        let hash = self.hash(key);
        let mut index = hash;
        // Possible entries from [hash, Size-1]
        while index < Size {
            match &self.entry_array[index].storage {
                &Storage::Occupied(ref entry) 
                  => { if entry.key.borrow() == key { return Some(entry) } },
                &Storage::UnOccupied => return None,
                _ => {},
            }
            index+=1;
        }

        // Possible entries from [0, hash - 1]
        index = 0;
        while index < Size {
            match &self.entry_array[index].storage {
                &Storage::Occupied(ref entry) 
                  => { if entry.key.borrow() == key { return Some(entry) } },
                &Storage::UnOccupied => return None,
                _ => {},
            }
            index+=1;
        }

        return None;
    }

    fn unlink(&mut self, index: usize) {
        // Cloning linkage as to avoid two mut references from existing at the same time
        let previous = self.entry_array[index].linkage.previous.clone();
        let next = self.entry_array[index].linkage.next.clone();

        if let Some(previous_index) = previous {
            self.entry_array[previous_index].linkage.next = next;
        } 
        else { // entry was first entry
            self.first_index = self.entry_array[index].linkage.next;
        }

        if let Some(next_index) = next {
            self.entry_array[next_index].linkage.previous = previous;
        }
        else { // entry was last entry
            self.last_index = self.entry_array[index].linkage.previous;
        }

        self.entry_array[index].linkage.previous = None;
        self.entry_array[index].linkage.next = None;
    }

    fn link_as_last(&mut self, index: usize) {
        if let Some(previous_last_index) = self.last_index {
            self.entry_array[index].linkage.previous = Some(previous_last_index);
            self.entry_array[previous_last_index].linkage.next = Some(index);
        }
        else {
            self.entry_array[index].linkage.previous = None;
        }
        self.entry_array[index].linkage.next = None;

        self.last_index = Some(index);
    }

    fn insert_at_index(&mut self, index: usize, key: K, value: V) {
        self.entry_array[index].storage = Storage::Occupied(Entry{key, value});
        if self.first_index.is_none() {
            self.first_index = Some(index);
        }
        self.link_as_last(index);
    }

    fn update_at_index(&mut self, index: usize, value: V) {
        if let &mut Storage::Occupied(ref mut entry) = &mut self.entry_array[index].storage {
            entry.value = value;
        }
        self.unlink(index);
        self.link_as_last(index);
    }

    fn remove_at_index(&mut self, index: usize) {
        self.unlink(index);
        self.entry_array[index].storage = Storage::OccupiedDeleted;
    }

    //}
//impl<K: std::hash::Hash, V, const Size: usize> ProbeHashMapTrait<K, V> for ProbeHashMap<K, V, Size> {

    pub fn insert(&mut self, key: K, value: V) -> Result<(), InsertionError> {
        // Find unoccupied index starting at hash value
        match self.find_entry_or_unoccupied(&key) {
            FindResult::None => return Err(InsertionError::ContainerFull),
            FindResult::Entry(index) => self.update_at_index(index, value),
            FindResult::UnOccupied(index) => self.insert_at_index(index, key, value),
        };

        return Ok(());
    }

    pub fn remove<Q>(&mut self, key: &Q)
    where K: std::borrow::Borrow<Q>, Q: std::hash::Hash + Eq + ?Sized {
        match self.find_index_of(key) {
            Some(index) => self.remove_at_index(index),
            None => {},
        };
    }
    
    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where K: std::borrow::Borrow<Q>, Q: std::hash::Hash + Eq + ?Sized {
        match self.find_entry(key) {
            None => return None,
            Some(entry) => {
                return Some(&entry.value);
            }
        };
    }
    
    pub fn get_last(&self) -> Option<&Entry<K, V>> {
        let index = match &self.last_index {
            &None => return None,
            &Some(index) => index,
        };

        let entry = match &self.entry_array[index].storage {
            &Storage::OccupiedDeleted => {
                assert!(false, "Undefined behaviour: last_index pointed to a deleted entry");
                return None;
            },
            &Storage::UnOccupied => {
                assert!(false, "Undefined behaviour: last_index pointed to an unoccupied entry");
                return None;
            }
            &Storage::Occupied(ref entry) => entry,
        };

        return Some(entry);
    }
    
    pub fn get_first(&self) -> Option<&Entry<K, V>> {
        let index = match &self.first_index {
            &None => return None,
            &Some(index) => index,
        };

        let entry = match &self.entry_array[index].storage {
            &Storage::OccupiedDeleted => {
                assert!(false, "Undefined behaviour: first_index pointed to a deleted entry");
                return None;
            },
            &Storage::UnOccupied => {
                assert!(false, "Undefined behaviour: first_index pointed to an unoccupied entry");
                return None;
            }
            &Storage::Occupied(ref entry) => entry,
        };

        return Some(entry);
    }
}