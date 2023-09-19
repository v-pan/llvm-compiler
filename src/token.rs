use crate::error::TokenizationError;
use std::{io::{BufReader, SeekFrom, Read}, io::Seek, fs::File};

use packed_struct::prelude::*;

#[derive(PackedStruct)]
#[derive(Clone, Copy, Debug)]
#[packed_struct(bits=40, bit_numbering="msb0")]
pub struct Token {
    #[packed_field(bits="0..32", endian="msb")]
    loc: u32,
    #[packed_field(bits="32..40", endian="msb", ty="enum")]
    ty: TokenType
}

impl Token {
    pub fn new(loc: u32, word: &str) -> Self {
        Token::try_keyword(loc, word)
        .or(
            Token::try_paren(loc, word)
        ).or(
            Token::try_operator(loc, word)
        ).or(
            Token::try_seperator(loc, word)
        ).or(
            Token::try_whitespace(loc, word)
        ).or(
            Token::try_quote(loc, word)
        ).unwrap_or(
            Token { loc, ty: TokenType::Unknown }
        )
    }

    pub fn get_string(&self, tokens: &Vec<Token>, reader: &mut BufReader<&File>) -> String {
        let idx = tokens.binary_search_by(|other| { other.loc.cmp(&self.loc) }).expect("Did not find token");
        let pos = SeekFrom::Start(self.loc.try_into().unwrap());

        let next = tokens.get(idx+1);
        reader.seek(pos).expect("Failed to seek to token start");

        if let Some(token) = next {
            let len = token.loc.checked_sub(self.loc).expect("Overflow occurred while getting token length");
            let mut buf = vec![0_u8; len.try_into().unwrap()];

            reader.read_exact(&mut buf).unwrap();

            // println!("Byte len: {len}, vec len: {}, buf: {:?}", buf.len(), buf);

            String::from_utf8(buf).unwrap()
        } else {
            let mut buf = vec![];
            reader.read_to_end(&mut buf).unwrap();
            String::from_utf8(buf).unwrap()
        }
    }

    pub fn get_string_packed(&self, tokens: &Vec<[u8;5]>, reader: &mut BufReader<&File>) -> String {
        let idx = tokens.binary_search(&self.pack().expect("Could not pack self")).expect("Did not find token");
        let pos = SeekFrom::Start(self.loc.try_into().unwrap());

        let next = tokens.get(idx+1);
        reader.seek(pos).expect("Failed to seek to token start");

        if let Some(token) = next {
            let len = Token::unpack_from_slice(token).unwrap().loc.checked_sub(self.loc).expect("Overflow occurred while getting token length");
            let mut buf = vec![0_u8; len.try_into().unwrap()];

            reader.read_exact(&mut buf).unwrap();

            // println!("Byte len: {len}, vec len: {}, buf: {:?}", buf.len(), buf);

            String::from_utf8(buf).unwrap()
        } else {
            let mut buf = vec![];
            reader.read_to_end(&mut buf).unwrap();
            String::from_utf8(buf).unwrap()
        }
    }

    pub fn try_keyword(loc: u32, word: &str) -> Option<Token> {
        match word {
            "fun" => Some(Token { loc: loc.into(), ty: TokenType::Function }),
            "if" => Some(Token { loc: loc.into(), ty: TokenType::If }),
            _ => None
        }
    }
    pub fn try_keyword_packed(loc: u32, word: &str) -> Result<[u8; 5], TokenizationError> {
        Ok(Token::try_keyword(loc, word).ok_or(TokenizationError::NoMatch)?.pack()?)
    }

