
use std::{fmt::Display, iter::Peekable, num::ParseFloatError};

// Our idea for a parser is a direct scan of the characters
// This gives us a lot of power on the exact parsing and when to stop it
// First, let's define a suitable struct that represents the data:

#[derive(Clone, Debug)]
pub struct ResultEntry {
  pub symbol: String,
  pub priceChange: f64,
  pub priceChangePercent: f64,
  pub lastPrice: f64,
  pub lastQty: f64,
  pub open: f64,
  pub high: f64,
  pub low: f64,
  pub volume: f64,
  pub amount: f64,
  pub bidPrice: f64,
  pub askPrice: f64,
  pub openTime: usize,
  pub closeTime: usize,
  pub firstTradeId: usize,
  pub tradeCount: usize,
  pub strikePrice: f64,
  pub exercisePrice: f64,
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

// An error enum that represents all errors that can occur during lexing
enum ParseTokenError {
    EndOfData, // There is no data left to be parsed
    UnrecognisedToken(char), // There was an unexpected token encountered
}

// An error enum that represents all errors that can occur during parsing
pub enum ParseError {
    EndOfData, // There is no data left to be parsed
    UnrecognisedToken(char), // There was an unexpected token encountered
    UnrecognisedKeyStringValuePair{ key: String, value: String }, // An unrecognised key with a string value was found
    UnrecognisedKeyNumberValuePair{ key: String, value: usize }, // An unrecognised key with a number value was found
    ParseFloatError{ key: String, value: String, error: ParseFloatError}, // An expected float point value could not be parsed as such
}

// Pretty printing for our ParseError
impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            &ParseError::EndOfData => {
                write!(f, "The end of data was reached.")
            },
            &ParseError::UnrecognisedToken(ref string) => {
                write!(f, "An unrecognised token {} was encountered.", string)
            },
            &ParseError::UnrecognisedKeyStringValuePair{ref key, ref value} => {
                write!(f, "Unexpected key {} found with string value {}", key, value)
            }
            &ParseError::UnrecognisedKeyNumberValuePair{ ref key, ref value } => {
                write!(f, "Unexpected key {} found with number value {}", key, value)
            },
            &ParseError::ParseFloatError{ ref key, ref value, ref error} => {
                write!(f, "Key entry {} with string value \"{}\" could not be parsed as float: {}", key, value, error)
            },
        }
    }
}

