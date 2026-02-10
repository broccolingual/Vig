use crate::lexer::{Lexer, Span, Token, TokenKind};

/// ポートの方向
#[derive(Debug, Clone, PartialEq)]
pub enum PortDirection {
    In,
    Out,
    Inout,
    Buffer,
}

/// VHDLの型参照（簡易）
#[derive(Debug, Clone, PartialEq)]
pub enum VhdlType {
    StdLogic,
    StdLogicVector { high: i64, low: i64 },
    Integer,
    Boolean,
    Other(String),
}

/// ポート定義
#[derive(Debug, Clone, PartialEq)]
pub struct PortDef {
    pub name: String,
    pub direction: PortDirection,
    pub vhdl_type: VhdlType,
    pub span: Span,
}

/// シグナル定義
#[derive(Debug, Clone, PartialEq)]
pub struct SignalDef {
    pub name: String,
    pub vhdl_type: VhdlType,
    pub default_value: Option<String>,
    pub span: Span,
}

/// エンティティ定義
#[derive(Debug, Clone, PartialEq)]
pub struct EntityDef {
    pub name: String,
    pub ports: Vec<PortDef>,
    pub span: Span,
}

/// アーキテクチャ定義
#[derive(Debug, Clone, PartialEq)]
pub struct ArchitectureDef {
    pub name: String,
    pub entity_name: String,
    pub signals: Vec<SignalDef>,
    pub span: Span,
}

/// 意味解析の結果
#[derive(Debug, Clone, PartialEq)]
pub struct AnalyzeResult {
    pub entities: Vec<EntityDef>,
    pub architectures: Vec<ArchitectureDef>,
}

impl std::fmt::Display for AnalyzeResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for entity in &self.entities {
            writeln!(f, "Entity: {}", entity.name)?;
            for port in &entity.ports {
                writeln!(
                    f,
                    "  Port: {} : {:?} {:?}",
                    port.name, port.direction, port.vhdl_type
                )?;
            }
        }
        for arch in &self.architectures {
            writeln!(f, "Architecture: {} of {}", arch.name, arch.entity_name)?;
            for sig in &arch.signals {
                write!(f, "  Signal: {} : {:?}", sig.name, sig.vhdl_type)?;
                if let Some(v) = &sig.default_value {
                    write!(f, " := {}", v)?;
                }
                writeln!(f)?;
            }
        }
        Ok(())
    }
}

/// 解析エラー
#[derive(Debug, Clone, PartialEq)]
pub struct AnalyzeError {
    pub message: String,
    pub span: Span,
}

impl AnalyzeError {
    pub fn new(message: impl Into<String>, span: Span) -> Self {
        Self {
            message: message.into(),
            span,
        }
    }
}

impl std::fmt::Display for AnalyzeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} at position {}-{}",
            self.message, self.span.start, self.span.end
        )
    }
}

impl std::error::Error for AnalyzeError {}

/// Eof用のセンチネルトークン
fn eof_token() -> Token {
    Token::new(TokenKind::Eof, Span::new(0, 0), String::new())
}

/// 意味解析器
pub struct Analyzer {
    tokens: Vec<Token>,
    pos: usize,
}

impl Analyzer {
    /// トークン列からAnalyzerを作成（Commentは除外）
    pub fn new(tokens: Vec<Token>) -> Self {
        let tokens: Vec<Token> = tokens
            .into_iter()
            .filter(|t| t.kind != TokenKind::Comment && t.kind != TokenKind::Eof)
            .collect();
        Self { tokens, pos: 0 }
    }

    /// 解析を実行
    pub fn analyze(&mut self) -> Result<AnalyzeResult, AnalyzeError> {
        let mut entities = Vec::new();
        let mut architectures = Vec::new();

        while self.current().kind != TokenKind::Eof {
            match self.current().kind {
                TokenKind::Entity => {
                    entities.push(self.parse_entity()?);
                }
                TokenKind::Architecture => {
                    architectures.push(self.parse_architecture()?);
                }
                _ => {
                    self.advance();
                }
            }
        }

        Ok(AnalyzeResult {
            entities,
            architectures,
        })
    }

