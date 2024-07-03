use std::{
    collections::HashMap,
    error::Error,
    fmt::{Debug, Display},
};

#[derive(Debug, Default)]
pub enum Element {
    #[default]
    Empty,
    Dictionary(HashMap<String, Element>),
    List(Vec<Element>),
    ByteString(Contents),
    Int(i32),
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct Metainfo {
    root_dir: Element,
}

pub struct TorrentDecoder<'a> {
    buffer: &'a [u8],
    index: usize,
}

pub enum FmtError {
    Dictionary,
    Integer,
    List,
    WrongCharacter,
    NumberNotInUtf8,
    FailedToParseInteger,
    WrongKeyFormat,
}

impl Display for FmtError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let error_msg = match self {
            FmtError::Dictionary => String::from("Dictionary formatting error"),
            FmtError::List => String::from("List formatting error"),
            FmtError::Integer => String::from("Integer formatting error"),
            FmtError::WrongCharacter => String::from("Decoding wrong character"),
            FmtError::NumberNotInUtf8 => String::from("Encoded number wasn't valid UTF-8"),
            FmtError::FailedToParseInteger => String::from("Failed to parse integer"),
            FmtError::WrongKeyFormat => String::from("Dictionary key wasn't a string"),
        };

        write!(f, "{}", error_msg)
    }
}

impl Debug for FmtError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let error_msg = match self {
            FmtError::Dictionary => String::from("Dictionary formatting error"),
            FmtError::List => String::from("List formatting error"),
            FmtError::Integer => String::from("Integer formatting error"),
            FmtError::WrongCharacter => String::from("Decoding wrong character"),
            FmtError::NumberNotInUtf8 => String::from("Encoded number was not valid UTF-8"),
            FmtError::FailedToParseInteger => String::from("Failed to parse integer"),
            FmtError::WrongKeyFormat => String::from("Dictionary key wasn't a string"),
        };

        write!(f, "{}", error_msg)
    }
}

impl Error for FmtError {}

#[derive(Debug)]
pub enum Contents {
    String(String),
    Bytes(Vec<u8>),
}

impl<'a> TorrentDecoder<'a> {
    pub fn new(buffer: &'a [u8]) -> Self {
        Self { buffer, index: 0 }
    }

    fn decode_character(&mut self, character: u8) -> Result<Option<Element>, FmtError> {
        match character {
            b'd' => Ok(Some(self.decode_dictionary()?)),
            b'l' => Ok(Some(self.decode_list()?)),
            b'i' => Ok(Some(self.decode_integer()?)),
            b'0'..=b'9' => {
                let bytestring = self.decode_byte_string();
                Ok(Some(Element::ByteString(bytestring?)))
            }
            b'e' => {
                // println!("END");
                self.index += 1;
                Ok(None)
            }

            _ => Err(FmtError::WrongCharacter),
        }
    }

    fn get_char(&mut self) -> u8 {
        self.buffer[self.index]
    }

    fn decode_list(&mut self) -> Result<Element, FmtError> {
        // println!("LIST");
        let mut character = self.get_char();

        if character != b'l' {
            // panic!("LISTA MAL FORMATEADA");
            return Err(FmtError::List);
        }
        self.index += 1;

        character = self.get_char();
        let mut elem_vector = vec![];

        while let Some(elem) = self.decode_character(character)? {
            character = self.get_char();
            elem_vector.push(elem)
        }

        Ok(Element::List(elem_vector))
    }

    fn decode_byte_string(&mut self) -> Result<Contents, FmtError> {
        // println!("STRING");
        let mut val = self.get_char();
        self.index += 1;

        let mut length_vec = vec![];

        while val != b':' {
            length_vec.push(val);

            val = self.get_char();
            self.index += 1;
        }

        let len_str = String::from_utf8(length_vec);

        if len_str.is_err() {
            return Err(FmtError::NumberNotInUtf8);
        }

        let len = len_str.unwrap().parse::<usize>();

        if len.is_err() {
            return Err(FmtError::FailedToParseInteger);
        }

        let len = len.unwrap();

        let decoded = std::str::from_utf8(&self.buffer[self.index..self.index + len]);

        let contents = match decoded {
            Err(_) => Contents::Bytes(Vec::from(&self.buffer[self.index..self.index + len])),
            Ok(v) => Contents::String(String::from(v)),
        };

        self.index += len;

        // println!("{len}:{contents}");

        Ok(contents)
    }

    fn decode_integer(&mut self) -> Result<Element, FmtError> {
        let character = self.get_char();

        if character != b'i' {
            return Err(FmtError::Integer);
        }
        self.index += 1;

        let mut index_right = self.index;
        let mut val_right = self.buffer[index_right];

        while val_right != b'e' {
            index_right += 1;
            val_right = self.buffer[index_right];
        }

        let num_str = String::from_utf8_lossy(&self.buffer[self.index..index_right]);

        let num = num_str.parse::<i32>();

        if num.is_err() {
            return Err(FmtError::FailedToParseInteger);
        }
        let num = num.unwrap();

        self.index = index_right + 1;

        Ok(Element::Int(num))
    }

    fn decode_dictionary(&mut self) -> Result<Element, FmtError> {
        // println!("DICCIONARIO");
        let mut dictionary = HashMap::new();
        let character = self.get_char();

        if character != b'd' {
            return Err(FmtError::Dictionary);
        }
        self.index += 1;

        loop {
            // println!("CLAVE");
            let character = self.get_char();
            let key_elem = self.decode_character(character)?;

            let key = if key_elem.is_none() {
                return Ok(Element::Dictionary(dictionary));
            } else {
                match key_elem.unwrap() {
                    Element::ByteString(string) => {
                        if let Contents::String(v) = string {
                            v
                        } else {
                            return Err(FmtError::WrongKeyFormat);
                        }
                    }
                    _ => unreachable!(),
                }
            };

            let character = self.get_char();

            // println!("VALOR");
            let value = self.decode_character(character)?;

            dictionary.insert(key, value.unwrap());
        }
    }

    pub fn decode_metafile(&mut self) -> Result<Metainfo, FmtError> {
        let root_dir = self.decode_dictionary()?;

        Ok(Metainfo { root_dir })
    }
}