// An enum to represent the lexical tokens we are looking for in the data:
#[derive(Debug)]
enum Token {
    ArrayStart, // '[' marking the beginning of a JSON data array
    ArrayEnd, // ']' marking the end of a JSON data array
    ObjectStart, // '{' marking the beginning of a JSON data object
    ObjectEnd, // '}' marking the end of a JSON data object
    StringValue(String), // "sometext", the data containing all characters within the '"' span
    NumberValue(usize), // 1353426, data not marked with a '"' but restricted to a series of digits
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
    //        ^^       ^
    Key(String),
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

// Note that we are making an assumption here: the body is encoded in standard ASCI characters
// and one character always corresponds to one "letter".
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
    /// @return A token if the next token could be parsed successfully, an error otherwise (including end of data)
    fn consume_token(&mut self) -> Result<Token, ParseTokenError> {
        while let Some(character) = self.char_iterator.next() {
            match character {
                '[' => {
                    return Ok(Token::ArrayStart)
                },
                ']' => {
                    return Ok(Token::ArrayEnd)
                },
                '{' => {
                    return Ok(Token::ObjectStart)
                },
                '}' => {
                    return Ok(Token::ObjectEnd)
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
                    return Ok(Token::StringValue(value));
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
                                return Ok(Token::NumberValue(number_value.parse::<usize>().unwrap()));
                            }
                        }
                    }
                }
                _ => {
                    return Err(ParseTokenError::UnrecognisedToken(character));
                },
            }
        }

        return Err(ParseTokenError::EndOfData);
    }

    /// Set data of given entry according to JSON key string value pair
    /// @return Ok(()) if given key value pair is a valid entry, otherwise an error specifying the issue
    fn set_data_from_string(entry: &mut ResultEntry, key: &String, value: String) -> Result<(), ParseError>{
        match key.as_str() {
            "symbol" => {
                entry.symbol = value;
            },
            "priceChange" => {
                match value.parse::<f64>() {
                    Ok(value_f64) => entry.priceChange = value_f64,
                    Err(error) => return Err(ParseError::ParseFloatError{ key: key.clone(), value, error, }),
                }
            },
            "priceChangePercent" => {
                match value.parse::<f64>() {
                    Ok(value_f64) => entry.priceChangePercent = value_f64,
                    Err(error) => return Err(ParseError::ParseFloatError{ key: key.clone(), value, error, }),
                }
            },
            "lastPrice" => {
                match value.parse::<f64>() {
                    Ok(value_f64) => entry.lastPrice = value_f64,
                    Err(error) => return Err(ParseError::ParseFloatError{ key: key.clone(), value, error, }),
                }
            },
            "lastQty" => {
                match value.parse::<f64>() {
                    Ok(value_f64) => entry.lastQty = value_f64,
                    Err(error) => return Err(ParseError::ParseFloatError{ key: key.clone(), value, error, }),
                }
            },
            "open" => {
                match value.parse::<f64>() {
                    Ok(value_f64) => entry.open = value_f64,
                    Err(error) => return Err(ParseError::ParseFloatError{ key: key.clone(), value, error, }),
                }
            },
            "high" => {
                match value.parse::<f64>() {
                    Ok(value_f64) => entry.high = value_f64,
                    Err(error) => return Err(ParseError::ParseFloatError{ key: key.clone(), value, error, }),
                }
            },
            "low" => {
                match value.parse::<f64>() {
                    Ok(value_f64) => entry.low = value_f64,
                    Err(error) => return Err(ParseError::ParseFloatError{ key: key.clone(), value, error, }),
                }
            },
            "volume" => {
                match value.parse::<f64>() {
                    Ok(value_f64) => entry.volume = value_f64,
                    Err(error) => return Err(ParseError::ParseFloatError{ key: key.clone(), value, error, }),
                }
            },
            "amount" => {
                match value.parse::<f64>() {
                    Ok(value_f64) => entry.amount = value_f64,
                    Err(error) => return Err(ParseError::ParseFloatError{ key: key.clone(), value, error, }),
                }
            },
            "bidPrice" => {
                match value.parse::<f64>() {
                    Ok(value_f64) => entry.bidPrice = value_f64,
                    Err(error) => return Err(ParseError::ParseFloatError{ key: key.clone(), value, error, }),
                }
            },
            "askPrice" => {
                match value.parse::<f64>() {
                    Ok(value_f64) => entry.askPrice = value_f64,
                    Err(error) => return Err(ParseError::ParseFloatError{ key: key.clone(), value, error, }),
                }
            },
            "strikePrice" => {
                match value.parse::<f64>() {
                    Ok(value_f64) => entry.strikePrice = value_f64,
                    Err(error) => return Err(ParseError::ParseFloatError{ key: key.clone(), value, error, }),
                }
            },
            "exercisePrice" => {
                match value.parse::<f64>() {
                    Ok(value_f64) => entry.exercisePrice = value_f64,
                    Err(error) => return Err(ParseError::ParseFloatError{ key: key.clone(), value, error, }),
                }
            },

            _ => {
                return Err(ParseError::UnrecognisedKeyStringValuePair { key: key.clone(), value, });
            }
        }

        return Ok(());
    }

    /// Set data of given entry according to JSON key number value pair
    /// @return Ok(()) if given key value pair is a valid entry, otherwise an error specifying the issue
    fn set_data_from_number(entry: &mut ResultEntry, key: &String, value: usize) -> Result<(), ParseError> {
        match key.as_str() {
            "firstTradeId" => {
                entry.firstTradeId = value;
            },
            "tradeCount" => {
                entry.tradeCount = value;
            },
            "openTime" => {
                entry.openTime = value;
            },
            "closeTime" => {
                entry.closeTime = value;
            },

            _ => {
                return Err(ParseError::UnrecognisedKeyNumberValuePair { key: key.clone(), value, });
            }
        }

        return Ok(());
    }

    /// Parses until the first ResultEntry was found
    /// @return ResultEntry if there is data left, an error otherwise (including end of data)
    pub fn parse_single(&mut self) -> Result<ResultEntry, ParseError> {
        loop {
            let token = match self.consume_token() {
                Err(ParseTokenError::EndOfData) => break,
                Err(ParseTokenError::UnrecognisedToken(character)) => return Err(ParseError::UnrecognisedToken(character)),
                Ok(token) => token,
            };
        
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
                    return Ok(entry);
                },

                (&State::Key(ref key), Token::StringValue(value)) => {
                    if let Err(error) = Self::set_data_from_string(&mut self.current_entry, key, value) {
                        return Err(error);
                    }
                    self.state = State::Object;
                },

                (&State::Key(ref key), Token::NumberValue(value)) => {
                    if let Err(error) = Self::set_data_from_number(&mut self.current_entry, key, value) {
                        return Err(error);
                    }
                    self.state = State::Object;
                },

                (_, token) => {
                    print!("T!(unexpected token {:?} in state {:?})", token, self.state);
                }
            }
        }

        return Err(ParseError::EndOfData);
    }
}
