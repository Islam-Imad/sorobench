//! Lexer for the // ---- expectation DSL, a port of TestFileParser::Scanner.
//!
//! Byte-level (not char-level) to match the C++ std::string semantics exactly,
//! including \xNN string escapes producing arbitrary bytes. All physical lines
//! are concatenated with their newline stripped, so the // token (Newline) is
//! the only line separator inside a block.

use super::{PResult, ParseError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Token {
    Unknown,
    Invalid,
    Eos,
    Whitespace,
    LParen,
    RParen,
    LBrack,
    RBrack,
    LBrace,
    RBrace,
    Sub,
    Tilde,
    Colon,
    Comma,
    Period,
    Arrow,
    Newline,
    Comment,
    Number,
    HexNumber,
    String,
    Identifier,
    Ether,
    Wei,
    Hex,
    Boolean,
    Left,
    Library,
    Right,
    Failure,
    Gas,
}

impl Token {
    pub fn format(self) -> &'static str {
        match self {
            Token::Unknown => "unknown",
            Token::Invalid => "invalid",
            Token::Eos => "EOS",
            Token::Whitespace => "_",
            Token::LParen => "(",
            Token::RParen => ")",
            Token::LBrack => "[",
            Token::RBrack => "]",
            Token::LBrace => "{",
            Token::RBrace => "}",
            Token::Sub => "-",
            Token::Tilde => "~",
            Token::Colon => ":",
            Token::Comma => ",",
            Token::Period => ".",
            Token::Arrow => "->",
            Token::Newline => "//",
            Token::Comment => "#",
            Token::Number => "number",
            Token::HexNumber => "hex_number",
            Token::String => "string",
            Token::Identifier => "identifier",
            Token::Ether => "ether",
            Token::Wei => "wei",
            Token::Hex => "hex",
            Token::Boolean => "boolean",
            Token::Left => "left",
            Token::Library => "library",
            Token::Right => "right",
            Token::Failure => "FAILURE",
            Token::Gas => "gas",
        }
    }
}

fn is_identifier_start(c: u8) -> bool {
    c == b'_' || c == b'$' || c.is_ascii_alphabetic()
}
fn is_decimal_digit(c: u8) -> bool {
    c.is_ascii_digit()
}
fn is_identifier_part(c: u8) -> bool {
    is_identifier_start(c) || is_decimal_digit(c)
}
fn is_hex_digit(c: u8) -> bool {
    is_decimal_digit(c) || (b'a'..=b'f').contains(&c) || (b'A'..=b'F').contains(&c)
}
fn is_white_space(c: u8) -> bool {
    matches!(c, b' ' | b'\n' | b'\t' | b'\r' | 0x0b | 0x0c)
}

pub struct Scanner {
    source: Vec<u8>,
    pos: usize,
    current_token: Token,
    current_literal: Vec<u8>,
}

impl Scanner {
    pub fn new(source: &str) -> Scanner {
        let joined: Vec<u8> = source.lines().flat_map(|l| l.bytes()).collect();
        Scanner {
            source: joined,
            pos: 0,
            current_token: Token::Unknown,
            current_literal: Vec::new(),
        }
    }

    pub fn current_token(&self) -> Token {
        self.current_token
    }

    pub fn current_literal(&self) -> &[u8] {
        &self.current_literal
    }

    pub fn current_literal_string(&self) -> String {
        String::from_utf8_lossy(&self.current_literal).into_owned()
    }

    fn current(&self) -> u8 {
        if self.pos >= self.source.len() {
            b'\0'
        } else {
            self.source[self.pos]
        }
    }

    // Next char without advancing; \0 if fewer than 2 chars remain.
    fn peek(&self) -> u8 {
        if self.pos + 1 >= self.source.len() {
            b'\0'
        } else {
            self.source[self.pos + 1]
        }
    }

    fn advance(&mut self, n: usize) {
        self.pos = (self.pos + n).min(self.source.len());
    }

    fn is_end_of_file(&self) -> bool {
        self.pos >= self.source.len()
    }

    // Scan and store the next token (mirror of scanNextToken).
    pub fn scan_next_token(&mut self) -> PResult<()> {
        self.current_token = Token::Unknown;
        self.current_literal.clear();
        loop {
            match self.current() {
                b'/' => {
                    self.advance(1);
                    if self.current() == b'/' {
                        self.select(Token::Newline, Vec::new());
                    } else {
                        self.select(Token::Invalid, Vec::new());
                    }
                }
                b'-' => {
                    if self.peek() == b'>' {
                        self.advance(1);
                        self.select(Token::Arrow, Vec::new());
                    } else {
                        self.select(Token::Sub, Vec::new());
                    }
                }
                b'~' => {
                    self.advance(1);
                    let line = self.read_line();
                    self.select(Token::Tilde, line);
                }
                b':' => self.select(Token::Colon, Vec::new()),
                b'#' => {
                    let c = self.scan_comment();
                    self.select(Token::Comment, c);
                }
                b',' => self.select(Token::Comma, Vec::new()),
                b'(' => self.select(Token::LParen, Vec::new()),
                b')' => self.select(Token::RParen, Vec::new()),
                b'[' => self.select(Token::LBrack, Vec::new()),
                b']' => self.select(Token::RBrack, Vec::new()),
                b'"' => {
                    let s = self.scan_string()?;
                    self.select(Token::String, s);
                }
                c => {
                    if is_identifier_start(c) {
                        let ident = self.scan_identifier_or_keyword();
                        let (tok, lit) = detect_keyword(&ident);
                        self.current_token = tok;
                        self.current_literal = lit;
                        self.advance(1);
                    } else if is_decimal_digit(c) {
                        if c == b'0' && self.peek() == b'x' {
                            self.advance(1);
                            self.advance(1);
                            let mut lit = b"0x".to_vec();
                            lit.extend_from_slice(&self.scan_hex_number());
                            self.select(Token::HexNumber, lit);
                        } else {
                            let n = self.scan_decimal_number();
                            self.select(Token::Number, n);
                        }
                    } else if is_white_space(c) {
                        self.select(Token::Whitespace, Vec::new());
                    } else if self.is_end_of_file() {
                        self.current_token = Token::Eos;
                        self.current_literal.clear();
                    } else {
                        return Err(ParseError::new(format!(
                            "Unexpected character: '{}'",
                            c as char
                        )));
                    }
                }
            }
            if self.current_token != Token::Whitespace {
                break;
            }
        }
        Ok(())
    }

