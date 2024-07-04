use std::collections::HashMap;

#[derive(Debug, Default)]
pub enum Element {
    #[default]
    Empty,
    Dictionary(HashMap<String, Element>),
    List(Vec<Element>),
    ByteString(Contents),
    Int(i32),
}

impl Element {
    pub fn get_dictionary(self) -> Result<HashMap<String, Element>, ElementError> {
        match self {
            Element::Dictionary(v) => Ok(v),
            _ => Err(ElementError::WasntDictionary),
        }
    }

    pub fn get_string(self) -> Result<String, ElementError> {
        match self {
            Self::ByteString(v) => {
                if let Contents::String(contents) = v {
                    Ok(contents)
                } else {
                    Err(ElementError::WasntString)
                }
            }

            _ => Err(ElementError::WasntString),
        }
    }

    pub fn get_list(self) -> Result<Vec<Element>, ElementError> {
        match self {
            Self::List(v) => Ok(v),

            _ => Err(ElementError::WasntList),
        }
    }

    pub fn get_integer(self) -> Result<i32, ElementError> {
        match self {
            Self::Int(v) => Ok(v),

            _ => Err(ElementError::WasntInt),
        }
    }
}

#[derive(Debug)]
#[allow(clippy::enum_variant_names)]
pub enum ElementError {
    WasntString,
    WasntList,
    WasntDictionary,
    WasntInt
}

#[derive(Debug, Clone)]
pub enum Contents {
    String(String),
    Bytes(Vec<u8>),
}