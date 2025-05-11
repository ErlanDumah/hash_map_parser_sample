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


use std::{iter::Peekable, time::Instant};

use reqwest::blocking::Client;



///
/// Sample result
/// 
/// [
///   {
///     "symbol": "BTC-200730-9000-C",
///     "priceChange": "-16.2038",        //24-hour price change
///     "priceChangePercent": "-0.0162",  //24-hour percent price change
///     "lastPrice": "1000",              //Last trade price
///     "lastQty": "1000",                //Last trade amount
///     "open": "1016.2038",              //24-hour open price
///     "high": "1016.2038",              //24-hour high
///     "low": "0",                       //24-hour low
///     "volume": "5",                    //Trading volume(contracts)
///     "amount": "1",                    //Trade amount(in quote asset)
///     "bidPrice":"999.34",              //The best buy price
///     "askPrice":"1000.23",             //The best sell price
///     "openTime": 1592317127349,        //Time the first trade occurred within the last 24 hours
///     "closeTime": 1592380593516,       //Time the last trade occurred within the last 24 hours     
///     "firstTradeId": 1,                //First trade ID
///     "tradeCount": 5,                  //Number of trades
///     "strikePrice": "9000",            //Strike price
///     "exercisePrice": "3000.3356"      //return estimated settlement price one hour before exercise, return index price at other times
///   }
/// ]
/// 

// Our idea for a parser is a direct scan of the characters
// This gives us a lot of power on the exact parsing and when to stop it

// First, let's define a suitable struct that represents the data:

#[derive(Clone, Debug)]
struct ResultEntry {
  symbol: String,
  priceChange: f64,
  priceChangePercent: f64,  //24-hour percent price change
  lastPrice: f64,              //Last trade price
  lastQty: f64,               //Last trade amount
  open: f64,              //24-hour open price
  high: f64,              //24-hour high
  low: f64,                       //24-hour low
  volume: f64,                    //Trading volume(contracts)
  amount: f64,                    //Trade amount(in quote asset)
  bidPrice: f64,              //The best buy price
  askPrice: f64,             //The best sell price
  openTime: usize,        //Time the first trade occurred within the last 24 hours
  closeTime: usize,       //Time the last trade occurred within the last 24 hours     
  firstTradeId: usize,                //First trade ID
  tradeCount: usize,                  //Number of trades
  strikePrice: f64,            //Strike price
  exercisePrice: f64,      //return estimated settlement price one hour before exercise, return index price at other times
}


impl ResultEntry {
    pub fn new() -> Self {
        ResultEntry { 
            symbol: String::new(),
            priceChange: 0.0,
            priceChangePercent: 0.0,
            lastPrice: 0.0,
            lastQty: 0.0,
            open: 0.0,
            high: 0.0, 
            low: 0.0, 
            volume: 0.0, 
            amount: 0.0, 
            bidPrice: 0.0, 
            askPrice: 0.0, 
            openTime: 0,
            closeTime: 0,
            firstTradeId: 0, 
            tradeCount: 0, 
            strikePrice: 0.0, 
            exercisePrice: 0.0,
        }
    }
}


// Note that we are making an assumption here: the body is encoded in standard ASCI characters
// and there will be no complicated string shenanigans
/*
const CHAR_ARRAY_START_AS_U8: u8 = '[' as u8;
const CHAR_ARRAY_END_AS_U8: u8 = ']' as u8;
const CHAR_OBJECT_START_AS_U8: u8 = '{' as u8;
const CHAR_OBJECT_END_AS_U8: u8 = '}' as u8;
const CHAR_STRING_SEPARATOR_AS_U8: u8 = '"' as u8;

const STR_SYMBOL_AS_STR: str = "symbol";
const STR_PRICE_CHANGE_AS_STR: str = "priceChange";
const STR_PRICE_CHANGE_PERCENT_AS_STR: str = "priceChangePercent";
const STR_LAST_PRICE_AS_STR: str = "lastPrice";
const STR_LAST_QTY_AS_STR: str = "lastQty";
const STR_OPEN_AS_STR: str = "open";
const STR_LOW_AS_STR: str = "low";
const STR_VOLUME_AS_STR: str = "volume";
const STR_AMOUNT_AS_STR: str = "amount";
const STR_BID_PRICE_AS_STR: str = "bidPrice";
const STR_ASK_PRICE_AS_STR: str = "askPrice";
const STR_OPEN_TIME_AS_STR: str = "openTime";
const STR_CLOSE_TIME_AS_STR: str = "closeTime";
const STR_FIRST_TRADE_ID_AS_STR: str = "firstTradeId";
const STR_TRADE_COUNT_AS_STR: str = "tradeCount";
const STR_STRIKE_PRICE_AS_STR: str = "strikePrice";
const STR_EXERCISE_PRICE_AS_STR: str = "exercisePrice";
*/

