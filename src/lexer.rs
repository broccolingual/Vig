/// VHDLのトークンの種類を表す列挙型
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // 識別子とリテラル
    Identifier,
    Number,
    BitStringLiteral, // B"1010", X"FF"など
    CharacterLiteral, // '0', '1'など
    StringLiteral,

    // VHDLキーワード
    Entity,
    Architecture,
    Port,
    Signal,
    Process,
    Begin,
    End,
    If,
    Then,
    Else,
    Elsif,
    Case,
    When,
    Is,
    Of,
    Others,
    Library,
    Use,
    In,
    Out,
    Inout,
    Buffer,
    Generic,
    Map,
    Component,
    To,
    Downto,

    // 型
    StdLogic,
    StdLogicVector,
    Integer,
    Boolean,

    // 演算子
    Assignment,  // :=
    Association, // =>
    Eq,          // =
    Neq,         // /=
    Lt,          // <
    Lte,         // <=
    Gt,          // >
    Gte,         // >=
    Plus,        // +
    Minus,       // -
    Star,        // *
    Slash,       // /
    Power,       // **
    Ampersand,   // &
    And,
    Or,
    Not,
    Xor,
    Nand,
    Nor,

    // 区切り文字
    LeftParen,  // (
    RightParen, // )
    Semicolon,  // ;
    Colon,      // :
    Comma,      // ,
    Dot,        // .
    Apostrophe, // '

    // 特殊トークン
    Comment,
    Eof,

    // エラー
    Unknown,
}

/// トークンの位置情報
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn len(&self) -> usize {
        self.end - self.start
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// トークン本体
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
    pub text: String,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span, text: String) -> Self {
        Self { kind, span, text }
    }
}

/// Lexerのエラー型
#[derive(Debug, Clone, PartialEq)]
pub struct LexError {
    pub message: String,
    pub span: Span,
}

impl LexError {
    pub fn new(message: impl Into<String>, span: Span) -> Self {
        Self {
            message: message.into(),
            span,
        }
    }
}

impl std::fmt::Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} at position {}-{}",
            self.message, self.span.start, self.span.end
        )
    }
}

impl std::error::Error for LexError {}

/// Lexer本体
///
/// VHDLソースコードをトークン列に分割します
pub struct Lexer<'source> {
    position: usize,
    chars: std::str::Chars<'source>,
    current_char: Option<char>,
}

impl<'source> Lexer<'source> {
    /// 新しいLexerを作成
    pub fn new(source: &'source str) -> Self {
        let mut chars = source.chars();
        let current_char = chars.next();

        Self {
            position: 0,
            chars,
            current_char,
        }
    }

    /// 記号表による記号のトークン化
    /// 戻り値: (TokenKind, 消費する文字数)
    fn try_symbol(&self, ch: char) -> Option<(TokenKind, usize)> {
        let next_ch = self.peek();

        // 2文字記号を優先的にチェック
        if let Some(next) = next_ch {
            match (ch, next) {
                (':', '=') => return Some((TokenKind::Assignment, 2)),
                ('=', '>') => return Some((TokenKind::Association, 2)),
                ('/', '=') => return Some((TokenKind::Neq, 2)),
                ('<', '=') => return Some((TokenKind::Lte, 2)),
                ('>', '=') => return Some((TokenKind::Gte, 2)),
                ('*', '*') => return Some((TokenKind::Power, 2)),
                _ => {}
            }
        }

        // 1文字記号
        match ch {
            ':' => Some((TokenKind::Colon, 1)),
            '=' => Some((TokenKind::Eq, 1)),
            '<' => Some((TokenKind::Lt, 1)),
            '>' => Some((TokenKind::Gt, 1)),
            '+' => Some((TokenKind::Plus, 1)),
            '-' => Some((TokenKind::Minus, 1)),
            '*' => Some((TokenKind::Star, 1)),
            '/' => Some((TokenKind::Slash, 1)),
            '&' => Some((TokenKind::Ampersand, 1)),
            '(' => Some((TokenKind::LeftParen, 1)),
            ')' => Some((TokenKind::RightParen, 1)),
            ';' => Some((TokenKind::Semicolon, 1)),
            ',' => Some((TokenKind::Comma, 1)),
            '.' => Some((TokenKind::Dot, 1)),
            _ => None,
        }
    }