    // advance() then set token+literal (mirror of the selectToken lambda).
    fn select(&mut self, token: Token, literal: Vec<u8>) {
        self.advance(1);
        self.current_token = token;
        self.current_literal = literal;
    }

    // Reads a "~ ..." side-effect line up to (but not including) the next /.
    fn read_line(&mut self) -> Vec<u8> {
        let mut line = Vec::new();
        while self.peek() != b'\0' && self.peek() != b'/' {
            self.advance(1);
            line.push(self.current());
        }
        line
    }

    fn scan_comment(&mut self) -> Vec<u8> {
        let mut comment = Vec::new();
        self.advance(1);
        while self.current() != b'#' {
            if self.is_end_of_file() {
                break; // defensive: C++ would assert on an unterminated comment
            }
            comment.push(self.current());
            self.advance(1);
        }
        comment
    }

    fn scan_identifier_or_keyword(&mut self) -> Vec<u8> {
        let mut ident = vec![self.current()];
        while is_identifier_part(self.peek()) {
            self.advance(1);
            ident.push(self.current());
        }
        ident
    }

    fn scan_decimal_number(&mut self) -> Vec<u8> {
        let mut number = vec![self.current()];
        while is_decimal_digit(self.peek()) || self.peek() == b'.' {
            self.advance(1);
            number.push(self.current());
        }
        number
    }

    fn scan_hex_number(&mut self) -> Vec<u8> {
        let mut number = vec![self.current()];
        while is_hex_digit(self.peek()) {
            self.advance(1);
            number.push(self.current());
        }
        number
    }

    fn scan_string(&mut self) -> PResult<Vec<u8>> {
        let mut s = Vec::new();
        self.advance(1);
        while self.current() != b'"' {
            if self.is_end_of_file() {
                break; // defensive: C++ would assert on an unterminated string
            }
            if self.current() == b'\\' {
                self.advance(1);
                match self.current() {
                    b'\\' => {
                        s.push(b'\\');
                        self.advance(1);
                    }
                    b'n' => {
                        s.push(b'\n');
                        self.advance(1);
                    }
                    b'r' => {
                        s.push(b'\r');
                        self.advance(1);
                    }
                    b't' => {
                        s.push(b'\t');
                        self.advance(1);
                    }
                    b'0' => {
                        s.push(b'\0');
                        self.advance(1);
                    }
                    b'x' => s.push(self.scan_hex_part()?),
                    _ => {
                        return Err(ParseError::new(
                            "Invalid or escape sequence found in string literal.",
                        ))
                    }
                }
            } else {
                s.push(self.current());
                self.advance(1);
            }
        }
        Ok(s)
    }

    fn scan_hex_part(&mut self) -> PResult<u8> {
        let to_lower = |c: u8| c.to_ascii_lowercase();
        self.advance(1); // skip 'x'

        let mut value: u8;
        let c = self.current();
        if c.is_ascii_digit() {
            value = c - b'0';
        } else if (b'a'..=b'f').contains(&to_lower(c)) {
            value = to_lower(c) - b'a' + 10;
        } else {
            return Err(ParseError::new("\\x used with no following hex digits."));
        }

        self.advance(1);
        if self.current() == b'"' {
            return Ok(value);
        }

        value <<= 4;
        let c = self.current();
        if c.is_ascii_digit() {
            value |= c - b'0';
        } else if (b'a'..=b'f').contains(&to_lower(c)) {
            value |= to_lower(c) - b'a' + 10;
        }
        self.advance(1);
        Ok(value)
    }
}

fn detect_keyword(literal: &[u8]) -> (Token, Vec<u8>) {
    match literal {
        b"true" => (Token::Boolean, b"true".to_vec()),
        b"false" => (Token::Boolean, b"false".to_vec()),
        b"ether" => (Token::Ether, Vec::new()),
        b"wei" => (Token::Wei, Vec::new()),
        b"left" => (Token::Left, Vec::new()),
        b"library" => (Token::Library, Vec::new()),
        b"right" => (Token::Right, Vec::new()),
        b"hex" => (Token::Hex, Vec::new()),
        b"FAILURE" => (Token::Failure, Vec::new()),
        b"gas" => (Token::Gas, Vec::new()),
        other => (Token::Identifier, other.to_vec()),
    }
}