// An enum to represent the tokens we are looking for in the data:
#[derive(Debug)]
enum Token {
    ArrayStart, // '[' marking the beginning of JSON data array
    ArrayEnd, // ']'
    ObjectStart, // '{'
    ObjectEnd, // '}'
    StringValue(String), // "sometext"
    NumberValue(usize), // 1353426
    //KeyIdentifier // ':', can be ignored
    //DataSeparator // ',', can be ignored
}


// A few state machine states to represent the circumstances after each token:
#[derive(Debug)]
enum State {
    // [{"key": "value"}]
    //^                 ^
    Init,
    // [{"key": "value"}]
    // ^               ^
    Array,
    // [{"key": "value"}]
    //  ^             ^  
    Object,

    // [{"key": "value"}]
    //        ^
    Key(String),

    // [{"key": "value"}]
    //                 ^
    ValueString{key: String, value: String},
    // [{"key": 36275}]
    //               ^
    ValueNumber{key: String, value: String},
}

// Of course, this is way more complicated than using Serde for example
// But this also gives us the power of optimizing the entirety of the algorithm
// Let's define our parser as a struct that borrows data with lifetime 'data
#[derive(Debug)]
pub struct Parser<'data>{
    state: State,
    char_iterator: Peekable<std::str::Chars<'data>>,
    current_entry: ResultEntry,
}


impl<'data> Parser<'data> {
    /// Create a new Parser that borrows data from the String given
    pub fn new(data_as_string: &'data String) -> Self {
        Parser{
            state: State::Init,
            char_iterator: data_as_string.chars().peekable(),
            current_entry: ResultEntry::new(),
        }
    }

    /// Consumes the next token from our current data stream
    /// @return A token if there is data left, None otherwise
    fn consume_token(&mut self) -> Option<Token> {
        while let Some(character) = self.char_iterator.next() {
            match character {
                '[' => {
                    return Some(Token::ArrayStart)
                },
                ']' => {
                    return Some(Token::ArrayEnd)
                },
                '{' => {
                    return Some(Token::ObjectStart)
                },
                '}' => {
                    return Some(Token::ObjectEnd)
                },
                ',' | ':' => {
                    // Purposefully skip key identifiers and separators
                    continue;
                }
                '"' => {
                    // Parse a string: any character is accepted until next occurence of '"'
                    let mut value = String::new();
                    while let Some(string_character) = self.char_iterator.next() {
                        if string_character == '"' {
                            break;
                        }
                        value.push(string_character);
                    }
                    return Some(Token::StringValue(value));
                },
                '0' | '1' | '2' | '3' |  '4' |  '5' |  '6' |  '7' |  '8' |  '9' => {
                    // Parse a number string: add characters until a non-digit appears
                    // Important here is to not consume the first non-digit character
                    let mut number_value = String::new();
                    number_value.push(character);
                    while let Some(number_character) = self.char_iterator.peek() {
                        match number_character {
                            '0' | '1' | '2' | '3' |  '4' |  '5' |  '6' |  '7' |  '8' |  '9' => {
                                number_value.push(number_character.clone());
                                self.char_iterator.next();
                            },
                            _ => {
                                // We're making the bold assertion here that a string of number characters will never fail to parse into a usize
                                return Some(Token::NumberValue(number_value.parse::<usize>().unwrap()));
                            }
                        }
                    }
                }
                _ => {
                    println!("Unexpected character at consume_token(): {}", character);
                },
            }
        }

        return None;
    }