    /// 文字列がVHDLキーワードかチェックして対応するTokenKindを返す
    fn keyword_or_identifier(text: &str) -> TokenKind {
        // VHDLは大文字小文字を区別しないため、小文字に統一して比較
        match text.to_lowercase().as_str() {
            "entity" => TokenKind::Entity,
            "architecture" => TokenKind::Architecture,
            "port" => TokenKind::Port,
            "signal" => TokenKind::Signal,
            "process" => TokenKind::Process,
            "begin" => TokenKind::Begin,
            "end" => TokenKind::End,
            "if" => TokenKind::If,
            "then" => TokenKind::Then,
            "else" => TokenKind::Else,
            "elsif" => TokenKind::Elsif,
            "case" => TokenKind::Case,
            "when" => TokenKind::When,
            "is" => TokenKind::Is,
            "of" => TokenKind::Of,
            "others" => TokenKind::Others,
            "library" => TokenKind::Library,
            "use" => TokenKind::Use,
            "in" => TokenKind::In,
            "out" => TokenKind::Out,
            "inout" => TokenKind::Inout,
            "buffer" => TokenKind::Buffer,
            "generic" => TokenKind::Generic,
            "map" => TokenKind::Map,
            "component" => TokenKind::Component,
            "to" => TokenKind::To,
            "downto" => TokenKind::Downto,
            "std_logic" => TokenKind::StdLogic,
            "std_logic_vector" => TokenKind::StdLogicVector,
            "integer" => TokenKind::Integer,
            "boolean" => TokenKind::Boolean,
            "and" => TokenKind::And,
            "or" => TokenKind::Or,
            "not" => TokenKind::Not,
            "xor" => TokenKind::Xor,
            "nand" => TokenKind::Nand,
            "nor" => TokenKind::Nor,
            _ => TokenKind::Identifier,
        }
    }

    /// 現在の文字を取得
    fn current(&self) -> Option<char> {
        self.current_char
    }

    /// 次の文字を先読み
    fn peek(&self) -> Option<char> {
        self.chars.clone().next()
    }

    /// 次の文字に進む
    fn advance(&mut self) {
        if let Some(ch) = self.current_char {
            self.position += ch.len_utf8();
            self.current_char = self.chars.next();
        }
    }

