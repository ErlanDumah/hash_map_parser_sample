
// Assumptions:
// A solution utilizing std HashMap collections is not desired
// The solution describes a fixed size hashmap. I am assuming that to mean no 
//  automatic expansion of internal storage. The solution given could be extended
//  with an automatic expansion algorithm that inserts all of the current entries into
//  the new storage array.

// Approach:
// A hashmap implementation where we have a "linked list" like structure built in:
// Every entry also features a link to its previous and next element, and on insert
//  we fix the links of the updated entry and the previous last entry
// This way we an support O(1) fetch of first and last elements


mod probe_hash_map;
use probe_hash_map::ProbeHashMap;


fn main() {
    let file_path = "./assets/98-0.txt";

    // We are of course reading the whole file here as opposed to using a stream
    // I'm assuming here optimizing the reading of the file is not the point of the exercise
    println!("Hello");
    let file = match std::fs::read_to_string(file_path) {
        Ok(file) => file,
        Err(error) => {
            println!("Reading the asset file failed: {}", error);
            return;
        }
    };

    let mut hash_map: ProbeHashMap<String, u32, 100000> = ProbeHashMap::new();

    let mut count = 0;
    file.split_whitespace().enumerate().for_each(|(index, word)| {
        match hash_map.insert(String::from(word), index as u32) {
            Ok(()) => {
                count+=1;
            },
            Err(insertion_error) => {
                println!("Error at insertion of word {} at index {}: {}", word, index, insertion_error);
            }
        }
    });

    println!("Finished insertion of {} word entries.", count);
}


#[cfg(test)]
mod tests {
    use crate::ProbeHashMap;

    // A nifty little macro that allows us to write one-line asserts
    macro_rules! matches(
        ($e:expr, $p:pat) => (
            match $e {
                $p => true,
                _ => false
            }
        )
    );

    #[test]
    fn insert_works() {
        let mut hash_map: ProbeHashMap<String, i32, 200> = ProbeHashMap::new();

        assert!(hash_map.insert(String::from("abc"), 5).is_ok());

        assert!(matches!(hash_map.get("abc"), Some(5)));
    }

    #[test]
    fn update_works() {
        let mut hash_map: ProbeHashMap<String, i32, 200> = ProbeHashMap::new();

        assert!(hash_map.insert(String::from("abc"), 5).is_ok());
        assert!(hash_map.insert(String::from("abc"), 10).is_ok());

        assert!(matches!(hash_map.get("abc"), Some(10)));
    }

    #[test]
    fn remove_works() {
        let mut hash_map: ProbeHashMap<String, i32, 200> = ProbeHashMap::new();

        assert!(hash_map.insert(String::from("abc"), 5).is_ok());
        hash_map.remove("abc");

        assert!(matches!(hash_map.get("abc"), None));
    }
    
    #[test]
    fn get_first_works() {
        let mut hash_map: ProbeHashMap<String, i32, 200> = ProbeHashMap::new();

        assert!(hash_map.insert(String::from("abc"), 5).is_ok());
        assert!(hash_map.get_first().is_some());
        assert!(matches!(hash_map.get_first().unwrap().key.as_str(), "abc"));
        assert!(matches!(hash_map.get_first().unwrap().value, 5));
        
        // Add new entry; first remains abc
        assert!(hash_map.insert(String::from("bcd"), 10).is_ok());
        assert!(hash_map.get_first().is_some());
        assert!(matches!(hash_map.get_first().unwrap().key.as_str(), "abc"));
        assert!(matches!(hash_map.get_first().unwrap().value, 5));

        // Removal of the previous first should make bcd become the next first
        hash_map.remove("abc");
        assert!(hash_map.get_first().is_some());
        assert!(matches!(hash_map.get_first().unwrap().key.as_str(), "bcd"));
        assert!(matches!(hash_map.get_first().unwrap().value, 10));
        
        // Add another entry; update bcd; this should cause cdf to become the next first
        assert!(hash_map.insert(String::from("cdf"), 15).is_ok());
        assert!(hash_map.insert(String::from("bcd"), 20).is_ok());
        assert!(hash_map.get_first().is_some());
        assert!(matches!(hash_map.get_first().unwrap().key.as_str(), "cdf"));
        assert!(matches!(hash_map.get_first().unwrap().value, 15));
    }

    #[test]
    fn get_last_works() {
        let mut hash_map: ProbeHashMap<String, i32, 200> = ProbeHashMap::new();

        assert!(hash_map.insert(String::from("abc"), 5).is_ok());
        assert!(hash_map.get_last().is_some());
        assert!(matches!(hash_map.get_last().unwrap().key.as_str(), "abc"));
        assert!(matches!(hash_map.get_last().unwrap().value, 5));

        assert!(hash_map.insert(String::from("bcd"), 15).is_ok());
        assert!(hash_map.get_last().is_some());
        assert!(matches!(hash_map.get_last().unwrap().key.as_str(), "bcd"));
        assert!(matches!(hash_map.get_last().unwrap().value, 15));

        // will update the map: this should also cause abc to become the next last
        assert!(hash_map.insert(String::from("abc"), 10).is_ok());
        assert!(hash_map.get_last().is_some());
        assert!(matches!(hash_map.get_last().unwrap().key.as_str(), "abc"));
        assert!(matches!(hash_map.get_last().unwrap().value, 10));
        
        // Remove last entry: this should cause bcd to become the new last
        hash_map.remove("abc");
        assert!(hash_map.get_last().is_some());
        assert!(matches!(hash_map.get_last().unwrap().key.as_str(), "bcd"));
        assert!(matches!(hash_map.get_last().unwrap().value, 15));
    }

}
