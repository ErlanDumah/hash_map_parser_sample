// Assumptions:
// The assignment aims to create a parser of large data from
//  an endpoint, with the point of minimizing feedback lag that
//  would result from parsing all of the data at once and then
//  returning.

// Approach: 
// A custom-made lexer / parser that directly scans the
//  data fetched as a stream of character entries. Having defined
//  it this way gives us control from when to where the parser operates,
//  with the possibility of stopping it after a fixed amount of data
//  and resuming at a later time.


use std::time::Instant;

use reqwest::blocking::Client;

use parser_sample::Parser;
use parser_sample::parser::ParseError;

fn main() {
    // We're just using a simple library here to access the HTTP request in a blocking way
    let url = "https://eapi.binance.com/eapi/v1/ticker";

    let client = Client::new();
    let http_result = client.get(url)
        .send();

    let response = match http_result {
        Ok(response) => response,
        Err(error) => {
            println!("Request was not successful: {:#?}", error);
            return;
        },
    };

    println!("Request was successful: {:#?}", response);

    let body_text = match response.text() {
        Ok(text) => text,
        Err(error) => {
            println!("Request could not be parsed as bytes: {:#?}", error);
            return;
        }
    };

    // Save the result to assets for easy access for testing, benchmarking
    //let file_path = "./assets/body_text.json";
    //match std::fs::write(file_path, body_text) {
    //    Ok(()) => {},
    //    Err(error) => {
    //        println!("Writing the response into {} failed: {}", file_path, error);
    //    }
    //}
    //println!("Successfully wrote the response into {}", file_path);
    //return;

    // Message byte count: 491542
    println!("Message byte count: {}", body_text.len());

    let mut parser = Parser::new(&body_text);

    let start = Instant::now();
    // Test parsing a single entry.
    let single_entry = match parser.parse_single() {
        Err(error) => {
            println!("Error parsing first entry: {}", error);
            return;
        },
        Ok(entry) => entry,
    };
    let duration = start.elapsed();

    // Parsing a single entry took 91.8µs (that's in debug though)
    println!("Parsing a single entry took {:?}", duration);

    // First  element symbol: ResultEntry { symbol: "BNB-250511-665-P", priceChange: -10.5, 
    // priceChangePercent: -0.84, lastPrice: 2.0, lastQty: 0.0, open: 12.5, high: 12.5, low: 2.0,
    // volume: 8.45, amount: 46.58, bidPrice: 1.9, askPrice: 3.0, openTime: 1746897259343, closeTime: 
    // 1746937541235, firstTradeId: 1, tradeCount: 8, strikePrice: 665.0, exercisePrice: 665.12765896 }
    // Seems to work fine, on to benchmarking
    println!("First element symbol: {:?}", single_entry);

    let start = Instant::now();
    // Parse another 100 entries:
    for _ in 0..100 {
        match parser.parse_single() {
            Err(error) => {
                println!("Error parsing 100 entries: {}", error);
                return;
            },
            Ok(_) => {},
        }
    }
    let duration = start.elapsed();

    // Parsing 100 further entries took 2.1233ms (that's in debug though)
    println!("Parsing 100 further entries took {:?}", duration);

    // Find benchmarking code in Q2/benches/bench_parser.rs
}


mod tests {
    use crate::Parser;
    use crate::ParseError;

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
    fn parse_single_works() {
        let file_path = "./assets/body_text.json";
        let file = match std::fs::read_to_string(file_path) {
            Ok(file) => file,
            Err(error) => {
                assert!(false, "Reading the asset file failed: {}", error);
                return;
            }
        };

        let mut parser = Parser::new(&file);

        // The first entry happened to be uninteresting, lots of 0s and 0.0s
        let _ = match parser.parse_single() {
            Err(error) => assert!(false, "parse_single() produced an error: {}", error),
            Ok(_) => {},
        };

        let second_entry = match parser.parse_single() {
            Err(error) => { 
                assert!(false, "parse_single() produced an error: {}", error);
                return;
            }
            Ok(entry) => entry,
        };

        assert!(matches!(second_entry.symbol.as_str(), "ETH-250516-2550-C"));
        assert!(matches!(second_entry.priceChange, -1.6));
        assert!(matches!(second_entry.priceChangePercent, -0.0201));
        assert!(matches!(second_entry.lastPrice, 78.0));
        assert!(matches!(second_entry.lastQty, 0.2));
        assert!(matches!(second_entry.open, 79.6));
        assert!(matches!(second_entry.high, 115.8)); 
        assert!(matches!(second_entry.low, 77.2)); 
        assert!(matches!(second_entry.volume, 72.26)); 
        assert!(matches!(second_entry.amount, 6090.82)); 
        assert!(matches!(second_entry.bidPrice, 84.8)); 
        assert!(matches!(second_entry.askPrice, 85.8)); 
        assert!(matches!(second_entry.openTime, 1746898120943)); 
        assert!(matches!(second_entry.closeTime, 1746954696155)); 
        assert!(matches!(second_entry.firstTradeId, 1));  
        assert!(matches!(second_entry.tradeCount, 24));  
        assert!(matches!(second_entry.strikePrice, 2550.0)); 
        assert!(matches!(second_entry.exercisePrice, 2511.22651163));
    }
    
    #[test]
    fn parsing_entire_data_works() {
        let file_path = "./assets/body_text.json";
        let file = match std::fs::read_to_string(file_path) {
            Ok(file) => file,
            Err(error) => {
                assert!(false, "Reading the asset file failed: {}", error);
                return;
            }
        };

        let mut parser = Parser::new(&file);

        let mut count = 0;
        // No error is being thrown during parsing of the entire file
        loop {
            match parser.parse_single() {
                Err(ParseError::EndOfData) => break,
                Err(error) => {
                    assert!(false, "parse_single produced a non-EndOfData error: {}", error);
                }
                Ok(_) => count += 1,
            }
        }

        assert_eq!(count, 1436);
    }
}