    /// 条件を満たす間、文字を消費し続ける
    fn consume_while<F>(&mut self, start: usize, predicate: F) -> (String, Span)
    where
        F: Fn(char) -> bool,
    {
        let mut result = String::new();

        while let Some(ch) = self.current() {
            if predicate(ch) {
                result.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        let span = Span::new(start, self.position);
        (result, span)
    }

    /// 空白文字をスキップ
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current() {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// 識別子またはキーワードをトークン化
    fn lex_identifier(&mut self, start: usize) -> Token {
        let (text, span) = self.consume_while(start, |ch| ch.is_alphanumeric() || ch == '_');

        let kind = Self::keyword_or_identifier(&text);
        Token::new(kind, span, text)
    }

    /// 数値をトークン化
    fn lex_number(&mut self, start: usize) -> Token {
        let (text, span) = self.consume_while(start, |ch| {
            ch.is_ascii_digit()
                || ch == '.'
                || ch == '_'
                || ch.is_ascii_lowercase() && "eE".contains(ch)
        });

        Token::new(TokenKind::Number, span, text)
    }

    /// VHDLコメント（-- から行末まで）をトークン化
    fn lex_comment(&mut self, start: usize) -> Token {
        self.advance(); // 2つ目の '-' をスキップ

        let (text, span) = self.consume_while(start, |ch| ch != '\n');

        Token::new(TokenKind::Comment, span, text)
    }

    /// 文字リテラルをトークン化 ('0', '1', 'X'など)
    fn lex_character(&mut self, start: usize) -> Result<Token, LexError> {
        self.advance(); // 開始の '\'' をスキップ
        let mut text = String::from("'");

        if let Some(ch) = self.current() {
            text.push(ch);
            self.advance();

            if let Some('\'') = self.current() {
                text.push('\'');
                self.advance();
                let span = Span::new(start, self.position);
                return Ok(Token::new(TokenKind::CharacterLiteral, span, text));
            }
        }

        let span = Span::new(start, self.position);
        Err(LexError::new("invalid character literal", span))
    }

    /// 文字列リテラルをトークン化
    fn lex_string_literal(&mut self, start: usize) -> Result<Token, LexError> {
        self.advance(); // 開始の '"' をスキップ
        let mut text = String::from("\"");

        while let Some(ch) = self.current() {
            text.push(ch);
            self.advance();

            if ch == '"' {
                let span = Span::new(start, self.position);
                return Ok(Token::new(TokenKind::StringLiteral, span, text));
            }
        }

        // 文字列が閉じられていない
        let span = Span::new(start, self.position);
        Err(LexError::new("unclosed string literal", span))
    }

    /// 次のトークンを取得
    pub fn next_token(&mut self) -> Result<Token, LexError> {
        self.skip_whitespace();

        let start = self.position;

        match self.current() {
            None => {
                let span = Span::new(start, start);
                Ok(Token::new(TokenKind::Eof, span, String::new()))
            }

            Some(ch) if ch.is_alphabetic() || ch == '_' => Ok(self.lex_identifier(start)),

            Some(ch) if ch.is_ascii_digit() => Ok(self.lex_number(start)),

            Some('"') => self.lex_string_literal(start),

            Some('\'') => self.lex_character(start),

            // コメント --
            Some('-') if self.peek() == Some('-') => Ok(self.lex_comment(start)),

            // 記号・演算子の処理（記号表を使用）
            Some(ch) => {
                if let Some((kind, len)) = self.try_symbol(ch) {
                    let mut text = String::new();
                    for _ in 0..len {
                        if let Some(c) = self.current() {
                            text.push(c);
                            self.advance();
                        }
                    }
                    let span = Span::new(start, self.position);
                    Ok(Token::new(kind, span, text))
                } else {
                    self.advance();
                    let span = Span::new(start, self.position);
                    Err(LexError::new(
                        format!("unexpected character: '{}'", ch),
                        span,
                    ))
                }
            }
        }
    }
}

/// LexerをIteratorとして扱えるようにする
impl<'source> Iterator for Lexer<'source> {
    type Item = Result<Token, LexError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_token() {
            Ok(token) if token.kind == TokenKind::Eof => None,
            result => Some(result),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vhdl_keywords() {
        let source = "entity architecture begin end signal";
        let mut lexer = Lexer::new(source);

        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Entity);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Architecture);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Begin);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::End);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Signal);
    }

    #[test]
    fn test_case_insensitive() {
        let source = "ENTITY Entity entity";
        let mut lexer = Lexer::new(source);

        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Entity);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Entity);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Entity);
    }

    #[test]
    fn test_operators() {
        let source = ":= => <= >= /= **";
        let mut lexer = Lexer::new(source);

        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Assignment);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Association);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Lte);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Gte);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Neq);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Power);
    }

    #[test]
    fn test_comment() {
        let source = "signal -- this is a comment\nport";
        let mut lexer = Lexer::new(source);

        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Signal);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Comment);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Port);
    }

    #[test]
    fn test_character_literal() {
        let source = "'0' '1' 'X'";
        let mut lexer = Lexer::new(source);

        let token1 = lexer.next_token().unwrap();
        assert_eq!(token1.kind, TokenKind::CharacterLiteral);
        assert_eq!(token1.text, "'0'");

        let token2 = lexer.next_token().unwrap();
        assert_eq!(token2.kind, TokenKind::CharacterLiteral);
        assert_eq!(token2.text, "'1'");

        let token3 = lexer.next_token().unwrap();
        assert_eq!(token3.kind, TokenKind::CharacterLiteral);
        assert_eq!(token3.text, "'X'");
    }

    #[test]
    fn test_string_literal() {
        let source = r#""hello" "std_logic""#;
        let mut lexer = Lexer::new(source);

        let token1 = lexer.next_token().unwrap();
        assert_eq!(token1.kind, TokenKind::StringLiteral);
        assert_eq!(token1.text, r#""hello""#);

        let token2 = lexer.next_token().unwrap();
        assert_eq!(token2.kind, TokenKind::StringLiteral);
        assert_eq!(token2.text, r#""std_logic""#);
    }

    #[test]
    fn test_simple_entity() {
        let source = r#"
entity test is
    port (
        clk : in std_logic;
        data : out std_logic_vector
    );
end entity;
        "#;

        let lexer = Lexer::new(source);
        let tokens: Vec<_> = lexer.collect::<Result<Vec<_>, _>>().unwrap();

        assert_eq!(tokens[0].kind, TokenKind::Entity);
        assert_eq!(tokens[1].kind, TokenKind::Identifier);
        assert_eq!(tokens[1].text, "test");
        assert_eq!(tokens[2].kind, TokenKind::Is);
        assert_eq!(tokens[3].kind, TokenKind::Port);
    }

    #[test]
    fn test_signal_assignment() {
        let source = "signal_name <= '1';";
        let mut lexer = Lexer::new(source);

        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Identifier);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Lte);
        assert_eq!(
            lexer.next_token().unwrap().kind,
            TokenKind::CharacterLiteral
        );
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Semicolon);
    }

    #[test]
    fn test_logical_operators() {
        let source = "a and b or not c";
        let mut lexer = Lexer::new(source);

        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Identifier);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::And);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Identifier);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Or);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Not);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Identifier);
    }
}