    /// Parses until the first ResultEntry was found
    /// @return ResultEntry if there is data left, None otherwise
    pub fn parse_single(&mut self) -> Option<ResultEntry> {
        while let Some(token) = self.consume_token() {
            match (&self.state, token) {
                (&State::Init, Token::ArrayStart) => {
                    self.state = State::Array;
                },

                (&State::Array, Token::ObjectStart) => {
                    self.state = State::Object;
                },
                (&State::Array, Token::ArrayEnd) => {
                    self.state = State::Init;
                },

                (&State::Object, Token::StringValue(key)) => {
                    self.state = State::Key(key);
                },
                (&State::Object, Token::ObjectEnd) => {
                    self.state = State::Array;
                    let entry = self.current_entry.clone();
                    self.current_entry = ResultEntry::new();
                    return Some(entry);
                },

                (&State::Key(ref key), Token::StringValue(value)) => {
                    match key.as_str() {
                        "symbol" => {
                            self.current_entry.symbol = value;
                        },
                        "priceChange" => {
                            match value.parse::<f64>() {
                                Ok(value_f64) => self.current_entry.priceChange = value_f64,
                                Err(error) => {},
                            }
                        },
                        "priceChangePercent" => {
                            match value.parse::<f64>() {
                                Ok(value_f64) => self.current_entry.priceChangePercent = value_f64,
                                Err(error) => {},
                            }
                        },
                        "lastPrice" => {
                            match value.parse::<f64>() {
                                Ok(value_f64) => self.current_entry.lastPrice = value_f64,
                                Err(error) => {},
                            }
                        },
                        "lastQty" => {
                            match value.parse::<f64>() {
                                Ok(value_f64) => self.current_entry.lastQty = value_f64,
                                Err(error) => {},
                            }
                        },
                        "open" => {
                            match value.parse::<f64>() {
                                Ok(value_f64) => self.current_entry.open = value_f64,
                                Err(error) => {},
                            }
                        },
                        "high" => {
                            match value.parse::<f64>() {
                                Ok(value_f64) => self.current_entry.high = value_f64,
                                Err(error) => {},
                            }
                        },
                        "low" => {
                            match value.parse::<f64>() {
                                Ok(value_f64) => self.current_entry.low = value_f64,
                                Err(error) => {},
                            }
                        },
                        "volume" => {
                            match value.parse::<f64>() {
                                Ok(value_f64) => self.current_entry.volume = value_f64,
                                Err(error) => {},
                            }
                        },
                        "amount" => {
                            match value.parse::<f64>() {
                                Ok(value_f64) => self.current_entry.amount = value_f64,
                                Err(error) => {},
                            }
                        },
                        "bidPrice" => {
                            match value.parse::<f64>() {
                                Ok(value_f64) => self.current_entry.bidPrice = value_f64,
                                Err(error) => {},
                            }
                        },
                        "askPrice" => {
                            match value.parse::<f64>() {
                                Ok(value_f64) => self.current_entry.askPrice = value_f64,
                                Err(error) => {},
                            }
                        },
                        "strikePrice" => {
                            match value.parse::<f64>() {
                                Ok(value_f64) => self.current_entry.strikePrice = value_f64,
                                Err(error) => {},
                            }
                        },
                        "exercisePrice" => {
                            match value.parse::<f64>() {
                                Ok(value_f64) => self.current_entry.exercisePrice = value_f64,
                                Err(error) => {},
                            }
                        },

                        _ => {
                            println!("Unexpected key found for string value {}: {}", key, value);
                        }
                    }
                    self.state = State::Object;
                },

                (&State::Key(ref key), Token::NumberValue(value)) => {
                    match key.as_str() {
                        "firstTradeId" => {
                            self.current_entry.firstTradeId = value;
                        },
                        "tradeCount" => {
                            self.current_entry.tradeCount = value;
                        },
                        "openTime" => {
                            self.current_entry.openTime = value;
                        },
                        "closeTime" => {
                            self.current_entry.closeTime = value;
                        },

                        _ => {
                            print!("Unexpected key found for number value {}: {}", key, value);
                        }
                    }
                    self.state = State::Object;
                },

                (_, token) => {
                    print!("T!(unexpected token {:?} in state {:?})", token, self.state);
                }
            }
        }
        return None;
    }
}



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

    // Initially trying to just output the whole response as text here crashed my CLI.
    // Message byte count: 491542
    println!("Message byte count: {}", body_text.len());

    // The data is, as expected huge
    let mut parser = Parser::new(&body_text);

    let start = Instant::now();

    // Test parsing a single entry.
    let single_entry = match parser.parse_single() {
        None => {
            println!("Could not first element");
            return;
        },
        Some(entry) => entry,
    };

    let duration = start.elapsed();

    // Parsing a single entry took 91.8µs
    println!("Parsing a single entry took {:?}", duration);

    // First  element symbol: ResultEntry { symbol: "BNB-250511-665-P", priceChange: -10.5, 
    // priceChangePercent: -0.84, lastPrice: 2.0, lastQty: 0.0, open: 12.5, high: 12.5, low: 2.0,
    // volume: 8.45, amount: 46.58, bidPrice: 1.9, askPrice: 3.0, openTime: 1746897259343, closeTime: 
    // 1746937541235, firstTradeId: 1, tradeCount: 8, strikePrice: 665.0, exercisePrice: 665.12765896 }
    // Seems to work fine, on to benchmarking
    println!("First element symbol: {:?}", single_entry);
}


mod tests {
    use crate::Parser;

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
            None => assert!(false, "parse_single() produced None"),
            Some(_) => {},
        };

        let second_entry = match parser.parse_single() {
            None => { 
                assert!(false, "parse_single() produced None");
                return;
            }
            Some(entry) => entry,
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
        while let Some(_) = parser.parse_single() {
            count += 1;
        }

        assert_eq!(count, 1436);
    }
}