    pub fn try_paren(loc: u32, word: &str) -> Option<Token> {
        match word {
            "(" => Some(Token { loc: loc.into(), ty: TokenType::OpenParen }),
            ")" => Some(Token { loc: loc.into(), ty: TokenType::CloseParen }),
            "<" => Some(Token { loc: loc.into(), ty: TokenType::OpenAngle }),
            ">" => Some(Token { loc: loc.into(), ty: TokenType::CloseAngle }),
            "{" => Some(Token { loc: loc.into(), ty: TokenType::OpenCurly }),
            "}" => Some(Token { loc: loc.into(), ty: TokenType::CloseCurly }),
            _ => None
        }
    }
    pub fn try_paren_packed(loc: u32, word: &str) -> Result<[u8; 5], TokenizationError> {
        Ok(Token::try_paren(loc, word).ok_or(TokenizationError::NoMatch)?.pack()?)
    }

    pub fn try_operator(loc: u32, word: &str) -> Option<Token> {
        match word {
            "+" => Some(Token { loc: loc.into(), ty: TokenType::Plus }),
            "-" => Some(Token { loc: loc.into(), ty: TokenType::Minus }),
            "*" => Some(Token { loc: loc.into(), ty: TokenType::Star }),
            "/" => Some(Token { loc: loc.into(), ty: TokenType::Slash }),
            _ => None
        }
    }
    pub fn try_operator_packed(loc: u32, word: &str) -> Result<[u8; 5], TokenizationError> {
        Ok(Token::try_operator(loc, word).ok_or(TokenizationError::NoMatch)?.pack()?)
    }

    pub fn try_seperator(loc: u32, word: &str) -> Option<Token> {
        match word {
            ":" => Some(Token { loc: loc.into(), ty: TokenType::TypeSeperator }),
            "," => Some(Token { loc: loc.into(), ty: TokenType::Comma }),
            _ => None
        }
    }
    pub fn try_seperator_packed(loc: u32, word: &str) -> Result<[u8; 5], TokenizationError> {
        Ok(Token::try_seperator(loc, word).ok_or(TokenizationError::NoMatch)?.pack()?)
    }

    pub fn try_quote(loc: u32, word: &str) -> Option<Token> {
        match word {
            "\"" => Some(Token { loc: loc.into(), ty: TokenType::DoubleQuote }),
            "\'" => Some(Token { loc: loc.into(), ty: TokenType::SingleQuote }),
            "`" => Some(Token { loc: loc.into(), ty: TokenType::Backtick }),
            _ => None
        }
    }
    pub fn try_quote_packed(loc: u32, word: &str) -> Result<[u8; 5], TokenizationError> {
        Ok(Token::try_quote(loc, word).ok_or(TokenizationError::NoMatch)?.pack()?)
    }

    pub fn try_whitespace(loc: u32, word: &str) -> Option<Token> {
        match word {
            " " => Some(Token { loc: loc.into(), ty: TokenType::Space }),
            "\n" => Some(Token { loc: loc.into(), ty: TokenType::Newline }),
            "\r\n" => Some(Token { loc: loc.into(), ty: TokenType::Newline }),
            _ => None
        }
    }
    pub fn try_whitespace_packed(loc: u32, word: &str) -> Result<[u8; 5], TokenizationError> {
        Ok(Token::try_whitespace(loc, word).ok_or(TokenizationError::NoMatch)?.pack()?)
    }
}

#[derive(PrimitiveEnum_u8)]
#[derive(Clone, Copy, Debug)]
pub enum TokenType {
    // Keywords
    Function,
    If,

    // Parentheses
    OpenParen,
    CloseParen,
    OpenCurly,
    CloseCurly,
    OpenAngle, // The angle brackets are also technically operators, context depending
    CloseAngle,

    // Quotes
    DoubleQuote,
    SingleQuote,
    Backtick,

    // Seperators
    TypeSeperator,
    Comma,

    // Operators (excl. angle brackets, see above)
    Plus,
    Minus,
    Star,
    Slash,

    // Whitespace
    Space,
    Newline,

    // Comments - Currently think comments aren't being split on, but will be tokenized as slashes and stars
    // LineComment,
    // OpenMultilineComment, 
    // CloseMultilineComment,

    // Unknown: Either an identifier or literal
    Unknown,
}
