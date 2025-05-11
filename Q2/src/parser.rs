
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
    EndOfData,
    UnrecognisedToken,
}

// An error enum that represents all errors that can occur during parsing
pub enum ParseError {
    EndOfData, // There is no data left to be parsed
    UnrecognisedToken, // There was an unexpected token encountered
    UnrecognisedKeyValuePair,
    ParseFloatError(ParseFloatError), // An expected float point value could not be parsed as such
}

// Pretty printing for our ParseError
impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            &ParseError::EndOfData => {
                write!(f, "The end of data was reached.")
            },
            &ParseError::UnrecognisedToken => {
                write!(f, "An unrecognised token was encountered.")
            },
            &ParseError::UnrecognisedKeyValuePair => {
                write!(f, "An unrecognised key value pair was encountered.")
            },
            &ParseError::ParseFloatError(error) => {
                write!(f, "There was a parse float error: {}", error)
            },
        }
    }
}

// An enum to represent the lexical tokens we are looking for in the data:
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
    //   ^    ^^       ^
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
    /// @return A token if there is data left, None otherwise
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
                    return Err(ParseTokenError::UnrecognisedToken);
                },
            }
        }

        return Err(ParseTokenError::EndOfData);
    }

    /// Parses until the first ResultEntry was found
    /// @return ResultEntry if there is data left, None otherwise
    pub fn parse_single(&mut self) -> Result<ResultEntry, ParseError> {
        loop {
            let token = match self.consume_token() {
                Err(ParseTokenError::EndOfData) => break,
                Err(ParseTokenError::UnrecognisedToken) => return Err(ParseError::UnrecognisedToken),
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
                    match key.as_str() {
                        "symbol" => {
                            self.current_entry.symbol = value;
                        },
                        "priceChange" => {
                            match value.parse::<f64>() {
                                Ok(value_f64) => self.current_entry.priceChange = value_f64,
                                Err(error) => return Err(ParseError::ParseFloatError(error)),
                            }
                        },
                        "priceChangePercent" => {
                            match value.parse::<f64>() {
                                Ok(value_f64) => self.current_entry.priceChangePercent = value_f64,
                                Err(error) => return Err(ParseError::ParseFloatError(error)),
                            }
                        },
                        "lastPrice" => {
                            match value.parse::<f64>() {
                                Ok(value_f64) => self.current_entry.lastPrice = value_f64,
                                Err(error) => return Err(ParseError::ParseFloatError(error)),
                            }
                        },
                        "lastQty" => {
                            match value.parse::<f64>() {
                                Ok(value_f64) => self.current_entry.lastQty = value_f64,
                                Err(error) => return Err(ParseError::ParseFloatError(error)),
                            }
                        },
                        "open" => {
                            match value.parse::<f64>() {
                                Ok(value_f64) => self.current_entry.open = value_f64,
                                Err(error) => return Err(ParseError::ParseFloatError(error)),
                            }
                        },
                        "high" => {
                            match value.parse::<f64>() {
                                Ok(value_f64) => self.current_entry.high = value_f64,
                                Err(error) => return Err(ParseError::ParseFloatError(error)),
                            }
                        },
                        "low" => {
                            match value.parse::<f64>() {
                                Ok(value_f64) => self.current_entry.low = value_f64,
                                Err(error) => return Err(ParseError::ParseFloatError(error)),
                            }
                        },
                        "volume" => {
                            match value.parse::<f64>() {
                                Ok(value_f64) => self.current_entry.volume = value_f64,
                                Err(error) => return Err(ParseError::ParseFloatError(error)),
                            }
                        },
                        "amount" => {
                            match value.parse::<f64>() {
                                Ok(value_f64) => self.current_entry.amount = value_f64,
                                Err(error) => return Err(ParseError::ParseFloatError(error)),
                            }
                        },
                        "bidPrice" => {
                            match value.parse::<f64>() {
                                Ok(value_f64) => self.current_entry.bidPrice = value_f64,
                                Err(error) => return Err(ParseError::ParseFloatError(error)),
                            }
                        },
                        "askPrice" => {
                            match value.parse::<f64>() {
                                Ok(value_f64) => self.current_entry.askPrice = value_f64,
                                Err(error) => return Err(ParseError::ParseFloatError(error)),
                            }
                        },
                        "strikePrice" => {
                            match value.parse::<f64>() {
                                Ok(value_f64) => self.current_entry.strikePrice = value_f64,
                                Err(error) => return Err(ParseError::ParseFloatError(error)),
                            }
                        },
                        "exercisePrice" => {
                            match value.parse::<f64>() {
                                Ok(value_f64) => self.current_entry.exercisePrice = value_f64,
                                Err(error) => return Err(ParseError::ParseFloatError(error)),
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

        return Err(ParseError::EndOfData);
    }
}