    // --- トークン操作 ---

    fn current(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&EOF_SENTINEL)
    }

    fn peek(&self, offset: usize) -> &Token {
        self.tokens.get(self.pos + offset).unwrap_or(&EOF_SENTINEL)
    }

    fn advance(&mut self) {
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
    }

    fn expect(&mut self, kind: TokenKind) -> Result<Token, AnalyzeError> {
        let token = self.current().clone();
        if token.kind == kind {
            self.advance();
            Ok(token)
        } else {
            Err(AnalyzeError::new(
                format!(
                    "expected {:?}, found {:?} '{}'",
                    kind, token.kind, token.text
                ),
                token.span,
            ))
        }
    }

    fn eat(&mut self, kind: TokenKind) -> bool {
        if self.current().kind == kind {
            self.advance();
            true
        } else {
            false
        }
    }

    fn skip_until(&mut self, kinds: &[TokenKind]) {
        while self.current().kind != TokenKind::Eof {
            if kinds.contains(&self.current().kind) {
                return;
            }
            self.advance();
        }
    }

    // --- Entity 解析 ---

    fn parse_entity(&mut self) -> Result<EntityDef, AnalyzeError> {
        let start = self.current().span;
        self.expect(TokenKind::Entity)?;
        let name = self.expect(TokenKind::Identifier)?.text;
        self.expect(TokenKind::Is)?;

        let mut ports = Vec::new();
        if self.current().kind == TokenKind::Port {
            self.advance(); // port
            self.expect(TokenKind::LeftParen)?;
            ports = self.parse_port_list()?;
            self.expect(TokenKind::RightParen)?;
            self.expect(TokenKind::Semicolon)?;
        }

        // end [entity] [name] ;
        self.skip_until(&[TokenKind::Semicolon]);
        let end = self.current().span;
        self.advance(); // ;

        Ok(EntityDef {
            name,
            ports,
            span: Span::new(start.start, end.end),
        })
    }

    fn parse_port_list(&mut self) -> Result<Vec<PortDef>, AnalyzeError> {
        let mut ports = Vec::new();

        while self.current().kind != TokenKind::RightParen && self.current().kind != TokenKind::Eof
        {
            let mut group = self.parse_port_group()?;
            ports.append(&mut group);
            self.eat(TokenKind::Semicolon);
        }

        Ok(ports)
    }

    fn parse_port_group(&mut self) -> Result<Vec<PortDef>, AnalyzeError> {
        let span = self.current().span;
        let mut names = Vec::new();

        names.push(self.expect(TokenKind::Identifier)?.text);
        while self.eat(TokenKind::Comma) {
            names.push(self.expect(TokenKind::Identifier)?.text);
        }

        self.expect(TokenKind::Colon)?;
        let direction = self.parse_direction()?;
        let vhdl_type = self.parse_type()?;

        Ok(names
            .into_iter()
            .map(|n| PortDef {
                name: n,
                direction: direction.clone(),
                vhdl_type: vhdl_type.clone(),
                span,
            })
            .collect())
    }

    fn parse_direction(&mut self) -> Result<PortDirection, AnalyzeError> {
        let token = self.current().clone();
        match token.kind {
            TokenKind::In => {
                self.advance();
                Ok(PortDirection::In)
            }
            TokenKind::Out => {
                self.advance();
                Ok(PortDirection::Out)
            }
            TokenKind::Inout => {
                self.advance();
                Ok(PortDirection::Inout)
            }
            TokenKind::Buffer => {
                self.advance();
                Ok(PortDirection::Buffer)
            }
            _ => Err(AnalyzeError::new(
                format!("expected port direction, found '{}'", token.text),
                token.span,
            )),
        }
    }

    // --- 型の解析 ---

    fn parse_type(&mut self) -> Result<VhdlType, AnalyzeError> {
        let token = self.current().clone();
        match token.kind {
            TokenKind::StdLogic => {
                self.advance();
                Ok(VhdlType::StdLogic)
            }
            TokenKind::StdLogicVector => {
                self.advance();
                if self.current().kind == TokenKind::LeftParen {
                    self.advance(); // (
                    let high: i64 = self.expect(TokenKind::Number)?.text.parse().unwrap_or(0);
                    // downto or to
                    if self.current().kind == TokenKind::Downto
                        || self.current().kind == TokenKind::To
                    {
                        self.advance();
                    }
                    let low: i64 = self.expect(TokenKind::Number)?.text.parse().unwrap_or(0);
                    self.expect(TokenKind::RightParen)?;
                    Ok(VhdlType::StdLogicVector { high, low })
                } else {
                    Ok(VhdlType::StdLogicVector { high: 0, low: 0 })
                }
            }
            TokenKind::Integer => {
                self.advance();
                Ok(VhdlType::Integer)
            }
            TokenKind::Boolean => {
                self.advance();
                Ok(VhdlType::Boolean)
            }
            TokenKind::Identifier => {
                self.advance();
                Ok(VhdlType::Other(token.text))
            }
            _ => Err(AnalyzeError::new(
                format!("expected type, found '{}'", token.text),
                token.span,
            )),
        }
    }

    // --- Architecture 解析 ---

    fn parse_architecture(&mut self) -> Result<ArchitectureDef, AnalyzeError> {
        let start = self.current().span;
        self.expect(TokenKind::Architecture)?;
        let arch_name = self.expect(TokenKind::Identifier)?.text;
        self.expect(TokenKind::Of)?;
        let entity_name = self.expect(TokenKind::Identifier)?.text;
        self.expect(TokenKind::Is)?;

        let mut signals = Vec::new();

        // 宣言部: begin が来るまで signal を抽出
        while self.current().kind != TokenKind::Begin && self.current().kind != TokenKind::Eof {
            if self.current().kind == TokenKind::Signal {
                signals.push(self.parse_signal_decl()?);
            } else {
                self.advance();
            }
        }

        // begin 以降の本体をスキップ（end architecture を探す）
        self.skip_until_end_architecture();

        let end_pos = self.pos.saturating_sub(1);
        let end = self.tokens.get(end_pos).map(|t| t.span).unwrap_or(start);

        Ok(ArchitectureDef {
            name: arch_name,
            entity_name,
            signals,
            span: Span::new(start.start, end.end),
        })
    }

    fn skip_until_end_architecture(&mut self) {
        while self.current().kind != TokenKind::Eof {
            if self.current().kind == TokenKind::End && self.peek(1).kind == TokenKind::Architecture
            {
                self.skip_until(&[TokenKind::Semicolon]);
                self.advance(); // ;
                return;
            }
            self.advance();
        }
    }

    fn parse_signal_decl(&mut self) -> Result<SignalDef, AnalyzeError> {
        let start = self.current().span;
        self.expect(TokenKind::Signal)?;
        let name = self.expect(TokenKind::Identifier)?.text;
        self.expect(TokenKind::Colon)?;
        let vhdl_type = self.parse_type()?;

        let default_value = if self.current().kind == TokenKind::Assignment {
            self.advance(); // :=
            Some(self.parse_default_value())
        } else {
            None
        };

        let end = self.current().span;
        self.expect(TokenKind::Semicolon)?;

        Ok(SignalDef {
            name,
            vhdl_type,
            default_value,
            span: Span::new(start.start, end.end),
        })
    }

    fn parse_default_value(&mut self) -> String {
        let mut parts = Vec::new();
        while self.current().kind != TokenKind::Semicolon && self.current().kind != TokenKind::Eof {
            parts.push(self.current().text.clone());
            self.advance();
        }
        parts.join(" ")
    }
}

/// Eofセンチネル（borrowの都合でstaticに保持）
static EOF_SENTINEL: std::sync::LazyLock<Token> = std::sync::LazyLock::new(eof_token);

/// ソースコードから直接解析する便利関数
pub fn analyze_vhdl(source: &str) -> Result<AnalyzeResult, AnalyzeError> {
    let lexer = Lexer::new(source);
    let tokens: Vec<Token> = lexer.filter_map(|r| r.ok()).collect();
    let mut analyzer = Analyzer::new(tokens);
    analyzer.analyze()
}
