// ============================================================
// RUST-- LANGUAGE CORE
// Лексер, Парсер, Компилятор (в байткод), VM, Декомпилятор
// ============================================================

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

// ======================== TOKENS ============================

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    IntLit(i64),
    StrLit(String),
    BoolLit(bool),
    Ident(String),

    // Keywords
    Let,
    Fn,
    If,
    Else,
    Loop,
    Break,
    Return,
    Print,

    // Types
    TInt,
    TStr,
    TBool,

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Assign,
    Eq,
    Neq,
    Lt,
    Gt,
    Lte,
    Gte,
    And,
    Or,
    Not,

    // Delimiters
    LParen,
    RParen,
    LBrace,
    RBrace,
    Comma,
    Semicolon,
    Colon,

    // Special
    Eof,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Token::IntLit(n) => write!(f, "{}", n),
            Token::StrLit(s) => write!(f, "\"{}\"", s),
            Token::BoolLit(b) => write!(f, "{}", b),
            Token::Ident(s) => write!(f, "{}", s),
            Token::Let => write!(f, "let"),
            Token::Fn => write!(f, "fn"),
            Token::If => write!(f, "if"),
            Token::Else => write!(f, "else"),
            Token::Loop => write!(f, "loop"),
            Token::Break => write!(f, "break"),
            Token::Return => write!(f, "return"),
            Token::Print => write!(f, "print"),
            Token::TInt => write!(f, "int"),
            Token::TStr => write!(f, "str"),
            Token::TBool => write!(f, "bool"),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Star => write!(f, "*"),
            Token::Slash => write!(f, "/"),
            Token::Percent => write!(f, "%"),
            Token::Assign => write!(f, "="),
            Token::Eq => write!(f, "=="),
            Token::Neq => write!(f, "!="),
            Token::Lt => write!(f, "<"),
            Token::Gt => write!(f, ">"),
            Token::Lte => write!(f, "<="),
            Token::Gte => write!(f, ">="),
            Token::And => write!(f, "&&"),
            Token::Or => write!(f, "||"),
            Token::Not => write!(f, "!"),
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
            Token::LBrace => write!(f, "{{"),
            Token::RBrace => write!(f, "}}"),
            Token::Comma => write!(f, ","),
            Token::Semicolon => write!(f, ";"),
            Token::Colon => write!(f, ":"),
            Token::Eof => write!(f, "EOF"),
        }
    }
}

// ======================== LEXER =============================

pub struct Lexer {
    input: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
}

#[derive(Debug, Clone)]
pub struct LexError {
    pub msg: String,
    pub line: usize,
    pub col: usize,
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[Lex Error] Строка {}, столбец {}: {}", self.line, self.col, self.msg)
    }
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Lexer {
            input: input.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    fn peek(&self) -> char {
        if self.pos < self.input.len() {
            self.input[self.pos]
        } else {
            '\0'
        }
    }

    fn peek_next(&self) -> char {
        if self.pos + 1 < self.input.len() {
            self.input[self.pos + 1]
        } else {
            '\0'
        }
    }

    fn advance(&mut self) -> char {
        let c = self.peek();
        self.pos += 1;
        if c == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        c
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.input.len() && self.peek().is_whitespace() {
            self.advance();
        }
    }

    fn skip_comment(&mut self) {
        if self.peek() == '/' && self.peek_next() == '/' {
            while self.pos < self.input.len() && self.peek() != '\n' {
                self.advance();
            }
        }
    }

    fn read_string(&mut self) -> Result<Token, LexError> {
        self.advance(); // skip opening "
        let mut s = String::new();
        loop {
            if self.pos >= self.input.len() {
                return Err(LexError {
                    msg: "Незакрытая строка".into(),
                    line: self.line,
                    col: self.col,
                });
            }
            let c = self.advance();
            if c == '"' {
                break;
            }
            if c == '\\' {
                let esc = self.advance();
                match esc {
                    'n' => s.push('\n'),
                    't' => s.push('\t'),
                    '\\' => s.push('\\'),
                    '"' => s.push('"'),
                    _ => s.push(esc),
                }
            } else {
                s.push(c);
            }
        }
        Ok(Token::StrLit(s))
    }

    fn read_number(&mut self) -> Token {
        let mut num = String::new();
        while self.pos < self.input.len() && self.peek().is_ascii_digit() {
            num.push(self.advance());
        }
        Token::IntLit(num.parse().unwrap())
    }

    fn read_ident(&mut self) -> Token {
        let mut ident = String::new();
        while self.pos < self.input.len()
            && (self.peek().is_alphanumeric() || self.peek() == '_')
        {
            ident.push(self.advance());
        }
        match ident.as_str() {
            "let" => Token::Let,
            "fn" => Token::Fn,
            "if" => Token::If,
            "else" => Token::Else,
            "loop" => Token::Loop,
            "break" => Token::Break,
            "return" => Token::Return,
            "print" => Token::Print,
            "int" => Token::TInt,
            "str" => Token::TStr,
            "bool" => Token::TBool,
            "true" => Token::BoolLit(true),
            "false" => Token::BoolLit(false),
            _ => Token::Ident(ident),
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens = Vec::new();
        loop {
            self.skip_whitespace();
            self.skip_comment();
            self.skip_whitespace();

            if self.pos >= self.input.len() {
                tokens.push(Token::Eof);
                break;
            }

            let c = self.peek();

            let token = match c {
                '"' => self.read_string()?,
                '0'..='9' => self.read_number(),
                'a'..='z' | 'A'..='Z' | '_' => self.read_ident(),
                '+' => { self.advance(); Token::Plus }
                '-' => { self.advance(); Token::Minus }
                '*' => { self.advance(); Token::Star }
                '/' => {
                    if self.peek_next() == '/' {
                        self.skip_comment();
                        continue;
                    }
                    self.advance();
                    Token::Slash
                }
                '%' => { self.advance(); Token::Percent }
                '=' => {
                    self.advance();
                    if self.peek() == '=' {
                        self.advance();
                        Token::Eq
                    } else {
                        Token::Assign
                    }
                }
                '!' => {
                    self.advance();
                    if self.peek() == '=' {
                        self.advance();
                        Token::Neq
                    } else {
                        Token::Not
                    }
                }
                '<' => {
                    self.advance();
                    if self.peek() == '=' {
                        self.advance();
                        Token::Lte
                    } else {
                        Token::Lt
                    }
                }
                '>' => {
                    self.advance();
                    if self.peek() == '=' {
                        self.advance();
                        Token::Gte
                    } else {
                        Token::Gt
                    }
                }
                '&' => {
                    self.advance();
                    if self.peek() == '&' {
                        self.advance();
                        Token::And
                    } else {
                        return Err(LexError {
                            msg: "Ожидалось '&&'".into(),
                            line: self.line,
                            col: self.col,
                        });
                    }
                }
                '|' => {
                    self.advance();
                    if self.peek() == '|' {
                        self.advance();
                        Token::Or
                    } else {
                        return Err(LexError {
                            msg: "Ожидалось '||'".into(),
                            line: self.line,
                            col: self.col,
                        });
                    }
                }
                '(' => { self.advance(); Token::LParen }
                ')' => { self.advance(); Token::RParen }
                '{' => { self.advance(); Token::LBrace }
                '}' => { self.advance(); Token::RBrace }
                ',' => { self.advance(); Token::Comma }
                ';' => { self.advance(); Token::Semicolon }
                ':' => { self.advance(); Token::Colon }
                _ => {
                    return Err(LexError {
                        msg: format!("Неожиданный символ: '{}'", c),
                        line: self.line,
                        col: self.col,
                    });
                }
            };

            tokens.push(token);
        }

        Ok(tokens)
    }
}

// ======================== AST ===============================

#[derive(Debug, Clone)]
pub enum Expr {
    IntLit(i64),
    StrLit(String),
    BoolLit(bool),
    Ident(String),
    BinOp {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    UnaryOp {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    Call {
        name: String,
        args: Vec<Expr>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Neq,
    Lt,
    Gt,
    Lte,
    Gte,
    And,
    Or,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum UnaryOp {
    Neg,
    Not,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Let {
        name: String,
        value: Expr,
    },
    Assign {
        name: String,
        value: Expr,
    },
    Print(Expr),
    If {
        condition: Expr,
        then_block: Vec<Stmt>,
        else_block: Option<Vec<Stmt>>,
    },
    Loop {
        body: Vec<Stmt>,
    },
    Break,
    FnDef {
        name: String,
        params: Vec<String>,
        body: Vec<Stmt>,
    },
    Return(Option<Expr>),
    ExprStmt(Expr),
}

// ======================== PARSER ============================

pub struct Parser {
    pub tokens: Vec<Token>,
    pub pos: usize,
}

#[derive(Debug, Clone)]
pub struct ParseError {
    pub msg: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[Parse Error] {}", self.msg)
    }
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    fn peek(&self) -> &Token {
        if self.pos < self.tokens.len() {
            &self.tokens[self.pos]
        } else {
            &Token::Eof
        }
    }

    fn advance(&mut self) -> Token {
        let tok = self.tokens[self.pos].clone();
        self.pos += 1;
        tok
    }

    fn expect(&mut self, expected: &Token) -> Result<(), ParseError> {
        let tok = self.advance();
        if std::mem::discriminant(&tok) == std::mem::discriminant(expected) {
            Ok(())
        } else {
            Err(ParseError {
                msg: format!("Ожидалось '{}', получено '{}'", expected, tok),
            })
        }
    }

    pub fn parse_program(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut stmts = Vec::new();
        while *self.peek() != Token::Eof {
            stmts.push(self.parse_stmt()?);
        }
        Ok(stmts)
    }

    fn parse_stmt(&mut self) -> Result<Stmt, ParseError> {
        match self.peek().clone() {
            Token::Let => self.parse_let(),
            Token::Fn => self.parse_fn_def(),
            Token::If => self.parse_if(),
            Token::Loop => self.parse_loop(),
            Token::Break => {
                self.advance();
                self.expect(&Token::Semicolon)?;
                Ok(Stmt::Break)
            }
            Token::Return => self.parse_return(),
            Token::Print => self.parse_print(),
            Token::Ident(_) => {
                // Could be assignment or expression statement
                let name = if let Token::Ident(n) = self.peek().clone() {
                    n
                } else {
                    unreachable!()
                };

                // Look ahead for assignment
                if self.pos + 1 < self.tokens.len() && self.tokens[self.pos + 1] == Token::Assign {
                    self.advance(); // ident
                    self.advance(); // =
                    let value = self.parse_expr()?;
                    self.expect(&Token::Semicolon)?;
                    Ok(Stmt::Assign { name, value })
                } else {
                    let expr = self.parse_expr()?;
                    self.expect(&Token::Semicolon)?;
                    Ok(Stmt::ExprStmt(expr))
                }
            }
            _ => {
                let expr = self.parse_expr()?;
                self.expect(&Token::Semicolon)?;
                Ok(Stmt::ExprStmt(expr))
            }
        }
    }

    fn parse_let(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // let
        let name = match self.advance() {
            Token::Ident(n) => n,
            t => return Err(ParseError {
                msg: format!("Ожидалось имя переменной, получено '{}'", t),
            }),
        };
        self.expect(&Token::Assign)?;
        let value = self.parse_expr()?;
        self.expect(&Token::Semicolon)?;
        Ok(Stmt::Let { name, value })
    }

    fn parse_print(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // print
        self.expect(&Token::LParen)?;
        let expr = self.parse_expr()?;
        self.expect(&Token::RParen)?;
        self.expect(&Token::Semicolon)?;
        Ok(Stmt::Print(expr))
    }

    fn parse_fn_def(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // fn
        let name = match self.advance() {
            Token::Ident(n) => n,
            t => return Err(ParseError {
                msg: format!("Ожидалось имя функции, получено '{}'", t),
            }),
        };
        self.expect(&Token::LParen)?;
        let mut params = Vec::new();
        while *self.peek() != Token::RParen {
            match self.advance() {
                Token::Ident(p) => params.push(p),
                t => return Err(ParseError {
                    msg: format!("Ожидалось имя параметра, получено '{}'", t),
                }),
            }
            if *self.peek() == Token::Comma {
                self.advance();
            }
        }
        self.expect(&Token::RParen)?;
        let body = self.parse_block()?;
        Ok(Stmt::FnDef { name, params, body })
    }

    fn parse_if(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // if
        let condition = self.parse_expr()?;
        let then_block = self.parse_block()?;
        let else_block = if *self.peek() == Token::Else {
            self.advance();
            if *self.peek() == Token::If {
                Some(vec![self.parse_if()?])
            } else {
                Some(self.parse_block()?)
            }
        } else {
            None
        };
        Ok(Stmt::If {
            condition,
            then_block,
            else_block,
        })
    }

    fn parse_loop(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // loop
        let body = self.parse_block()?;
        Ok(Stmt::Loop { body })
    }

    fn parse_return(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // return
        if *self.peek() == Token::Semicolon {
            self.advance();
            Ok(Stmt::Return(None))
        } else {
            let expr = self.parse_expr()?;
            self.expect(&Token::Semicolon)?;
            Ok(Stmt::Return(Some(expr)))
        }
    }

    fn parse_block(&mut self) -> Result<Vec<Stmt>, ParseError> {
        self.expect(&Token::LBrace)?;
        let mut stmts = Vec::new();
        while *self.peek() != Token::RBrace && *self.peek() != Token::Eof {
            stmts.push(self.parse_stmt()?);
        }
        self.expect(&Token::RBrace)?;
        Ok(stmts)
    }

    // Expression parsing with precedence climbing
    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_and_expr()?;
        while *self.peek() == Token::Or {
            self.advance();
            let right = self.parse_and_expr()?;
            left = Expr::BinOp {
                op: BinOp::Or,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_and_expr(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_comparison()?;
        while *self.peek() == Token::And {
            self.advance();
            let right = self.parse_comparison()?;
            left = Expr::BinOp {
                op: BinOp::And,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_additive()?;
        loop {
            let op = match self.peek() {
                Token::Eq => BinOp::Eq,
                Token::Neq => BinOp::Neq,
                Token::Lt => BinOp::Lt,
                Token::Gt => BinOp::Gt,
                Token::Lte => BinOp::Lte,
                Token::Gte => BinOp::Gte,
                _ => break,
            };
            self.advance();
            let right = self.parse_additive()?;
            left = Expr::BinOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_additive(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_multiplicative()?;
        loop {
            let op = match self.peek() {
                Token::Plus => BinOp::Add,
                Token::Minus => BinOp::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_multiplicative()?;
            left = Expr::BinOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_unary()?;
        loop {
            let op = match self.peek() {
                Token::Star => BinOp::Mul,
                Token::Slash => BinOp::Div,
                Token::Percent => BinOp::Mod,
                _ => break,
            };
            self.advance();
            let right = self.parse_unary()?;
            left = Expr::BinOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        match self.peek().clone() {
            Token::Minus => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::UnaryOp {
                    op: UnaryOp::Neg,
                    expr: Box::new(expr),
                })
            }
            Token::Not => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::UnaryOp {
                    op: UnaryOp::Not,
                    expr: Box::new(expr),
                })
            }
            _ => self.parse_primary(),
        }
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        match self.peek().clone() {
            Token::IntLit(n) => {
                self.advance();
                Ok(Expr::IntLit(n))
            }
            Token::StrLit(s) => {
                self.advance();
                Ok(Expr::StrLit(s))
            }
            Token::BoolLit(b) => {
                self.advance();
                Ok(Expr::BoolLit(b))
            }
            Token::Ident(name) => {
                self.advance();
                if *self.peek() == Token::LParen {
                    self.advance(); // (
                    let mut args = Vec::new();
                    while *self.peek() != Token::RParen {
                        args.push(self.parse_expr()?);
                        if *self.peek() == Token::Comma {
                            self.advance();
                        }
                    }
                    self.expect(&Token::RParen)?;
                    Ok(Expr::Call { name, args })
                } else {
                    Ok(Expr::Ident(name))
                }
            }
            Token::LParen => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect(&Token::RParen)?;
                Ok(expr)
            }
            t => Err(ParseError {
                msg: format!("Неожиданный токен в выражении: '{}'", t),
            }),
        }
    }
}

// ======================== BYTECODE ==========================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Instruction {
    // Stack operations
    PushInt(i64),
    PushStr(String),
    PushBool(bool),

    // Variables
    Store(String),      // pop value, store to var
    Load(String),       // push var value

    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Neg,

    // Comparison
    CmpEq,
    CmpNeq,
    CmpLt,
    CmpGt,
    CmpLte,
    CmpGte,

    // Logic
    LogicAnd,
    LogicOr,
    LogicNot,

    // Control flow
    Jump(usize),        // unconditional jump
    JumpIfFalse(usize), // pop, jump if false
    Call(String, usize), // function name, arg count
    Ret,

    // I/O
    Print,

    // Special
    Halt,
    Nop,
    LoopStart,          // marker for decompiler
    LoopEnd,            // marker for decompiler
    Break,              // runtime break
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledProgram {
    pub magic: [u8; 4],           // "RS--"
    pub version: u8,
    pub instructions: Vec<Instruction>,
    pub functions: HashMap<String, FuncInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuncInfo {
    pub start: usize,
    pub params: Vec<String>,
}

impl CompiledProgram {
    pub fn new() -> Self {
        CompiledProgram {
            magic: [b'R', b'S', b'-', b'-'],
            version: 1,
            instructions: Vec::new(),
            functions: HashMap::new(),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        bincode_serialize(self)
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self, String> {
        bincode_deserialize(data)
    }
}

// Simple serialization (without external bincode dependency - using serde_json internally)
fn bincode_serialize(prog: &CompiledProgram) -> Vec<u8> {
    let json = serde_json::to_string(prog).unwrap();
    let mut result = Vec::new();
    // Magic header
    result.extend_from_slice(&prog.magic);
    result.push(prog.version);
    // JSON payload
    let json_bytes = json.as_bytes();
    let len = json_bytes.len() as u32;
    result.extend_from_slice(&len.to_le_bytes());
    result.extend_from_slice(json_bytes);
    result
}

fn bincode_deserialize(data: &[u8]) -> Result<CompiledProgram, String> {
    if data.len() < 9 {
        return Err("Файл слишком мал".into());
    }
    if &data[0..4] != b"RS--" {
        return Err("Неверный формат файла (ожидается RS--)".into());
    }
    let _version = data[4];
    let len = u32::from_le_bytes([data[5], data[6], data[7], data[8]]) as usize;
    if data.len() < 9 + len {
        return Err("Файл повреждён".into());
    }
    let json_str = std::str::from_utf8(&data[9..9 + len])
        .map_err(|e| format!("UTF-8 ошибка: {}", e))?;
    let prog: CompiledProgram = serde_json::from_str(json_str)
        .map_err(|e| format!("Ошибка десериализации: {}", e))?;
    Ok(prog)
}

// ======================== COMPILER ==========================

pub struct Compiler {
    program: CompiledProgram,
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            program: CompiledProgram::new(),
        }
    }

    fn emit(&mut self, instr: Instruction) -> usize {
        let idx = self.program.instructions.len();
        self.program.instructions.push(instr);
        idx
    }

    fn current_pos(&self) -> usize {
        self.program.instructions.len()
    }

    fn patch_jump(&mut self, idx: usize, target: usize) {
        match &mut self.program.instructions[idx] {
            Instruction::Jump(ref mut t) => *t = target,
            Instruction::JumpIfFalse(ref mut t) => *t = target,
            _ => {}
        }
    }

    pub fn compile(&mut self, stmts: &[Stmt]) -> Result<CompiledProgram, String> {
        // First pass: collect function definitions
        for stmt in stmts {
            if let Stmt::FnDef { name, params, .. } = stmt {
                self.program.functions.insert(
                    name.clone(),
                    FuncInfo {
                        start: 0, // will be patched
                        params: params.clone(),
                    },
                );
            }
        }

        // Compile top-level (skip fn defs, they're compiled separately)
        for stmt in stmts {
            match stmt {
                Stmt::FnDef { .. } => {} // handled below
                _ => self.compile_stmt(stmt)?,
            }
        }
        self.emit(Instruction::Halt);

        // Compile function bodies
        for stmt in stmts {
            if let Stmt::FnDef { name, params, body } = stmt {
                let start = self.current_pos();
                if let Some(info) = self.program.functions.get_mut(name) {
                    info.start = start;
                }
                // Store params as local vars (they're pushed by caller)
                for param in params.iter().rev() {
                    self.emit(Instruction::Store(param.clone()));
                }
                for s in body {
                    self.compile_stmt(s)?;
                }
                self.emit(Instruction::Ret);
            }
        }

        Ok(self.program.clone())
    }

    fn compile_stmt(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::Let { name, value } => {
                self.compile_expr(value)?;
                self.emit(Instruction::Store(name.clone()));
            }
            Stmt::Assign { name, value } => {
                self.compile_expr(value)?;
                self.emit(Instruction::Store(name.clone()));
            }
            Stmt::Print(expr) => {
                self.compile_expr(expr)?;
                self.emit(Instruction::Print);
            }
            Stmt::If {
                condition,
                then_block,
                else_block,
            } => {
                self.compile_expr(condition)?;
                let jump_false = self.emit(Instruction::JumpIfFalse(0));

                for s in then_block {
                    self.compile_stmt(s)?;
                }

                if let Some(else_stmts) = else_block {
                    let jump_over = self.emit(Instruction::Jump(0));
                    let else_start = self.current_pos();
                    self.patch_jump(jump_false, else_start);

                    for s in else_stmts {
                        self.compile_stmt(s)?;
                    }
                    let end = self.current_pos();
                    self.patch_jump(jump_over, end);
                } else {
                    let end = self.current_pos();
                    self.patch_jump(jump_false, end);
                }
            }
            Stmt::Loop { body } => {
                let loop_start = self.current_pos();
                self.emit(Instruction::LoopStart);

                // We need to track break positions to patch later
                let mut break_positions = Vec::new();

                for s in body {
                    self.compile_loop_stmt(s, &mut break_positions)?;
                }

                self.emit(Instruction::Jump(loop_start));
                let loop_end = self.current_pos();
                self.emit(Instruction::LoopEnd);

                // Patch all breaks
                for bp in break_positions {
                    self.patch_jump(bp, loop_end);
                }
            }
            Stmt::Break => {
                // This should be handled by compile_loop_stmt
                self.emit(Instruction::Break);
            }
            Stmt::Return(expr) => {
                if let Some(e) = expr {
                    self.compile_expr(e)?;
                }
                self.emit(Instruction::Ret);
            }
            Stmt::ExprStmt(expr) => {
                self.compile_expr(expr)?;
                // Pop result (not used) - we just leave it, VM can handle
            }
            Stmt::FnDef { .. } => {} // handled at top level
        }
        Ok(())
    }

    fn compile_loop_stmt(
        &mut self,
        stmt: &Stmt,
        break_positions: &mut Vec<usize>,
    ) -> Result<(), String> {
        match stmt {
            Stmt::Break => {
                let pos = self.emit(Instruction::Jump(0)); // patched later
                break_positions.push(pos);
            }
            Stmt::If {
                condition,
                then_block,
                else_block,
            } => {
                self.compile_expr(condition)?;
                let jump_false = self.emit(Instruction::JumpIfFalse(0));

                for s in then_block {
                    self.compile_loop_stmt(s, break_positions)?;
                }

                if let Some(else_stmts) = else_block {
                    let jump_over = self.emit(Instruction::Jump(0));
                    let else_start = self.current_pos();
                    self.patch_jump(jump_false, else_start);

                    for s in else_stmts {
                        self.compile_loop_stmt(s, break_positions)?;
                    }
                    let end = self.current_pos();
                    self.patch_jump(jump_over, end);
                } else {
                    let end = self.current_pos();
                    self.patch_jump(jump_false, end);
                }
            }
            _ => self.compile_stmt(stmt)?,
        }
        Ok(())
    }

    fn compile_expr(&mut self, expr: &Expr) -> Result<(), String> {
        match expr {
            Expr::IntLit(n) => {
                self.emit(Instruction::PushInt(*n));
            }
            Expr::StrLit(s) => {
                self.emit(Instruction::PushStr(s.clone()));
            }
            Expr::BoolLit(b) => {
                self.emit(Instruction::PushBool(*b));
            }
            Expr::Ident(name) => {
                self.emit(Instruction::Load(name.clone()));
            }
            Expr::BinOp { op, left, right } => {
                self.compile_expr(left)?;
                self.compile_expr(right)?;
                match op {
                    BinOp::Add => self.emit(Instruction::Add),
                    BinOp::Sub => self.emit(Instruction::Sub),
                    BinOp::Mul => self.emit(Instruction::Mul),
                    BinOp::Div => self.emit(Instruction::Div),
                    BinOp::Mod => self.emit(Instruction::Mod),
                    BinOp::Eq => self.emit(Instruction::CmpEq),
                    BinOp::Neq => self.emit(Instruction::CmpNeq),
                    BinOp::Lt => self.emit(Instruction::CmpLt),
                    BinOp::Gt => self.emit(Instruction::CmpGt),
                    BinOp::Lte => self.emit(Instruction::CmpLte),
                    BinOp::Gte => self.emit(Instruction::CmpGte),
                    BinOp::And => self.emit(Instruction::LogicAnd),
                    BinOp::Or => self.emit(Instruction::LogicOr),
                };
            }
            Expr::UnaryOp { op, expr } => {
                self.compile_expr(expr)?;
                match op {
                    UnaryOp::Neg => self.emit(Instruction::Neg),
                    UnaryOp::Not => self.emit(Instruction::LogicNot),
                };
            }
            Expr::Call { name, args } => {
                for arg in args {
                    self.compile_expr(arg)?;
                }
                self.emit(Instruction::Call(name.clone(), args.len()));
            }
        }
        Ok(())
    }
}

// ======================== VM ================================

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Str(String),
    Bool(bool),
    Void,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Int(n) => write!(f, "{}", n),
            Value::Str(s) => write!(f, "{}", s),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Void => write!(f, "void"),
        }
    }
}

impl Value {
    fn as_bool(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Int(n) => *n != 0,
            Value::Str(s) => !s.is_empty(),
            Value::Void => false,
        }
    }

    fn as_int(&self) -> Result<i64, String> {
        match self {
            Value::Int(n) => Ok(*n),
            _ => Err(format!("Ожидалось целое число, получено {:?}", self)),
        }
    }
}

#[derive(Debug)]
struct CallFrame {
    return_addr: usize,
    locals: HashMap<String, Value>,
}

pub struct VM {
    program: CompiledProgram,
    stack: Vec<Value>,
    call_stack: Vec<CallFrame>,
    globals: HashMap<String, Value>,
    ip: usize,
    pub output: Vec<String>,
    max_steps: usize,
}

impl VM {
    pub fn new(program: CompiledProgram) -> Self {
        VM {
            program,
            stack: Vec::new(),
            call_stack: Vec::new(),
            globals: HashMap::new(),
            ip: 0,
            output: Vec::new(),
            max_steps: 1_000_000,
        }
    }

    fn get_var(&self, name: &str) -> Result<Value, String> {
        // Check local scope first
        if let Some(frame) = self.call_stack.last() {
            if let Some(val) = frame.locals.get(name) {
                return Ok(val.clone());
            }
        }
        // Then globals
        self.globals
            .get(name)
            .cloned()
            .ok_or_else(|| format!("Переменная '{}' не определена", name))
    }

    fn set_var(&mut self, name: String, value: Value) {
        if let Some(frame) = self.call_stack.last_mut() {
            frame.locals.insert(name, value);
        } else {
            self.globals.insert(name, value);
        }
    }

    fn push(&mut self, val: Value) {
        self.stack.push(val);
    }

    fn pop(&mut self) -> Result<Value, String> {
        self.stack.pop().ok_or_else(|| "Стек пуст".to_string())
    }

    pub fn run(&mut self) -> Result<(), String> {
        let mut steps = 0;

        while self.ip < self.program.instructions.len() {
            steps += 1;
            if steps > self.max_steps {
                return Err("Превышен лимит шагов (возможно бесконечный цикл)".into());
            }

            let instr = self.program.instructions[self.ip].clone();

            match instr {
                Instruction::PushInt(n) => self.push(Value::Int(n)),
                Instruction::PushStr(s) => self.push(Value::Str(s)),
                Instruction::PushBool(b) => self.push(Value::Bool(b)),

                Instruction::Store(name) => {
                    let val = self.pop()?;
                    self.set_var(name, val);
                }
                Instruction::Load(name) => {
                    let val = self.get_var(&name)?;
                    self.push(val);
                }

                Instruction::Add => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    match (&a, &b) {
                        (Value::Int(x), Value::Int(y)) => self.push(Value::Int(x + y)),
                        (Value::Str(x), Value::Str(y)) => {
                            self.push(Value::Str(format!("{}{}", x, y)))
                        }
                        (Value::Str(x), _) => {
                            self.push(Value::Str(format!("{}{}", x, b)))
                        }
                        (_, Value::Str(y)) => {
                            self.push(Value::Str(format!("{}{}", a, y)))
                        }
                        _ => return Err(format!("Нельзя сложить {:?} и {:?}", a, b)),
                    }
                }
                Instruction::Sub => {
                    let b = self.pop()?.as_int()?;
                    let a = self.pop()?.as_int()?;
                    self.push(Value::Int(a - b));
                }
                Instruction::Mul => {
                    let b = self.pop()?.as_int()?;
                    let a = self.pop()?.as_int()?;
                    self.push(Value::Int(a * b));
                }
                Instruction::Div => {
                    let b = self.pop()?.as_int()?;
                    let a = self.pop()?.as_int()?;
                    if b == 0 {
                        return Err("Деление на ноль".into());
                    }
                    self.push(Value::Int(a / b));
                }
                Instruction::Mod => {
                    let b = self.pop()?.as_int()?;
                    let a = self.pop()?.as_int()?;
                    if b == 0 {
                        return Err("Деление на ноль".into());
                    }
                    self.push(Value::Int(a % b));
                }
                Instruction::Neg => {
                    let a = self.pop()?.as_int()?;
                    self.push(Value::Int(-a));
                }

                Instruction::CmpEq => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(Value::Bool(a == b));
                }
                Instruction::CmpNeq => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(Value::Bool(a != b));
                }
                Instruction::CmpLt => {
                    let b = self.pop()?.as_int()?;
                    let a = self.pop()?.as_int()?;
                    self.push(Value::Bool(a < b));
                }
                Instruction::CmpGt => {
                    let b = self.pop()?.as_int()?;
                    let a = self.pop()?.as_int()?;
                    self.push(Value::Bool(a > b));
                }
                Instruction::CmpLte => {
                    let b = self.pop()?.as_int()?;
                    let a = self.pop()?.as_int()?;
                    self.push(Value::Bool(a <= b));
                }
                Instruction::CmpGte => {
                    let b = self.pop()?.as_int()?;
                    let a = self.pop()?.as_int()?;
                    self.push(Value::Bool(a >= b));
                }

                Instruction::LogicAnd => {
                    let b = self.pop()?.as_bool();
                    let a = self.pop()?.as_bool();
                    self.push(Value::Bool(a && b));
                }
                Instruction::LogicOr => {
                    let b = self.pop()?.as_bool();
                    let a = self.pop()?.as_bool();
                    self.push(Value::Bool(a || b));
                }
                Instruction::LogicNot => {
                    let a = self.pop()?.as_bool();
                    self.push(Value::Bool(!a));
                }

                Instruction::Jump(target) => {
                    self.ip = target;
                    continue;
                }
                Instruction::JumpIfFalse(target) => {
                    let val = self.pop()?;
                    if !val.as_bool() {
                        self.ip = target;
                        continue;
                    }
                }

                Instruction::Call(name, _arg_count) => {
                    let func = self.program.functions.get(&name)
                        .ok_or_else(|| format!("Функция '{}' не найдена", name))?
                        .clone();

                    self.call_stack.push(CallFrame {
                        return_addr: self.ip + 1,
                        locals: HashMap::new(),
                    });

                    self.ip = func.start;
                    continue;
                }

                Instruction::Ret => {
                    if let Some(frame) = self.call_stack.pop() {
                        self.ip = frame.return_addr;
                        continue;
                    } else {
                        // Return from main
                        return Ok(());
                    }
                }

                Instruction::Print => {
                    let val = self.pop()?;
                    let s = format!("{}", val);
                    self.output.push(s);
                }

                Instruction::Halt => return Ok(()),
                Instruction::Nop | Instruction::LoopStart | Instruction::LoopEnd => {}
                Instruction::Break => {
                    // Should have been compiled to Jump, but handle gracefully
                    return Err("Unexpected break instruction at runtime".into());
                }
            }

            self.ip += 1;
        }

        Ok(())
    }
}

// ======================== DECOMPILER ========================

pub struct Decompiler {
    program: CompiledProgram,
    indent: usize,
    output: String,
}

impl Decompiler {
    pub fn new(program: CompiledProgram) -> Self {
        Decompiler {
            program,
            indent: 0,
            output: String::new(),
        }
    }

    fn indent_str(&self) -> String {
        "    ".repeat(self.indent)
    }

    fn writeln(&mut self, s: &str) {
        self.output.push_str(&self.indent_str());
        self.output.push_str(s);
        self.output.push('\n');
    }

    pub fn decompile(&mut self) -> String {
        self.output.clear();
        self.output.push_str("// Декомпилированный код Rust--\n");
        self.output.push_str("// Восстановлено из байткода RS--\n\n");

        // Decompile functions
        for (name, info) in &self.program.functions.clone() {
            let params = info.params.join(", ");
            self.writeln(&format!("fn {}({}) {{", name, params));
            self.indent += 1;
            self.decompile_range(info.start, self.find_func_end(info.start));
            self.indent -= 1;
            self.writeln("}\n");
        }

        // Decompile main body
        self.writeln("// === main ===");
        let main_end = self.find_halt();
        self.decompile_range(0, main_end);

        self.output.clone()
    }

    fn find_halt(&self) -> usize {
        for (i, instr) in self.program.instructions.iter().enumerate() {
            if matches!(instr, Instruction::Halt) {
                return i;
            }
        }
        self.program.instructions.len()
    }

    fn find_func_end(&self, start: usize) -> usize {
        for i in start..self.program.instructions.len() {
            if matches!(self.program.instructions[i], Instruction::Ret) {
                return i;
            }
        }
        self.program.instructions.len()
    }

	fn decompile_range(&mut self, start: usize, end: usize) {
		let mut i = start;

		while i < end && i < self.program.instructions.len() {
			let instr = self.program.instructions[i].clone();
			match instr {
				Instruction::Store(ref name) => {
					let value = self.reconstruct_expr(i);
					self.writeln(&format!("let {} = {};", name, value));
				}
				Instruction::Print => {
					let expr = self.reconstruct_expr_before(i);
					self.writeln(&format!("print({});", expr));
				}
				Instruction::PushInt(_)
				| Instruction::PushStr(_)
				| Instruction::PushBool(_)
				| Instruction::Load(_)
				| Instruction::Add
				| Instruction::Sub
				| Instruction::Mul
				| Instruction::Div
				| Instruction::Mod
				| Instruction::Neg
				| Instruction::CmpEq
				| Instruction::CmpNeq
				| Instruction::CmpLt
				| Instruction::CmpGt
				| Instruction::CmpLte
				| Instruction::CmpGte
				| Instruction::LogicAnd
				| Instruction::LogicOr
				| Instruction::LogicNot => {
					// Часть выражения — пропускаем
				}
				Instruction::JumpIfFalse(target) => {
					let cond = self.reconstruct_expr_before(i);
					self.writeln(&format!("if {} {{", cond));
					self.indent += 1;

					if target > i + 1
						&& target <= end
						&& target >= 2
						&& matches!(
							self.program.instructions.get(target - 1),
							Some(Instruction::Jump(_))
						)
					{
						// if-else
						self.decompile_range(i + 1, target - 1);
						self.indent -= 1;
						self.writeln("} else {");
						self.indent += 1;

						let else_end = if let Some(Instruction::Jump(e)) =
							self.program.instructions.get(target - 1)
						{
							*e
						} else {
							target
						};

						self.decompile_range(target, else_end);
						self.indent -= 1;
						self.writeln("}");
						self.writeln("");
						i = else_end;
						continue;
					} else {
						// if без else
						self.decompile_range(i + 1, target);
						self.indent -= 1;
						self.writeln("}");
						self.writeln("");
						i = target;
						continue;
					}
				}
				Instruction::LoopStart => {
					self.writeln("loop {");
					self.indent += 1;
					let loop_end = self.find_loop_end(i);
					self.decompile_range(i + 1, loop_end);
					self.indent -= 1;
					self.writeln("}");
					self.writeln("");
					i = loop_end + 1;
					continue;
				}
				Instruction::Jump(target) => {
					if target < i {
						// обратный прыжок — конец loop, пропускаем
					} else {
						self.writeln("break;");
					}
				}
				Instruction::Call(ref name, _) => {
					let args = self.reconstruct_call_args(i);
					self.writeln(&format!("{}({});", name, args));
				}
				Instruction::Ret => {
					self.writeln("return;");
				}
				Instruction::Halt
				| Instruction::Nop
				| Instruction::LoopEnd
				| Instruction::Break => {}
			}
			i += 1;
		}
	}

    fn find_loop_end(&self, start: usize) -> usize {
        let mut depth = 0;
        for i in start..self.program.instructions.len() {
            match &self.program.instructions[i] {
                Instruction::LoopStart => depth += 1,
                Instruction::LoopEnd => {
                    depth -= 1;
                    if depth == 0 {
                        return i;
                    }
                }
                _ => {}
            }
        }
        self.program.instructions.len()
    }

    fn reconstruct_expr(&self, store_pos: usize) -> String {
        // Simple reconstruction: look backwards from store
        if store_pos == 0 {
            return "???".into();
        }
        self.reconstruct_expr_at(store_pos - 1).0
    }

    fn reconstruct_expr_before(&self, pos: usize) -> String {
        if pos == 0 {
            return "???".into();
        }
        self.reconstruct_expr_at(pos - 1).0
    }

    // Returns (expression_string, how_many_instructions_consumed)
	fn reconstruct_expr_at(&self, pos: usize) -> (String, usize) {
		if pos >= self.program.instructions.len() {
			return ("???".into(), 0);
		}
		match &self.program.instructions[pos] {
			Instruction::PushInt(n) => (format!("{}", n), 1),
			Instruction::PushStr(s) => (format!("\"{}\"", s), 1),
			Instruction::PushBool(b) => (format!("{}", b), 1),
			Instruction::Load(name) => (name.clone(), 1),
			Instruction::Add
			| Instruction::Sub
			| Instruction::Mul
			| Instruction::Div
			| Instruction::Mod
			| Instruction::CmpEq
			| Instruction::CmpNeq
			| Instruction::CmpLt
			| Instruction::CmpGt
			| Instruction::CmpLte
			| Instruction::CmpGte
			| Instruction::LogicAnd
			| Instruction::LogicOr => {
				let op_str = match &self.program.instructions[pos] {
					Instruction::Add => " + ",
					Instruction::Sub => " - ",
					Instruction::Mul => " * ",
					Instruction::Div => " / ",
					Instruction::Mod => " % ",
					Instruction::CmpEq => " == ",
					Instruction::CmpNeq => " != ",
					Instruction::CmpLt => " < ",
					Instruction::CmpGt => " > ",
					Instruction::CmpLte => " <= ",
					Instruction::CmpGte => " >= ",
					Instruction::LogicAnd => " && ",
					Instruction::LogicOr => " || ",
					_ => " ? ",
				};
				if pos >= 2 {
					let (right, r_consumed) = self.reconstruct_expr_at(pos - 1);
					if pos >= 1 + r_consumed {
						let (left, l_consumed) =
							self.reconstruct_expr_at(pos - 1 - r_consumed);
						(
							format!("{}{}{}", left, op_str, right),
							1 + r_consumed + l_consumed,
						)
					} else {
						(format!("?{}{}", op_str, right), 1 + r_consumed)
					}
				} else {
					(format!("?{}?", op_str), 1)
				}
			}
			Instruction::Neg => {
				if pos >= 1 {
					let (expr, consumed) = self.reconstruct_expr_at(pos - 1);
					(format!("-{}", expr), 1 + consumed)
				} else {
					("-?".into(), 1)
				}
			}
			Instruction::LogicNot => {
				if pos >= 1 {
					let (expr, consumed) = self.reconstruct_expr_at(pos - 1);
					(format!("!{}", expr), 1 + consumed)
				} else {
					("!?".into(), 1)
				}
			}
			_ => ("???".into(), 1),
		}
	}

    fn reconstruct_call_args(&self, call_pos: usize) -> String {
        if let Instruction::Call(_, arg_count) = &self.program.instructions[call_pos] {
            let mut args = Vec::new();
            let mut pos = call_pos;
            for _ in 0..*arg_count {
                if pos == 0 {
                    break;
                }
                pos -= 1;
                let (expr, _consumed) = self.reconstruct_expr_at(pos);
                args.push(expr);
            }
            args.reverse();
            args.join(", ")
        } else {
            String::new()
        }
    }
}

// ======================== HELPERS ===========================

/// Full pipeline: source -> compile -> bytes
pub fn compile_source(source: &str) -> Result<(CompiledProgram, Vec<u8>), String> {
    // Lex
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().map_err(|e| e.to_string())?;

    // Parse
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().map_err(|e| e.to_string())?;

    // Compile
    let mut compiler = Compiler::new();
    let program = compiler.compile(&ast)?;
    let bytes = program.to_bytes();

    Ok((program, bytes))
}

/// Run source directly
pub fn run_source(source: &str) -> Result<Vec<String>, String> {
    let (program, _) = compile_source(source)?;
    let mut vm = VM::new(program);
    vm.run()?;
    Ok(vm.output)
}

pub fn run_program(program: CompiledProgram) -> Result<Vec<String>, String> {
    let mut vm = VM::new(program);
    vm.run()?;
    Ok(vm.output)
}

/// Run from bytes
pub fn run_bytes(bytes: &[u8]) -> Result<Vec<String>, String> {
    let program = CompiledProgram::from_bytes(bytes)?;
    let mut vm = VM::new(program);
    vm.run()?;
    Ok(vm.output)
}

/// Decompile bytes to source
pub fn decompile_bytes(bytes: &[u8]) -> Result<String, String> {
    let program = CompiledProgram::from_bytes(bytes)?;
    let mut decompiler = Decompiler::new(program);
    Ok(decompiler.decompile())
}

/// Get bytecode listing
pub fn disassemble(program: &CompiledProgram) -> String {
    let mut out = String::new();
    out.push_str("=== Байткод RS-- ===\n\n");

    if !program.functions.is_empty() {
        out.push_str("Функции:\n");
        for (name, info) in &program.functions {
            out.push_str(&format!(
                "  {} (params: {:?}, start: {})\n",
                name, info.params, info.start
            ));
        }
        out.push('\n');
    }

    out.push_str("Инструкции:\n");
    for (i, instr) in program.instructions.iter().enumerate() {
        out.push_str(&format!("  {:04}: {:?}\n", i, instr));
    }

    out
}
// ======================== ТРАНСПИЛЕР RS-- → RUST ========================

pub struct Transpiler {
    indent: usize,
    output: String,
}

impl Transpiler {
    pub fn new() -> Self {
        Transpiler {
            indent: 0,
            output: String::new(),
        }
    }

    fn indent_str(&self) -> String {
        "    ".repeat(self.indent)
    }

    fn write(&mut self, s: &str) {
        self.output.push_str(&self.indent_str());
        self.output.push_str(s);
        self.output.push('\n');
    }

    pub fn transpile(&mut self, source: &str) -> Result<String, String> {
        // Парсим RS--
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().map_err(|e| e.to_string())?;
        let mut parser = Parser::new(tokens);
        let stmts = parser.parse_program().map_err(|e| e.to_string())?;

        self.output.clear();

        // Хедер
        self.write("#![allow(unused_variables, unused_mut, unused_assignments)]");
        self.write("");

        // Сначала собираем все функции
        let mut functions = Vec::new();
        let mut main_stmts = Vec::new();

        for stmt in &stmts {
            match stmt {
                Stmt::FnDef { .. } => functions.push(stmt.clone()),
                _ => main_stmts.push(stmt.clone()),
            }
        }

        // Генерируем вспомогательный макрос для print с конкатенацией
        self.write("// Вспомогательный трейт для конкатенации разных типов");
        self.write("trait RsmmDisplay {");
        self.indent += 1;
        self.write("fn to_rsmm_string(&self) -> String;");
        self.indent -= 1;
        self.write("}");
        self.write("");
        self.write("impl RsmmDisplay for i64 {");
        self.indent += 1;
        self.write("fn to_rsmm_string(&self) -> String { self.to_string() }");
        self.indent -= 1;
        self.write("}");
        self.write("");
        self.write("impl RsmmDisplay for String {");
        self.indent += 1;
        self.write("fn to_rsmm_string(&self) -> String { self.clone() }");
        self.indent -= 1;
        self.write("}");
        self.write("");
        self.write("impl RsmmDisplay for &str {");
        self.indent += 1;
        self.write("fn to_rsmm_string(&self) -> String { self.to_string() }");
        self.indent -= 1;
        self.write("}");
        self.write("");
        self.write("impl RsmmDisplay for bool {");
        self.indent += 1;
        self.write("fn to_rsmm_string(&self) -> String { self.to_string() }");
        self.indent -= 1;
        self.write("}");
        self.write("");

        // Макрос для конкатенации через +
        self.write("macro_rules! rsmm_add {");
        self.indent += 1;
        self.write("($a:expr, $b:expr) => {{");
        self.indent += 1;
        self.write("// Пробуем как числа, иначе как строки");
        self.write("rsmm_add_impl(&$a, &$b)");
        self.indent -= 1;
        self.write("}};");
        self.indent -= 1;
        self.write("}");
        self.write("");

        // Enum для динамических значений
        self.write("#[derive(Clone, Debug)]");
        self.write("enum RsmmVal {");
        self.indent += 1;
        self.write("Int(i64),");
        self.write("Str(String),");
        self.write("Bool(bool),");
        self.indent -= 1;
        self.write("}");
        self.write("");

        self.write("impl std::fmt::Display for RsmmVal {");
        self.indent += 1;
        self.write("fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {");
        self.indent += 1;
        self.write("match self {");
        self.indent += 1;
        self.write("RsmmVal::Int(n) => write!(f, \"{}\", n),");
        self.write("RsmmVal::Str(s) => write!(f, \"{}\", s),");
        self.write("RsmmVal::Bool(b) => write!(f, \"{}\", b),");
        self.indent -= 1;
        self.write("}");
        self.indent -= 1;
        self.write("}");
        self.indent -= 1;
        self.write("}");
        self.write("");

        // Операции над RsmmVal
        self.write("impl RsmmVal {");
        self.indent += 1;

        self.write("fn as_int(&self) -> i64 {");
        self.indent += 1;
        self.write("match self { RsmmVal::Int(n) => *n, _ => panic!(\"Ожидалось число\") }");
        self.indent -= 1;
        self.write("}");

        self.write("fn as_bool(&self) -> bool {");
        self.indent += 1;
        self.write("match self {");
        self.indent += 1;
        self.write("RsmmVal::Bool(b) => *b,");
        self.write("RsmmVal::Int(n) => *n != 0,");
        self.write("RsmmVal::Str(s) => !s.is_empty(),");
        self.indent -= 1;
        self.write("}");
        self.indent -= 1;
        self.write("}");

        self.indent -= 1;
        self.write("}");
        self.write("");

        // Реализация сложения
        self.write("fn rsmm_add_impl(a: &RsmmVal, b: &RsmmVal) -> RsmmVal {");
        self.indent += 1;
        self.write("match (a, b) {");
        self.indent += 1;
        self.write("(RsmmVal::Int(x), RsmmVal::Int(y)) => RsmmVal::Int(x + y),");
        self.write("(RsmmVal::Str(x), RsmmVal::Str(y)) => RsmmVal::Str(format!(\"{}{}\", x, y)),");
        self.write("(RsmmVal::Str(x), other) => RsmmVal::Str(format!(\"{}{}\", x, other)),");
        self.write("(other, RsmmVal::Str(y)) => RsmmVal::Str(format!(\"{}{}\", other, y)),");
        self.write("_ => RsmmVal::Str(format!(\"{}{}\", a, b)),");
        self.indent -= 1;
        self.write("}");
        self.indent -= 1;
        self.write("}");
        self.write("");

        self.write("fn rsmm_sub(a: &RsmmVal, b: &RsmmVal) -> RsmmVal {");
        self.indent += 1;
        self.write("RsmmVal::Int(a.as_int() - b.as_int())");
        self.indent -= 1;
        self.write("}");
        self.write("");

        self.write("fn rsmm_mul(a: &RsmmVal, b: &RsmmVal) -> RsmmVal {");
        self.indent += 1;
        self.write("RsmmVal::Int(a.as_int() * b.as_int())");
        self.indent -= 1;
        self.write("}");
        self.write("");

        self.write("fn rsmm_div(a: &RsmmVal, b: &RsmmVal) -> RsmmVal {");
        self.indent += 1;
        self.write("RsmmVal::Int(a.as_int() / b.as_int())");
        self.indent -= 1;
        self.write("}");
        self.write("");

        self.write("fn rsmm_mod(a: &RsmmVal, b: &RsmmVal) -> RsmmVal {");
        self.indent += 1;
        self.write("RsmmVal::Int(a.as_int() % b.as_int())");
        self.indent -= 1;
        self.write("}");
        self.write("");

        self.write("fn rsmm_eq(a: &RsmmVal, b: &RsmmVal) -> RsmmVal {");
        self.indent += 1;
        self.write("match (a, b) {");
        self.indent += 1;
        self.write("(RsmmVal::Int(x), RsmmVal::Int(y)) => RsmmVal::Bool(x == y),");
        self.write("(RsmmVal::Str(x), RsmmVal::Str(y)) => RsmmVal::Bool(x == y),");
        self.write("(RsmmVal::Bool(x), RsmmVal::Bool(y)) => RsmmVal::Bool(x == y),");
        self.write("_ => RsmmVal::Bool(false),");
        self.indent -= 1;
        self.write("}");
        self.indent -= 1;
        self.write("}");
        self.write("");

        self.write("fn rsmm_neq(a: &RsmmVal, b: &RsmmVal) -> RsmmVal {");
        self.indent += 1;
        self.write("RsmmVal::Bool(!rsmm_eq(a, b).as_bool())");
        self.indent -= 1;
        self.write("}");
        self.write("");

        self.write("fn rsmm_lt(a: &RsmmVal, b: &RsmmVal) -> RsmmVal { RsmmVal::Bool(a.as_int() < b.as_int()) }");
        self.write("fn rsmm_gt(a: &RsmmVal, b: &RsmmVal) -> RsmmVal { RsmmVal::Bool(a.as_int() > b.as_int()) }");
        self.write("fn rsmm_lte(a: &RsmmVal, b: &RsmmVal) -> RsmmVal { RsmmVal::Bool(a.as_int() <= b.as_int()) }");
        self.write("fn rsmm_gte(a: &RsmmVal, b: &RsmmVal) -> RsmmVal { RsmmVal::Bool(a.as_int() >= b.as_int()) }");
        self.write("");

        // Генерируем функции пользователя
        for func in &functions {
            self.transpile_stmt(func)?;
            self.output.push('\n');
        }

        // Генерируем main
        self.write("fn main() {");
        self.indent += 1;
        for stmt in &main_stmts {
            self.transpile_stmt(stmt)?;
        }
        self.indent -= 1;
        self.write("}");

        Ok(self.output.clone())
    }

    fn transpile_stmt(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::Let { name, value } => {
                let expr = self.transpile_expr(value)?;
                self.write(&format!("let mut {} = {};", name, expr));
            }
            Stmt::Assign { name, value } => {
                let expr = self.transpile_expr(value)?;
                self.write(&format!("{} = {};", name, expr));
            }
            Stmt::Print(expr) => {
                let e = self.transpile_expr(expr)?;
                self.write(&format!("println!(\"{{}}\", {});", e));
            }
            Stmt::If {
                condition,
                then_block,
                else_block,
            } => {
                let cond = self.transpile_expr(condition)?;
                self.write(&format!("if ({}).as_bool() {{", cond));
                self.indent += 1;
                for s in then_block {
                    self.transpile_stmt(s)?;
                }
                self.indent -= 1;
                if let Some(else_stmts) = else_block {
                    self.write("} else {");
                    self.indent += 1;
                    for s in else_stmts {
                        self.transpile_stmt(s)?;
                    }
                    self.indent -= 1;
                }
                self.write("}");
            }
            Stmt::Loop { body } => {
                self.write("loop {");
                self.indent += 1;
                for s in body {
                    self.transpile_stmt(s)?;
                }
                self.indent -= 1;
                self.write("}");
            }
            Stmt::Break => {
                self.write("break;");
            }
            Stmt::FnDef { name, params, body } => {
                let params_str = params
                    .iter()
                    .map(|p| format!("{}: RsmmVal", p))
                    .collect::<Vec<_>>()
                    .join(", ");
                self.write(&format!("fn {}({}) -> RsmmVal {{", name, params_str));
                self.indent += 1;
                // Делаем параметры мутабельными
                for p in params {
                    self.write(&format!("let mut {} = {};", p, p));
                }
                for s in body {
                    self.transpile_stmt(s)?;
                }
                self.indent -= 1;
                self.write("}");
            }
            Stmt::Return(expr) => {
                if let Some(e) = expr {
                    let val = self.transpile_expr(e)?;
                    self.write(&format!("return {};", val));
                } else {
                    self.write("return;");
                }
            }
            Stmt::ExprStmt(expr) => {
                let e = self.transpile_expr(expr)?;
                self.write(&format!("{};", e));
            }
        }
        Ok(())
    }

    fn transpile_expr(&self, expr: &Expr) -> Result<String, String> {
        match expr {
            Expr::IntLit(n) => Ok(format!("RsmmVal::Int({})", n)),
            Expr::StrLit(s) => Ok(format!("RsmmVal::Str(\"{}\".to_string())", s.replace('\\', "\\\\").replace('"', "\\\""))),
            Expr::BoolLit(b) => Ok(format!("RsmmVal::Bool({})", b)),
            Expr::Ident(name) => Ok(format!("{}.clone()", name)),
            Expr::BinOp { op, left, right } => {
                let l = self.transpile_expr(left)?;
                let r = self.transpile_expr(right)?;
                let func = match op {
                    BinOp::Add => "rsmm_add_impl",
                    BinOp::Sub => "rsmm_sub",
                    BinOp::Mul => "rsmm_mul",
                    BinOp::Div => "rsmm_div",
                    BinOp::Mod => "rsmm_mod",
                    BinOp::Eq => "rsmm_eq",
                    BinOp::Neq => "rsmm_neq",
                    BinOp::Lt => "rsmm_lt",
                    BinOp::Gt => "rsmm_gt",
                    BinOp::Lte => "rsmm_lte",
                    BinOp::Gte => "rsmm_gte",
                    BinOp::And => {
                        return Ok(format!(
                            "RsmmVal::Bool(({}).as_bool() && ({}).as_bool())",
                            l, r
                        ));
                    }
                    BinOp::Or => {
                        return Ok(format!(
                            "RsmmVal::Bool(({}).as_bool() || ({}).as_bool())",
                            l, r
                        ));
                    }
                };
                Ok(format!("{}(&{}, &{})", func, l, r))
            }
            Expr::UnaryOp { op, expr } => {
                let e = self.transpile_expr(expr)?;
                match op {
                    UnaryOp::Neg => Ok(format!("RsmmVal::Int(-({}).as_int())", e)),
                    UnaryOp::Not => Ok(format!("RsmmVal::Bool(!({}).as_bool())", e)),
                }
            }
            Expr::Call { name, args } => {
                let args_str = args
                    .iter()
                    .map(|a| self.transpile_expr(a))
                    .collect::<Result<Vec<_>, _>>()?
                    .join(", ");
                Ok(format!("{}({})", name, args_str))
            }
        }
    }
}

// ======================== СБОРКА EXE ЧЕРЕЗ CARGO ========================

pub fn transpile_to_rust(source: &str) -> Result<String, String> {
    let mut t = Transpiler::new();
    t.transpile(source)
}

pub fn build_exe(source: &str, output_path: &std::path::Path) -> Result<String, String> {
    let rust_code = transpile_to_rust(source)?;
    
    let temp_dir = std::env::temp_dir().join("rsmm_build");
    let src_dir = temp_dir.join("src");
    
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&src_dir)
        .map_err(|e| format!("Не могу создать папку: {}", e))?;
    
    let cargo_toml = r#"[package]
name = "rsmm_output"
version = "0.1.0"
edition = "2021"

[profile.release]
opt-level = "z"
lto = true
strip = true
"#;
    
    std::fs::write(temp_dir.join("Cargo.toml"), cargo_toml)
        .map_err(|e| format!("Не могу записать Cargo.toml: {}", e))?;
    
    std::fs::write(src_dir.join("main.rs"), &rust_code)
        .map_err(|e| format!("Не могу записать main.rs: {}", e))?;
    
    let output = std::process::Command::new("cargo")
        .arg("build")
        .arg("--release")
        .arg("--manifest-path")
        .arg(temp_dir.join("Cargo.toml"))
        .output()
        .map_err(|e| format!("Не могу запустить cargo: {}\nУбедитесь что Rust установлен!", e))?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let debug_path = temp_dir.join("src").join("main.rs");
        
        return Err(format!(
            "Ошибка компиляции Rust!\n\n\
             Сгенерированный код сохранён: {}\n\n\
             stderr:\n{}\n\nstdout:\n{}",
            debug_path.display(), stderr, stdout
        ));
    }
    
    #[cfg(windows)]
    let exe_name = "rsmm_output.exe";
    #[cfg(not(windows))]
    let exe_name = "rsmm_output";
    
    let built_exe = temp_dir.join("target").join("release").join(exe_name);
    
    std::fs::copy(&built_exe, output_path)
        .map_err(|e| format!("Не могу скопировать exe: {}", e))?;
    
    let exe_size = std::fs::metadata(output_path)
        .map(|m| m.len())
        .unwrap_or(0);
    
    // ← ДОБАВЛЕНО: очищаем temp
    let _ = std::fs::remove_dir_all(&temp_dir);
    
    Ok(format!(
        "✅ EXE создан успешно!\n\
         Путь: {}\n\
         Размер: {} KB",
        output_path.display(),
        exe_size / 1024,
    ))
}
// ======================== МУЛЬТИФАЙЛОВЫЙ ПРОЕКТ ========================

#[derive(Clone, Debug)]
pub struct ProjectFile {
    pub name: String,      // имя файла: "main.rsmm", "math.rsmm"
    pub content: String,   // содержимое
    pub is_main: bool,     // главный файл?
}

#[derive(Clone, Debug)]
pub struct RustDependency {
    pub name: String,       // имя крейта: "rand", "serde"
    pub version: String,    // версия: "0.8", "1.0"
    pub features: Vec<String>, // фичи: ["derive"]
}

impl RustDependency {
    pub fn new(name: &str, version: &str) -> Self {
        RustDependency {
            name: name.to_string(),
            version: version.to_string(),
            features: Vec::new(),
        }
    }

    pub fn with_features(name: &str, version: &str, features: Vec<&str>) -> Self {
        RustDependency {
            name: name.to_string(),
            version: version.to_string(),
            features: features.iter().map(|s| s.to_string()).collect(),
        }
    }

    pub fn to_toml_line(&self) -> String {
        if self.features.is_empty() {
            format!("{} = \"{}\"", self.name, self.version)
        } else {
            let feats = self
                .features
                .iter()
                .map(|f| format!("\"{}\"", f))
                .collect::<Vec<_>>()
                .join(", ");
            format!(
                "{} = {{ version = \"{}\", features = [{}] }}",
                self.name, self.version, feats
            )
        }
    }
}

#[derive(Clone, Debug)]
pub struct Project {
    pub name: String,
    pub files: Vec<ProjectFile>,
    pub dependencies: Vec<RustDependency>,
    pub rust_blocks: Vec<String>,  // блоки чистого Rust кода (use, extern)
}

impl Project {
    pub fn new(name: &str) -> Self {
        Project {
            name: name.to_string(),
            files: Vec::new(),
            dependencies: Vec::new(),
            rust_blocks: Vec::new(),
        }
    }

    pub fn add_file(&mut self, name: &str, content: &str, is_main: bool) {
        self.files.push(ProjectFile {
            name: name.to_string(),
            content: content.to_string(),
            is_main,
        });
    }

    pub fn add_dep(&mut self, name: &str, version: &str) {
        self.dependencies.push(RustDependency::new(name, version));
    }

    pub fn add_dep_with_features(&mut self, name: &str, version: &str, features: Vec<&str>) {
        self.dependencies
            .push(RustDependency::with_features(name, version, features));
    }
}

// ======================== РАСШИРЕННЫЙ ПАРСЕР ========================
// Новые конструкции:
//   use! rand;                    — добавить зависимость
//   use! serde "1.0" [derive];   — с версией и фичами
//   rust! { ... }                — блок чистого Rust кода
//   import "math.rsmm";          — импорт другого файла RS--
//   extern fn rand_range(a, b);  — объявление внешней Rust функции

#[derive(Debug, Clone)]
pub enum ExtStmt {
    Normal(Stmt),
    UseCrate {
        name: String,
        version: Option<String>,
        features: Vec<String>,
    },
    RustBlock(String),
    Import(String),
    ExternFn {
        name: String,
        params: Vec<String>,
        rust_body: String,
    },
}

pub struct ExtParser {
    source: String,
    pos: usize,
}

impl ExtParser {
    pub fn new(source: &str) -> Self {
        ExtParser {
            source: source.to_string(),
            pos: 0,
        }
    }

    fn remaining(&self) -> &str {
        &self.source[self.pos..]
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.source.len() {
            let ch = self.source.as_bytes()[self.pos];
            if ch == b' ' || ch == b'\t' || ch == b'\n' || ch == b'\r' {
                self.pos += 1;
            } else if self.remaining().starts_with("//") {
                // Пропускаем комментарий до конца строки
                while self.pos < self.source.len() && self.source.as_bytes()[self.pos] != b'\n' {
                    self.pos += 1;
                }
            } else {
                break;
            }
        }
    }

    fn peek_word(&self) -> &str {
        let rem = self.remaining().trim_start();
        let end = rem
            .find(|c: char| !c.is_alphanumeric() && c != '_')
            .unwrap_or(rem.len());
        &rem[..end]
    }

    fn read_braced_block(&mut self) -> Result<String, ParseError> {
        self.skip_whitespace();
        if self.pos >= self.source.len() || self.source.as_bytes()[self.pos] != b'{' {
            return Err(ParseError {
                msg: "Ожидалось '{'".into(),
            });
        }
        self.pos += 1; // skip {

        let mut depth = 1;
        let start = self.pos;

        while self.pos < self.source.len() && depth > 0 {
            match self.source.as_bytes()[self.pos] {
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        let content = self.source[start..self.pos].to_string();
                        self.pos += 1; // skip }
                        return Ok(content);
                    }
                }
                b'"' => {
                    // Пропускаем строку
                    self.pos += 1;
                    while self.pos < self.source.len() {
                        if self.source.as_bytes()[self.pos] == b'\\' {
                            self.pos += 2;
                            continue;
                        }
                        if self.source.as_bytes()[self.pos] == b'"' {
                            break;
                        }
                        self.pos += 1;
                    }
                }
                _ => {}
            }
            self.pos += 1;
        }

        Err(ParseError {
            msg: "Незакрытый блок '}'".into(),
        })
    }

    fn read_string_lit(&mut self) -> Result<String, ParseError> {
        if self.pos >= self.source.len() || self.source.as_bytes()[self.pos] != b'"' {
            return Err(ParseError {
                msg: "Ожидалась строка".into(),
            });
        }
        self.pos += 1; // skip "
        let start = self.pos;
        while self.pos < self.source.len() {
            if self.source.as_bytes()[self.pos] == b'\\' {
                self.pos += 2;
                continue;
            }
            if self.source.as_bytes()[self.pos] == b'"' {
                let s = self.source[start..self.pos].to_string();
                self.pos += 1; // skip "
                return Ok(s);
            }
            self.pos += 1;
        }
        Err(ParseError {
            msg: "Незакрытая строка".into(),
        })
    }

    fn read_ident(&mut self) -> Result<String, ParseError> {
        self.skip_whitespace();
        let start = self.pos;
        while self.pos < self.source.len() {
            let ch = self.source.as_bytes()[self.pos];
            if ch.is_ascii_alphanumeric() || ch == b'_' {
                self.pos += 1;
            } else {
                break;
            }
        }
        if self.pos == start {
            return Err(ParseError {
                msg: "Ожидался идентификатор".into(),
            });
        }
        Ok(self.source[start..self.pos].to_string())
    }

    fn expect_char(&mut self, ch: char) -> Result<(), ParseError> {
        self.skip_whitespace();
        if self.pos < self.source.len() && self.source.as_bytes()[self.pos] == ch as u8 {
            self.pos += 1;
            Ok(())
        } else {
            let got = if self.pos < self.source.len() {
                self.source[self.pos..].chars().next().unwrap_or('?')
            } else {
                '?'
            };
            Err(ParseError {
                msg: format!("Ожидалось '{}', получено '{}'", ch, got),
            })
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.remaining().trim_start().chars().next()
    }

    pub fn parse_extended(&mut self) -> Result<Vec<ExtStmt>, ParseError> {
        let mut stmts = Vec::new();

        loop {
            self.skip_whitespace();
            if self.pos >= self.source.len() {
                break;
            }

            let word = self.peek_word().to_string();

            match word.as_str() {
                "use_crate" => {
                    stmts.push(self.parse_use_crate()?);
                }
                "rust" => {
                    stmts.push(self.parse_rust_block()?);
                }
                "import" => {
                    stmts.push(self.parse_import()?);
                }
                "extern_fn" => {
                    stmts.push(self.parse_extern_fn()?);
                }
                _ => {
                    // Обычный RS-- код: читаем до конца стейтмента
                    // и парсим через обычный парсер
                    let stmt = self.parse_normal_stmt()?;
                    stmts.push(ExtStmt::Normal(stmt));
                }
            }
        }

        Ok(stmts)
    }

    fn parse_use_crate(&mut self) -> Result<ExtStmt, ParseError> {
        // use_crate("name", "version", "feat1", "feat2");
        self.pos += "use_crate".len();
        self.skip_whitespace();
        self.expect_char('(')?;
        self.skip_whitespace();

        let name = self.read_string_lit()?;

        let mut version = None;
        let mut features = Vec::new();

        self.skip_whitespace();
        if self.peek_char() == Some(',') {
            self.pos += 1; // ,
            self.skip_whitespace();

            if self.peek_char() == Some('"') {
                version = Some(self.read_string_lit()?);
            }

            // Дополнительные аргументы = фичи
            self.skip_whitespace();
            while self.peek_char() == Some(',') {
                self.pos += 1;
                self.skip_whitespace();
                if self.peek_char() == Some('"') {
                    features.push(self.read_string_lit()?);
                } else if self.peek_char() != Some(')') {
                    let ident = self.read_ident()?;
                    features.push(ident);
                }
                self.skip_whitespace();
            }
        }

        self.skip_whitespace();
        self.expect_char(')')?;
        self.skip_whitespace();
        self.expect_char(';')?;

        Ok(ExtStmt::UseCrate {
            name,
            version,
            features,
        })
    }

    fn parse_rust_block(&mut self) -> Result<ExtStmt, ParseError> {
        self.pos += "rust".len();
        self.skip_whitespace();
        let code = self.read_braced_block()?;
        Ok(ExtStmt::RustBlock(code.trim().to_string()))
    }

    fn parse_import(&mut self) -> Result<ExtStmt, ParseError> {
        self.pos += "import".len();
        self.skip_whitespace();
        self.expect_char('(')?;
        self.skip_whitespace();
        let path = self.read_string_lit()?;
        self.skip_whitespace();
        self.expect_char(')')?;
        self.skip_whitespace();
        self.expect_char(';')?;
        Ok(ExtStmt::Import(path))
    }

    fn parse_extern_fn(&mut self) -> Result<ExtStmt, ParseError> {
        self.pos += "extern_fn".len();
        self.skip_whitespace();

        let name = self.read_ident()?;
        self.skip_whitespace();

        // Параметры
        self.expect_char('(')?;
        let mut params = Vec::new();
        loop {
            self.skip_whitespace();
            if self.peek_char() == Some(')') {
                break;
            }
            let param = self.read_ident()?;
            params.push(param);
            self.skip_whitespace();
            if self.peek_char() == Some(',') {
                self.pos += 1;
            }
        }
        self.expect_char(')')?;
        self.skip_whitespace();

        // Тело — сырой Rust код
        let body = self.read_braced_block()?;

        Ok(ExtStmt::ExternFn {
            name,
            params,
            rust_body: body.trim().to_string(),
        })
    }

    fn parse_normal_stmt(&mut self) -> Result<Stmt, ParseError> {
        // Читаем один стейтмент RS--
        // Находим конец: ; или закрывающую } для блоков
        let start = self.pos;

        // Определяем тип стейтмента
        let word = self.peek_word().to_string();

        let end_pos = match word.as_str() {
            "fn" | "if" | "loop" => {
                // Блочный стейтмент — ищем закрывающую }
                self.find_block_end()?
            }
            _ => {
                // Простой стейтмент — ищем ;
                self.find_semicolon()?
            }
        };

        let stmt_source = &self.source[start..end_pos];
        self.pos = end_pos;

        // Парсим через обычный парсер RS--
        let mut lexer = Lexer::new(stmt_source);
        let tokens = lexer.tokenize().map_err(|e| ParseError {
            msg: format!("{}", e),
        })?;

        let mut parser = Parser::new(tokens);
        parser.parse_stmt()
    }

    fn find_semicolon(&mut self) -> Result<usize, ParseError> {
        let mut pos = self.pos;
        let mut in_string = false;

        while pos < self.source.len() {
            let ch = self.source.as_bytes()[pos];
            if ch == b'"' && !in_string {
                in_string = true;
            } else if ch == b'"' && in_string {
                in_string = false;
            } else if ch == b'\\' && in_string {
                pos += 1; // skip escaped char
            } else if ch == b';' && !in_string {
                return Ok(pos + 1); // включаем ;
            }
            pos += 1;
        }

        Err(ParseError {
            msg: "Не найдена ';'".into(),
        })
    }

    fn find_block_end(&mut self) -> Result<usize, ParseError> {
        let mut pos = self.pos;
        let mut depth = 0;
        let mut found_brace = false;
        let mut in_string = false;

        while pos < self.source.len() {
            let ch = self.source.as_bytes()[pos];
            if ch == b'"' && !in_string {
                in_string = true;
            } else if ch == b'"' && in_string {
                in_string = false;
            } else if ch == b'\\' && in_string {
                pos += 1;
            } else if !in_string {
                if ch == b'{' {
                    depth += 1;
                    found_brace = true;
                } else if ch == b'}' {
                    depth -= 1;
                    if depth == 0 && found_brace {
                        // Проверяем есть ли else после
                        let after = self.source[pos + 1..].trim_start();
                        if after.starts_with("else") {
                            // Продолжаем до конца else блока
                            pos += 1;
                            continue;
                        }
                        return Ok(pos + 1);
                    }
                }
            }
            pos += 1;
        }

        Err(ParseError {
            msg: "Не найден конец блока '}'".into(),
        })
    }
}

// ======================== РАСШИРЕННЫЙ ТРАНСПИЛЕР ========================

pub struct ExtTranspiler {
    indent: usize,
    output: String,
    dependencies: Vec<RustDependency>,
    rust_blocks: Vec<String>,
    extern_fns: Vec<(String, Vec<String>, String)>, // (name, params, body)
}

impl ExtTranspiler {
    pub fn new() -> Self {
        ExtTranspiler {
            indent: 0,
            output: String::new(),
            dependencies: Vec::new(),
            rust_blocks: Vec::new(),
            extern_fns: Vec::new(),
        }
    }

    fn indent_str(&self) -> String {
        "    ".repeat(self.indent)
    }

    fn write(&mut self, s: &str) {
        self.output.push_str(&self.indent_str());
        self.output.push_str(s);
        self.output.push('\n');
    }

    fn write_raw(&mut self, s: &str) {
        self.output.push_str(s);
        self.output.push('\n');
    }

    /// Транспилирует проект с несколькими файлами
	pub fn transpile_project(&mut self, project: &Project) -> Result<TranspileResult, String> {
		let mut all_stmts: Vec<ExtStmt> = Vec::new();
		let mut imported_files: std::collections::HashSet<String> = std::collections::HashSet::new();

		// Добавляем предустановленные зависимости проекта
		self.dependencies.extend(project.dependencies.clone());
		self.rust_blocks.extend(project.rust_blocks.clone());

		// Находим main файл
		let main_file = project
			.files
			.iter()
			.find(|f| f.is_main)
			.ok_or("Нет главного файла (main)")?;

		// Парсим ТОЛЬКО main файл
		let parsed = self.parse_extended_file(&main_file.content, &main_file.name)?;

		// Обрабатываем стейтменты main файла
		for stmt in parsed {
			match &stmt {
				ExtStmt::Import(path) => {
					// Импортируем файл только если ещё не импортировали
					if imported_files.insert(path.clone()) {
						if let Some(imported) = project.files.iter().find(|f| f.name == *path) {
							let imported_stmts =
								self.parse_extended_file(&imported.content, &imported.name)?;
							
							// Из импортированного файла берём только функции и объявления
							for s in imported_stmts {
								match &s {
									ExtStmt::Normal(Stmt::FnDef { .. }) => all_stmts.push(s),
									ExtStmt::ExternFn { .. } => all_stmts.push(s),
									ExtStmt::UseCrate { .. } => all_stmts.push(s),
									ExtStmt::RustBlock(_) => all_stmts.push(s),
									ExtStmt::Import(nested_path) => {
										// Рекурсивный импорт
										if imported_files.insert(nested_path.clone()) {
											if let Some(nested) = project.files.iter().find(|f| f.name == *nested_path) {
												let nested_stmts = self.parse_extended_file(&nested.content, &nested.name)?;
												for ns in nested_stmts {
													match &ns {
														ExtStmt::Normal(Stmt::FnDef { .. }) => all_stmts.push(ns),
														ExtStmt::ExternFn { .. } => all_stmts.push(ns),
														ExtStmt::UseCrate { .. } => all_stmts.push(ns),
														ExtStmt::RustBlock(_) => all_stmts.push(ns),
														_ => {}
													}
												}
											}
										}
									}
									_ => {} // Пропускаем обычные стейтменты из библиотек
								}
							}
						} else {
							return Err(format!(
								"Файл '{}' не найден в проекте",
								path
							));
						}
					}
				}
				_ => all_stmts.push(stmt),
			}
		}

		// Генерируем Rust код
		let rust_code = self.generate_rust(&all_stmts)?;
		let cargo_toml = self.generate_cargo_toml(&project.name);

		Ok(TranspileResult {
			rust_code,
			cargo_toml,
			dependencies: self.dependencies.clone(),
		})
	}

    /// Транспилирует один файл
    pub fn transpile_single(&mut self, source: &str) -> Result<TranspileResult, String> {
        let stmts = self.parse_extended_file(source, "main.rsmm")?;
        let rust_code = self.generate_rust(&stmts)?;
        let cargo_toml = self.generate_cargo_toml("rsmm_output");

        Ok(TranspileResult {
            rust_code,
            cargo_toml,
            dependencies: self.dependencies.clone(),
        })
    }

	fn parse_extended_file(
		&mut self,
		source: &str,
		filename: &str,
	) -> Result<Vec<ExtStmt>, String> {
		let mut parser = ExtParser::new(source);
		let stmts = parser
			.parse_extended()
			.map_err(|e| format!("[{}] {}", filename, e))?;

		// Извлекаем зависимости и rust блоки
		for stmt in &stmts {
			match stmt {
				ExtStmt::UseCrate {
					name,
					version,
					features,
				} => {
					let ver = version.clone().unwrap_or_else(|| "*".to_string());
					if features.is_empty() {
						self.dependencies.push(RustDependency::new(name, &ver));
					} else {
						let feats: Vec<&str> = features.iter().map(|s| s.as_str()).collect();
						self.dependencies
							.push(RustDependency::with_features(name, &ver, feats));
					}
				}
				ExtStmt::RustBlock(code) => {
					self.rust_blocks.push(code.clone());
				}
				ExtStmt::ExternFn {
					name,
					params,
					rust_body,
				} => {
					self.extern_fns
						.push((name.clone(), params.clone(), rust_body.clone()));
				}
				_ => {}
			}
		}

		Ok(stmts)
	}
	fn generate_runtime(&mut self) {
		self.write_raw(r#"
	#[derive(Clone, Debug)]
	enum RsmmVal {
		Int(i64),
		Float(f64),
		Str(String),
		Bool(bool),
		None,
	}

	impl std::fmt::Display for RsmmVal {
		fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
			match self {
				RsmmVal::Int(n) => write!(f, "{}", n),
				RsmmVal::Float(n) => write!(f, "{}", n),
				RsmmVal::Str(s) => write!(f, "{}", s),
				RsmmVal::Bool(b) => write!(f, "{}", b),
				RsmmVal::None => write!(f, "none"),
			}
		}
	}

	impl RsmmVal {
		fn as_int(&self) -> i64 {
			match self {
				RsmmVal::Int(n) => *n,
				RsmmVal::Float(n) => *n as i64,
				RsmmVal::Bool(b) => if *b { 1 } else { 0 },
				RsmmVal::Str(s) => s.parse().unwrap_or(0),
				RsmmVal::None => 0,
			}
		}
		fn as_float(&self) -> f64 {
			match self {
				RsmmVal::Int(n) => *n as f64,
				RsmmVal::Float(n) => *n,
				RsmmVal::Bool(b) => if *b { 1.0 } else { 0.0 },
				RsmmVal::Str(s) => s.parse().unwrap_or(0.0),
				RsmmVal::None => 0.0,
			}
		}
		fn as_str(&self) -> String {
			format!("{}", self)
		}
		fn as_bool(&self) -> bool {
			match self {
				RsmmVal::Int(n) => *n != 0,
				RsmmVal::Float(n) => *n != 0.0,
				RsmmVal::Str(s) => !s.is_empty(),
				RsmmVal::Bool(b) => *b,
				RsmmVal::None => false,
			}
		}
	}

	fn rsmm_add_impl(a: &RsmmVal, b: &RsmmVal) -> RsmmVal {
		match (a, b) {
			(RsmmVal::Int(x), RsmmVal::Int(y)) => RsmmVal::Int(x + y),
			(RsmmVal::Float(x), RsmmVal::Float(y)) => RsmmVal::Float(x + y),
			(RsmmVal::Int(x), RsmmVal::Float(y)) => RsmmVal::Float(*x as f64 + y),
			(RsmmVal::Float(x), RsmmVal::Int(y)) => RsmmVal::Float(x + *y as f64),
			(RsmmVal::Str(x), other) => RsmmVal::Str(format!("{}{}", x, other)),
			(other, RsmmVal::Str(y)) => RsmmVal::Str(format!("{}{}", other, y)),
			_ => RsmmVal::Str(format!("{}{}", a, b)),
		}
	}

	fn rsmm_sub(a: &RsmmVal, b: &RsmmVal) -> RsmmVal {
		match (a, b) {
			(RsmmVal::Float(x), RsmmVal::Float(y)) => RsmmVal::Float(x - y),
			(RsmmVal::Int(x), RsmmVal::Float(y)) => RsmmVal::Float(*x as f64 - y),
			(RsmmVal::Float(x), RsmmVal::Int(y)) => RsmmVal::Float(x - *y as f64),
			_ => RsmmVal::Int(a.as_int() - b.as_int()),
		}
	}

	fn rsmm_mul(a: &RsmmVal, b: &RsmmVal) -> RsmmVal {
		match (a, b) {
			(RsmmVal::Float(x), RsmmVal::Float(y)) => RsmmVal::Float(x * y),
			(RsmmVal::Int(x), RsmmVal::Float(y)) => RsmmVal::Float(*x as f64 * y),
			(RsmmVal::Float(x), RsmmVal::Int(y)) => RsmmVal::Float(x * *y as f64),
			_ => RsmmVal::Int(a.as_int() * b.as_int()),
		}
	}

	fn rsmm_div(a: &RsmmVal, b: &RsmmVal) -> RsmmVal {
		match (a, b) {
			(RsmmVal::Float(x), RsmmVal::Float(y)) => RsmmVal::Float(x / y),
			(RsmmVal::Int(x), RsmmVal::Float(y)) => RsmmVal::Float(*x as f64 / y),
			(RsmmVal::Float(x), RsmmVal::Int(y)) => RsmmVal::Float(x / *y as f64),
			_ => RsmmVal::Int(a.as_int() / b.as_int()),
		}
	}

	fn rsmm_mod(a: &RsmmVal, b: &RsmmVal) -> RsmmVal {
		RsmmVal::Int(a.as_int() % b.as_int())
	}

	fn rsmm_eq(a: &RsmmVal, b: &RsmmVal) -> RsmmVal {
		match (a, b) {
			(RsmmVal::Int(x), RsmmVal::Int(y)) => RsmmVal::Bool(x == y),
			(RsmmVal::Float(x), RsmmVal::Float(y)) => RsmmVal::Bool(x == y),
			(RsmmVal::Str(x), RsmmVal::Str(y)) => RsmmVal::Bool(x == y),
			(RsmmVal::Bool(x), RsmmVal::Bool(y)) => RsmmVal::Bool(x == y),
			_ => RsmmVal::Bool(false),
		}
	}

	fn rsmm_neq(a: &RsmmVal, b: &RsmmVal) -> RsmmVal {
		RsmmVal::Bool(!rsmm_eq(a, b).as_bool())
	}

	fn rsmm_lt(a: &RsmmVal, b: &RsmmVal) -> RsmmVal {
		RsmmVal::Bool(a.as_int() < b.as_int())
	}

	fn rsmm_gt(a: &RsmmVal, b: &RsmmVal) -> RsmmVal {
		RsmmVal::Bool(a.as_int() > b.as_int())
	}

	fn rsmm_lte(a: &RsmmVal, b: &RsmmVal) -> RsmmVal {
		RsmmVal::Bool(a.as_int() <= b.as_int())
	}

	fn rsmm_gte(a: &RsmmVal, b: &RsmmVal) -> RsmmVal {
		RsmmVal::Bool(a.as_int() >= b.as_int())
	}
	"#);
	}
	

	fn generate_rust(&mut self, stmts: &[ExtStmt]) -> Result<String, String> {
		self.output.clear();

		// Хедер
		self.write_raw("#![allow(unused_variables, unused_mut, unused_assignments, dead_code)]");
		self.write_raw("");

		// Use statements из rust блоков
		for block in &self.rust_blocks.clone() {
			self.write_raw(block);
		}
		if !self.rust_blocks.is_empty() {
			self.write_raw("");
		}

		// Runtime
		self.generate_runtime();

		// Extern функции — без дубликатов
		let mut seen_extern: std::collections::HashSet<String> = std::collections::HashSet::new();
		for (name, params, body) in &self.extern_fns.clone() {
			if !seen_extern.insert(name.clone()) {
				continue; // Пропускаем дубликат
			}
			let params_str = params
				.iter()
				.map(|p| format!("{}: RsmmVal", p))
				.collect::<Vec<_>>()
				.join(", ");
			self.write(&format!("fn {}({}) -> RsmmVal {{", name, params_str));
			self.indent += 1;
			self.write_raw(body);
			self.indent -= 1;
			self.write("}");
			self.write_raw("");
		}

		// Собираем функции и main стейтменты — без дубликатов
		let mut functions = Vec::new();
		let mut main_stmts = Vec::new();
		let mut seen_fns: std::collections::HashSet<String> = std::collections::HashSet::new();

		for stmt in stmts {
			match stmt {
				ExtStmt::Normal(Stmt::FnDef { name, .. }) => {
					if seen_fns.insert(name.clone()) {
						functions.push(stmt.clone());
					}
					// Пропускаем дубликат
				}
				ExtStmt::Normal(_) => main_stmts.push(stmt.clone()),
				ExtStmt::UseCrate { .. }
				| ExtStmt::RustBlock(_)
				| ExtStmt::Import(_)
				| ExtStmt::ExternFn { .. } => {
					// Уже обработаны
				}
			}
		}

		// Функции пользователя
		let inner = Transpiler::new();
		for func in &functions {
			if let ExtStmt::Normal(stmt) = func {
				self.transpile_stmt_inner(stmt, &inner)?;
				self.write_raw("");
			}
		}

		// Main
		self.write("fn main() {");
		self.indent += 1;
		for stmt in &main_stmts {
			if let ExtStmt::Normal(s) = stmt {
				self.transpile_stmt_inner(s, &inner)?;
			}
		}
		self.indent -= 1;
		self.write("}");

		Ok(self.output.clone())
	}

    fn transpile_stmt_inner(&mut self, stmt: &Stmt, t: &Transpiler) -> Result<(), String> {
        match stmt {
            Stmt::Let { name, value } => {
                let expr = t.transpile_expr(value)?;
                self.write(&format!("let mut {} = {};", name, expr));
            }
            Stmt::Assign { name, value } => {
                let expr = t.transpile_expr(value)?;
                self.write(&format!("{} = {};", name, expr));
            }
            Stmt::Print(expr) => {
                let e = t.transpile_expr(expr)?;
                self.write(&format!("println!(\"{{}}\", {});", e));
            }
            Stmt::If {
                condition,
                then_block,
                else_block,
            } => {
                let cond = t.transpile_expr(condition)?;
                self.write(&format!("if ({}).as_bool() {{", cond));
                self.indent += 1;
                for s in then_block {
                    self.transpile_stmt_inner(s, t)?;
                }
                self.indent -= 1;
                if let Some(else_stmts) = else_block {
                    self.write("} else {");
                    self.indent += 1;
                    for s in else_stmts {
                        self.transpile_stmt_inner(s, t)?;
                    }
                    self.indent -= 1;
                }
                self.write("}");
            }
            Stmt::Loop { body } => {
                self.write("loop {");
                self.indent += 1;
                for s in body {
                    self.transpile_stmt_inner(s, t)?;
                }
                self.indent -= 1;
                self.write("}");
            }
            Stmt::Break => self.write("break;"),
            Stmt::FnDef { name, params, body } => {
                let params_str = params
                    .iter()
                    .map(|p| format!("{}: RsmmVal", p))
                    .collect::<Vec<_>>()
                    .join(", ");
                self.write(&format!("fn {}({}) {{", name, params_str));
                self.indent += 1;
                for p in params {
                    self.write(&format!("let mut {} = {};", p, p));
                }
                for s in body {
                    self.transpile_stmt_inner(s, t)?;
                }
                self.indent -= 1;
                self.write("}");
            }
            Stmt::Return(expr) => {
                if let Some(e) = expr {
                    let val = t.transpile_expr(e)?;
                    self.write(&format!("return {};", val));
                } else {
                    self.write("return;");
                }
            }
            Stmt::ExprStmt(expr) => {
                let e = t.transpile_expr(expr)?;
                self.write(&format!("{};", e));
            }
        }
        Ok(())
    }
	fn generate_cargo_toml(&self, name: &str) -> String {
		let mut toml = format!(
	r#"[package]
	name = "{}"
	version = "0.1.0"
	edition = "2021"

	[dependencies]
	"#,
			name
		);

		let mut seen = std::collections::HashSet::new();
		for dep in &self.dependencies {
			if seen.insert(dep.name.clone()) {
				toml.push_str(&dep.to_toml_line());
				toml.push('\n');
			}
		}

		toml.push_str(
	r#"
	[profile.release]
	opt-level = "z"
	lto = true
	strip = true
	"#,
		);

		toml
	}
}
#[derive(Debug, Clone)]
pub struct TranspileResult {
    pub rust_code: String,
    pub cargo_toml: String,
    pub dependencies: Vec<RustDependency>,
}

// ======================== СБОРКА ПРОЕКТА ========================

pub fn build_project_exe(project: &Project, output_path: &std::path::Path) -> Result<String, String> {
    let mut transpiler = ExtTranspiler::new();
    let result = transpiler.transpile_project(project)?;

    build_from_transpiled(&result, &project.name, output_path)
}

pub fn build_single_exe(source: &str, output_path: &std::path::Path) -> Result<String, String> {
    let mut transpiler = ExtTranspiler::new();
    let result = transpiler.transpile_single(source)?;

    build_from_transpiled(&result, "rsmm_output", output_path)
}

fn build_from_transpiled(
    result: &TranspileResult,
    name: &str,
    output_path: &std::path::Path,
) -> Result<String, String> {
    let temp_dir = std::env::temp_dir().join(format!("rsmm_build_{}", name));
    let src_dir = temp_dir.join("src");

    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&src_dir).map_err(|e| format!("Не могу создать папку: {}", e))?;

    std::fs::write(temp_dir.join("Cargo.toml"), &result.cargo_toml)
        .map_err(|e| format!("Не могу записать Cargo.toml: {}", e))?;

    std::fs::write(src_dir.join("main.rs"), &result.rust_code)
        .map_err(|e| format!("Не могу записать main.rs: {}", e))?;

    let output = std::process::Command::new("cargo")
        .arg("build")
        .arg("--release")
        .arg("--manifest-path")
        .arg(temp_dir.join("Cargo.toml"))
        .output()
        .map_err(|e| format!("Не могу запустить cargo: {}\nУстановлен ли Rust?", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // НЕ удаляем при ошибке — для отладки
        return Err(format!(
            "Ошибка компиляции!\n\nCargo.toml:\n{}\n\nmain.rs сохранён: {}\n\nОшибка:\n{}",
            result.cargo_toml,
            src_dir.join("main.rs").display(),
            stderr
        ));
    }

    #[cfg(windows)]
    let exe_name = format!("{}.exe", name);
    #[cfg(not(windows))]
    let exe_name = name.to_string();

    let built_exe = temp_dir.join("target").join("release").join(&exe_name);
    std::fs::copy(&built_exe, output_path)
        .map_err(|e| format!("Не могу скопировать exe: {}", e))?;

    let size = std::fs::metadata(output_path)
        .map(|m| m.len())
        .unwrap_or(0);

    let deps_info = if result.dependencies.is_empty() {
        "нет".to_string()
    } else {
        result
            .dependencies
            .iter()
            .map(|d| format!("  {} v{}", d.name, d.version))
            .collect::<Vec<_>>()
            .join("\n")
    };

    // ← ДОБАВЛЕНО: очищаем temp после успешной сборки
    let _ = std::fs::remove_dir_all(&temp_dir);

    Ok(format!(
        "✅ EXE собран!\n\
         Путь: {}\n\
         Размер: {} KB\n\
         Зависимости:\n{}",
        output_path.display(),
        size / 1024,
        deps_info
    ))
}

// Делаем transpile_expr публичным для ExtTranspiler
impl Transpiler {
    pub fn transpile_expr_pub(&self, expr: &Expr) -> Result<String, String> {
        self.transpile_expr(expr)
    }
}

// Обновлённая функция для main.rs — замена старой build_exe
pub fn build_exe_v2(source: &str, output_path: &std::path::Path) -> Result<String, String> {
    build_single_exe(source, output_path)
}
// ======================== СБОРКА MSI УСТАНОВЩИКА ========================

pub fn build_msi(
    source: &str,
    project_name: &str,
    output_path: &std::path::Path,
) -> Result<String, String> {
    let temp_dir = std::env::temp_dir().join("rsmm_msi_build");
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&temp_dir)
        .map_err(|e| format!("Не могу создать папку: {}", e))?;

    let exe_path = temp_dir.join(format!("{}.exe", project_name));

    build_single_exe(source, &exe_path)?;

    let wxs_content = generate_wxs(project_name, &exe_path)?;
    let wxs_path = temp_dir.join("installer.wxs");
    std::fs::write(&wxs_path, &wxs_content)
        .map_err(|e| format!("Не могу записать WXS: {}", e))?;

    let msi_result = build_with_wix(&wxs_path, &temp_dir, output_path, project_name);

    let result = match msi_result {
        Ok(info) => Ok(info),
        Err(_) => {
            build_self_extracting_installer(
                &exe_path,
                project_name,
                output_path,
            )
        }
    };

    // ← ДОБАВЛЕНО: очищаем temp
    let _ = std::fs::remove_dir_all(&temp_dir);

    result
}
pub fn build_project_msi(
    project: &Project,
    output_path: &std::path::Path,
) -> Result<String, String> {
    let temp_dir = std::env::temp_dir().join("rsmm_msi_build");
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&temp_dir)
        .map_err(|e| format!("Не могу создать папку: {}", e))?;

    let exe_path = temp_dir.join(format!("{}.exe", project.name));

    build_project_exe(project, &exe_path)?;

    let wxs_content = generate_wxs(&project.name, &exe_path)?;
    let wxs_path = temp_dir.join("installer.wxs");
    let _ = std::fs::write(&wxs_path, &wxs_content);

    let msi_result = build_with_wix(&wxs_path, &temp_dir, output_path, &project.name);

    let result = match msi_result {
        Ok(info) => Ok(info),
        Err(_) => {
            build_self_extracting_installer(
                &exe_path,
                &project.name,
                output_path,
            )
        }
    };

    // ← ДОБАВЛЕНО: очищаем temp
    let _ = std::fs::remove_dir_all(&temp_dir);

    result
}

fn generate_wxs(name: &str, exe_path: &std::path::Path) -> Result<String, String> {
    let exe_full = exe_path
        .canonicalize()
        .unwrap_or_else(|_| exe_path.to_path_buf())
        .to_string_lossy()
        .to_string();

    // Генерируем уникальные UUID
    let product_id = simple_uuid(name, 1);
    let upgrade_id = simple_uuid(name, 2);
    let component_id = simple_uuid(name, 3);

    Ok(format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<Wix xmlns="http://schemas.microsoft.com/wix/2006/wi">
    <Product Id="{product_id}"
             Name="{name}"
             Language="1033"
             Version="1.0.0"
             Manufacturer="Rust--"
             UpgradeCode="{upgrade_id}">

        <Package InstallerVersion="200"
                 Compressed="yes"
                 InstallScope="perUser"
                 Description="{name} - Created with Rust--"
                 Comments="Built with Rust-- IDE"/>

        <MediaTemplate EmbedCab="yes"/>

        <Directory Id="TARGETDIR" Name="SourceDir">
            <Directory Id="LocalAppDataFolder">
                <Directory Id="INSTALLFOLDER" Name="{name}">
                    <Component Id="MainComponent" Guid="{component_id}">
                        <File Id="MainExe"
                              Source="{exe_full}"
                              KeyPath="yes"/>
                    </Component>
                </Directory>
            </Directory>
            <Directory Id="ProgramMenuFolder">
                <Directory Id="ApplicationProgramsFolder" Name="{name}"/>
            </Directory>
        </Directory>

        <DirectoryRef Id="ApplicationProgramsFolder">
            <Component Id="ApplicationShortcut" Guid="{product_id}">
                <Shortcut Id="ApplicationStartMenuShortcut"
                          Name="{name}"
                          Description="{name}"
                          Target="[INSTALLFOLDER]{name}.exe"
                          WorkingDirectory="INSTALLFOLDER"/>
                <RemoveFolder Id="CleanUpShortCut"
                              Directory="ApplicationProgramsFolder"
                              On="uninstall"/>
                <RegistryValue Root="HKCU"
                               Key="Software\Rust--\{name}"
                               Name="installed"
                               Type="integer"
                               Value="1"
                               KeyPath="yes"/>
            </Component>
        </DirectoryRef>

        <Feature Id="MainFeature" Title="{name}" Level="1">
            <ComponentRef Id="MainComponent"/>
            <ComponentRef Id="ApplicationShortcut"/>
        </Feature>

        <UIRef Id="WixUI_Minimal"/>
    </Product>
</Wix>"#
    ))
}

fn simple_uuid(seed: &str, variant: u32) -> String {
    // Генерируем детерминированный UUID из имени
    let mut hash: u64 = 0xcbf29ce484222325;
    for b in seed.bytes() {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash ^= variant as u64;

    format!(
        "{:08x}-{:04x}-4{:03x}-{:04x}-{:012x}",
        (hash & 0xFFFFFFFF) as u32,
        ((hash >> 32) & 0xFFFF) as u16,
        ((hash >> 48) & 0x0FFF) as u16,
        (((hash >> 16) & 0x3FFF) | 0x8000) as u16,
        hash & 0xFFFFFFFFFFFF
    )
}

fn build_with_wix(
    wxs_path: &std::path::Path,
    temp_dir: &std::path::Path,
    output_path: &std::path::Path,
    name: &str,
) -> Result<String, String> {
    // Пробуем найти WiX Toolset
    let candle = find_wix_tool("candle");
    let light = find_wix_tool("light");

    let candle_cmd = candle.ok_or("WiX Toolset не найден")?;
    let light_cmd = light.ok_or("WiX Toolset не найден")?;

    // candle (компиляция WXS -> WIXOBJ)
    let wixobj_path = temp_dir.join("installer.wixobj");
    let output = std::process::Command::new(&candle_cmd)
        .arg(wxs_path)
        .arg("-o")
        .arg(&wixobj_path)
        .output()
        .map_err(|e| format!("candle error: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "candle failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    // light (линковка WIXOBJ -> MSI)
    let output = std::process::Command::new(&light_cmd)
        .arg(&wixobj_path)
        .arg("-o")
        .arg(output_path)
        .arg("-ext")
        .arg("WixUIExtension")
        .output()
        .map_err(|e| format!("light error: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "light failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let size = std::fs::metadata(output_path)
        .map(|m| m.len())
        .unwrap_or(0);

    Ok(format!(
        "✅ MSI установщик создан (WiX)!\n\
         Путь: {}\n\
         Размер: {} KB\n\
         Программа: {}",
        output_path.display(),
        size / 1024,
        name
    ))
}

fn find_wix_tool(tool: &str) -> Option<String> {
    // Проверяем в PATH
    if std::process::Command::new(tool)
        .arg("/?")
        .output()
        .is_ok()
    {
        return Some(tool.to_string());
    }

    // Проверяем стандартные пути WiX
    let wix_paths = vec![
        format!("C:\\Program Files (x86)\\WiX Toolset v3.11\\bin\\{}.exe", tool),
        format!("C:\\Program Files (x86)\\WiX Toolset v3.14\\bin\\{}.exe", tool),
        format!("C:\\Program Files\\WiX Toolset v3.11\\bin\\{}.exe", tool),
        format!("C:\\Program Files\\WiX Toolset v4.0\\bin\\{}.exe", tool),
    ];

    // Также проверяем через переменную WIX
    if let Ok(wix_dir) = std::env::var("WIX") {
        let path = format!("{}bin\\{}.exe", wix_dir, tool);
        if std::path::Path::new(&path).exists() {
            return Some(path);
        }
    }

    for path in wix_paths {
        if std::path::Path::new(&path).exists() {
            return Some(path);
        }
    }

    None
}

// Fallback: создаём самораспаковывающийся установщик
fn build_self_extracting_installer(
    exe_path: &std::path::Path,
    name: &str,
    output_path: &std::path::Path,
) -> Result<String, String> {
    let _exe_bytes = std::fs::read(exe_path)
        .map_err(|e| format!("Не могу прочитать exe: {}", e))?;

    let installer_source = INSTALLER_TEMPLATE.replace("%%APP_NAME%%", name);

    let build_dir = std::env::temp_dir().join("rsmm_installer_build");
    let src_dir = build_dir.join("src");
    let _ = std::fs::remove_dir_all(&build_dir);
    std::fs::create_dir_all(&src_dir)
        .map_err(|e| format!("Не могу создать папку: {}", e))?;

    std::fs::copy(exe_path, src_dir.join("app.exe"))
        .map_err(|e| format!("Не могу скопировать exe: {}", e))?;

    let cargo_toml = format!(
        r#"[package]
name = "{}_installer"
version = "1.0.0"
edition = "2021"

[dependencies]
eframe = "0.31"
rfd = "0.15"

[profile.release]
opt-level = "z"
lto = true
strip = true
"#,
        name
    );

    std::fs::write(build_dir.join("Cargo.toml"), &cargo_toml)
        .map_err(|e| format!("Не могу записать Cargo.toml: {}", e))?;

    std::fs::write(src_dir.join("main.rs"), &installer_source)
        .map_err(|e| format!("Не могу записать main.rs: {}", e))?;

    let output = std::process::Command::new("cargo")
        .arg("build")
        .arg("--release")
        .arg("--manifest-path")
        .arg(build_dir.join("Cargo.toml"))
        .output()
        .map_err(|e| format!("cargo error: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "Ошибка сборки установщика:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    #[cfg(windows)]
    let installer_exe = format!("{}_installer.exe", name);
    #[cfg(not(windows))]
    let installer_exe = format!("{}_installer", name);

    let built = build_dir
        .join("target")
        .join("release")
        .join(&installer_exe);

    let final_path = if output_path.extension().map(|e| e == "msi").unwrap_or(false) {
        output_path.with_extension("exe")
    } else {
        output_path.to_path_buf()
    };

    std::fs::copy(&built, &final_path)
        .map_err(|e| format!("Не могу скопировать: {}", e))?;

    let size = std::fs::metadata(&final_path)
        .map(|m| m.len())
        .unwrap_or(0);

    // ← ДОБАВЛЕНО: очищаем temp
    let _ = std::fs::remove_dir_all(&build_dir);

    Ok(format!(
        "✅ GUI Установщик создан!\n\
         \n\
         Путь: {}\n\
         Размер: {} KB\n\
         Программа: {}\n\
         \n\
         Установщик включает:\n\
         • Графический интерфейс (GUI)\n\
         • Выбор папки установки\n\
         • Прогресс-бар\n\
         • Создание ярлыка на рабочем столе\n\
         • Запуск после установки",
        final_path.display(),
        size / 1024,
        name
    ))
}
const INSTALLER_TEMPLATE: &str = r##"#![windows_subsystem = "windows"]

use std::io::Write;
use std::path::PathBuf;

const APP_NAME: &str = "%%APP_NAME%%";
const APP_EXE: &[u8] = include_bytes!("app.exe");

fn main() {
    let app = InstallerApp::new();
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([500.0, 400.0])
            .with_resizable(false)
            .with_title(format!("Установка {}", APP_NAME)),
        ..Default::default()
    };
    let _ = eframe::run_native(
        &format!("Установка {}", APP_NAME),
        options,
        Box::new(|_cc| Ok(Box::new(app))),
    );
}

#[derive(PartialEq)]
enum InstallerStep {
    Welcome,
    SelectPath,
    Installing,
    Done,
    Error,
}

struct InstallerApp {
    step: InstallerStep,
    install_path: String,
    progress: f32,
    status_text: String,
    error_text: String,
    create_shortcut: bool,
    launch_after: bool,
    installed_exe_path: Option<PathBuf>,
}

impl InstallerApp {
    fn new() -> Self {
        let default_path = if cfg!(windows) {
            let appdata = std::env::var("LOCALAPPDATA")
                .unwrap_or_else(|_| "C:\\Program Files".to_string());
            format!("{}\\{}", appdata, APP_NAME)
        } else {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/usr/local".to_string());
            format!("{}/.local/bin", home)
        };

        InstallerApp {
            step: InstallerStep::Welcome,
            install_path: default_path,
            progress: 0.0,
            status_text: String::new(),
            error_text: String::new(),
            create_shortcut: true,
            launch_after: true,
            installed_exe_path: None,
        }
    }

    fn do_install(&mut self) {
        self.step = InstallerStep::Installing;
        self.progress = 0.0;
        self.status_text = "Создание папки...".into();

        let install_dir = PathBuf::from(&self.install_path);

        if let Err(e) = std::fs::create_dir_all(&install_dir) {
            self.error_text = format!("Не могу создать папку: {}", e);
            self.step = InstallerStep::Error;
            return;
        }
        self.progress = 0.3;
        self.status_text = "Копирование файлов...".into();

        let exe_name = if cfg!(windows) {
            format!("{}.exe", APP_NAME)
        } else {
            APP_NAME.to_string()
        };

        let exe_path = install_dir.join(&exe_name);
        if let Err(e) = std::fs::write(&exe_path, APP_EXE) {
            self.error_text = format!("Не могу записать файл: {}", e);
            self.step = InstallerStep::Error;
            return;
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&exe_path, std::fs::Permissions::from_mode(0o755));
        }

        self.progress = 0.7;

        if self.create_shortcut {
            self.status_text = "Создание ярлыка...".into();
            #[cfg(windows)]
            {
                let vbs_content = format!(
                    "Set oWS = WScript.CreateObject(\"WScript.Shell\")\n\
                     Set oLink = oWS.CreateShortcut(oWS.SpecialFolders(\"Desktop\") & \"\\{}.lnk\")\n\
                     oLink.TargetPath = \"{}\"\n\
                     oLink.WorkingDirectory = \"{}\"\n\
                     oLink.Description = \"{}\"\n\
                     oLink.Save",
                    APP_NAME,
                    exe_path.display(),
                    install_dir.display(),
                    APP_NAME
                );
                let vbs_path = std::env::temp_dir().join("create_shortcut.vbs");
                if std::fs::write(&vbs_path, &vbs_content).is_ok() {
                    let _ = std::process::Command::new("wscript")
                        .arg(&vbs_path)
                        .output();
                    let _ = std::fs::remove_file(&vbs_path);
                }
            }
        }

        self.progress = 1.0;
        self.status_text = "Готово!".into();
        self.installed_exe_path = Some(exe_path);
        self.step = InstallerStep::Done;
    }
}

impl eframe::App for InstallerApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        use eframe::egui;

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(15.0);
                ui.label(
                    egui::RichText::new(format!("\u{1F4E6} {}", APP_NAME))
                        .size(28.0)
                        .strong()
                        .color(egui::Color32::from_rgb(100, 180, 255)),
                );
                ui.label(
                    egui::RichText::new("Создано с помощью Rust-- IDE")
                        .size(12.0)
                        .italics()
                        .color(egui::Color32::GRAY),
                );
                ui.add_space(10.0);
            });

            ui.separator();
            ui.add_space(10.0);

            match self.step {
                InstallerStep::Welcome => {
                    ui.vertical_centered(|ui| {
                        ui.add_space(20.0);
                        ui.label(
                            egui::RichText::new("Добро пожаловать!")
                                .size(20.0)
                                .strong(),
                        );
                        ui.add_space(15.0);
                        ui.label(format!(
                            "Этот мастер установит {} на ваш компьютер.",
                            APP_NAME
                        ));
                        ui.add_space(30.0);

                        if ui
                            .button(egui::RichText::new("  Далее \u{2192}  ").size(16.0))
                            .clicked()
                        {
                            self.step = InstallerStep::SelectPath;
                        }

                        ui.add_space(10.0);
                        if ui.button("Отмена").clicked() {
                            std::process::exit(0);
                        }
                    });
                }

                InstallerStep::SelectPath => {
                    ui.add_space(10.0);
                    ui.label(
                        egui::RichText::new("Выберите папку установки:")
                            .size(16.0)
                            .strong(),
                    );
                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        ui.label("Путь:");
                        ui.text_edit_singleline(&mut self.install_path);
                        if ui.button("\u{1F4C1} Обзор...").clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .set_directory(&self.install_path)
                                .pick_folder()
                            {
                                self.install_path = path
                                    .join(APP_NAME)
                                    .to_string_lossy()
                                    .to_string();
                            }
                        }
                    });

                    ui.add_space(15.0);
                    ui.checkbox(&mut self.create_shortcut, "Создать ярлык на рабочем столе");
                    ui.checkbox(&mut self.launch_after, "Запустить после установки");

                    ui.add_space(30.0);
                    ui.horizontal(|ui| {
                        if ui.button("\u{2190} Назад").clicked() {
                            self.step = InstallerStep::Welcome;
                        }
                        ui.add_space(20.0);
                        if ui
                            .button(
                                egui::RichText::new("  Установить  ")
                                    .size(16.0)
                                    .color(egui::Color32::from_rgb(100, 255, 100)),
                            )
                            .clicked()
                        {
                            self.do_install();
                        }
                        ui.add_space(20.0);
                        if ui.button("Отмена").clicked() {
                            std::process::exit(0);
                        }
                    });
                }

                InstallerStep::Installing => {
                    ui.add_space(30.0);
                    ui.vertical_centered(|ui| {
                        ui.label(
                            egui::RichText::new("Установка...").size(18.0).strong(),
                        );
                        ui.add_space(20.0);
                        ui.add(
                            egui::ProgressBar::new(self.progress)
                                .text(&self.status_text)
                                .animate(true),
                        );
                    });
                }

                InstallerStep::Done => {
                    ui.add_space(20.0);
                    ui.vertical_centered(|ui| {
                        ui.label(
                            egui::RichText::new("\u{2705} Установка завершена!")
                                .size(22.0)
                                .strong()
                                .color(egui::Color32::from_rgb(100, 255, 100)),
                        );
                        ui.add_space(15.0);
                        if let Some(path) = &self.installed_exe_path {
                            ui.label(format!("Программа: {}", path.display()));
                        }
                        ui.add_space(25.0);
                        if ui
                            .button(egui::RichText::new("  Готово  ").size(16.0))
                            .clicked()
                        {
                            if self.launch_after {
                                if let Some(path) = &self.installed_exe_path {
                                    let _ = std::process::Command::new(path).spawn();
                                }
                            }
                            std::process::exit(0);
                        }
                    });
                }

                InstallerStep::Error => {
                    ui.add_space(20.0);
                    ui.vertical_centered(|ui| {
                        ui.label(
                            egui::RichText::new("\u{274C} Ошибка!")
                                .size(22.0)
                                .strong()
                                .color(egui::Color32::from_rgb(255, 100, 100)),
                        );
                        ui.add_space(15.0);
                        ui.label(&self.error_text);
                        ui.add_space(25.0);
                        if ui.button("Закрыть").clicked() {
                            std::process::exit(1);
                        }
                    });
                }
            }
        });
    }
}
"##;
pub fn build_exe_with_compression(
    source: &str, 
    output_path: &std::path::Path,
    profile: &str,
) -> Result<String, String> {
    let mut transpiler = ExtTranspiler::new();
    let result = transpiler.transpile_single(source)?;

    let temp_dir = std::env::temp_dir().join("rsmm_build_compressed");
    let src_dir = temp_dir.join("src");

    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&src_dir).map_err(|e| format!("Не могу создать папку: {}", e))?;

	let cargo_toml = format!(
	r#"[package]
	name = "rsmm_output"
	version = "0.1.0"
	edition = "2021"

	[dependencies]
	{}

	{}
	"#,
		result.dependencies.iter()
			.map(|d| d.to_toml_line())
			.collect::<Vec<_>>()
			.join("\n"),
		profile
	);

    std::fs::write(temp_dir.join("Cargo.toml"), &cargo_toml)
        .map_err(|e| format!("Не могу записать Cargo.toml: {}", e))?;

    std::fs::write(src_dir.join("main.rs"), &result.rust_code)
        .map_err(|e| format!("Не могу записать main.rs: {}", e))?;

    let output = std::process::Command::new("cargo")
        .arg("build")
        .arg("--release")
        .arg("--manifest-path")
        .arg(temp_dir.join("Cargo.toml"))
        .output()
        .map_err(|e| format!("Не могу запустить cargo: {}\nУстановлен ли Rust?", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "Ошибка компиляции!\n\nmain.rs сохранён: {}\n\nОшибка:\n{}",
            src_dir.join("main.rs").display(),
            stderr
        ));
    }

    #[cfg(windows)]
    let exe_name = "rsmm_output.exe";
    #[cfg(not(windows))]
    let exe_name = "rsmm_output";

    let built_exe = temp_dir.join("target").join("release").join(exe_name);
    std::fs::copy(&built_exe, output_path)
        .map_err(|e| format!("Не могу скопировать exe: {}", e))?;

    let size = std::fs::metadata(output_path)
        .map(|m| m.len())
        .unwrap_or(0);

    // ← ДОБАВЛЕНО: очищаем temp
    let _ = std::fs::remove_dir_all(&temp_dir);

    Ok(format!(
        "✅ EXE собран!\nПуть: {}\nРазмер: {} KB",
        output_path.display(),
        size / 1024
    ))
}

pub fn build_project_exe_with_compression(
    project: &Project, 
    output_path: &std::path::Path,
    profile: &str,
) -> Result<String, String> {
    let mut transpiler = ExtTranspiler::new();
    let result = transpiler.transpile_project(project)?;

    let temp_dir = std::env::temp_dir().join(format!("rsmm_build_{}", project.name));
    let src_dir = temp_dir.join("src");

    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&src_dir).map_err(|e| format!("Не могу создать папку: {}", e))?;

    let mut cargo_content = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
"#,
        project.name
    );

    let mut seen = std::collections::HashSet::new();
    for dep in &result.dependencies {
        if seen.insert(dep.name.clone()) {
            cargo_content.push_str(&dep.to_toml_line());
            cargo_content.push('\n');
        }
    }

    cargo_content.push('\n');
    cargo_content.push_str(profile);

    std::fs::write(temp_dir.join("Cargo.toml"), &cargo_content)
        .map_err(|e| format!("Не могу записать Cargo.toml: {}", e))?;

    std::fs::write(src_dir.join("main.rs"), &result.rust_code)
        .map_err(|e| format!("Не могу записать main.rs: {}", e))?;

    let output = std::process::Command::new("cargo")
        .arg("build")
        .arg("--release")
        .arg("--manifest-path")
        .arg(temp_dir.join("Cargo.toml"))
        .output()
        .map_err(|e| format!("Не могу запустить cargo: {}\nУстановлен ли Rust?", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "Ошибка компиляции!\n\nОшибка:\n{}",
            stderr
        ));
    }

    #[cfg(windows)]
    let exe_name = format!("{}.exe", project.name);
    #[cfg(not(windows))]
    let exe_name = project.name.clone();

    let built_exe = temp_dir.join("target").join("release").join(&exe_name);
    std::fs::copy(&built_exe, output_path)
        .map_err(|e| format!("Не могу скопировать exe: {}", e))?;

    let size = std::fs::metadata(output_path)
        .map(|m| m.len())
        .unwrap_or(0);

    // ← ДОБАВЛЕНО: очищаем temp
    let _ = std::fs::remove_dir_all(&temp_dir);

    Ok(format!(
        "✅ EXE собран!\nПуть: {}\nРазмер: {} KB",
        output_path.display(),
        size / 1024
    ))
}
// ======================== NASM КОМПИЛЯЦИЯ ========================

pub struct NasmCompiler {
    asm: String,
    data_section: String,
    bss_section: String,
    label_counter: usize,
    variables: HashMap<String, usize>, // имя -> индекс переменной
    var_count: usize,
    string_counter: usize,
    functions: HashMap<String, usize>, // имя -> кол-во параметров
}

impl NasmCompiler {
    pub fn new() -> Self {
        NasmCompiler {
            asm: String::new(),
            data_section: String::new(),
            bss_section: String::new(),
            label_counter: 0,
            variables: HashMap::new(),
            var_count: 0,
            string_counter: 0,
            functions: HashMap::new(),
        }
    }

	fn new_label(&mut self, prefix: &str) -> String {
		self.label_counter += 1;
		format!("L_{}_{}", prefix, self.label_counter)
	}

    fn add_string(&mut self, s: &str) -> String {
        self.string_counter += 1;
        let label = format!("__str_{}", self.string_counter);
        
        // Экранируем строку для NASM
        let mut nasm_str = String::new();
        let mut in_quote = false;
        
        for ch in s.chars() {
            match ch {
                '\n' => {
                    if in_quote {
                        nasm_str.push_str("\", ");
                        in_quote = false;
                    }
                    nasm_str.push_str("10, ");
                }
                '\r' => {
                    if in_quote {
                        nasm_str.push_str("\", ");
                        in_quote = false;
                    }
                    nasm_str.push_str("13, ");
                }
                '\t' => {
                    if in_quote {
                        nasm_str.push_str("\", ");
                        in_quote = false;
                    }
                    nasm_str.push_str("9, ");
                }
                _ => {
                    if !in_quote {
                        nasm_str.push('"');
                        in_quote = true;
                    }
                    nasm_str.push(ch);
                }
            }
        }
        if in_quote {
            nasm_str.push('"');
        }
        
        // Убираем trailing ", "
        let nasm_str = nasm_str.trim_end_matches(", ").to_string();
        
        let len_label = format!("{}_len", label);
        self.data_section.push_str(&format!("    {} db {}, 0\n", label, nasm_str));
        self.data_section.push_str(&format!("    {} equ $ - {} - 1\n", len_label, label));
        
        label
    }

    fn get_var_offset(&mut self, name: &str) -> String {
        if let Some(&idx) = self.variables.get(name) {
            format!("[rbp-{}]", (idx + 1) * 8)
        } else {
            self.var_count += 1;
            self.variables.insert(name.to_string(), self.var_count - 1);
            format!("[rbp-{}]", self.var_count * 8)
        }
    }

    fn emit(&mut self, line: &str) {
        self.asm.push_str(line);
        self.asm.push('\n');
    }

    fn emit_indent(&mut self, line: &str) {
        self.asm.push_str("    ");
        self.asm.push_str(line);
        self.asm.push('\n');
    }
	fn expr_might_be_string(&self, expr: &Expr) -> bool {
        match expr {
            Expr::StrLit(_) => true,
            Expr::BinOp { op: BinOp::Add, left, right } => {
                self.expr_might_be_string(left) || self.expr_might_be_string(right)
            }
            _ => false,
        }
    }

    fn compile_expr_as_str(&mut self, expr: &Expr) -> Result<(), String> {
        match expr {
            Expr::StrLit(s) => {
                let label = self.add_string(s);
                self.emit_indent(&format!("lea rax, [rel {}]", label));
            }
            Expr::IntLit(n) => {
                self.emit_indent(&format!("mov rcx, {}", n));
                self.emit_indent("call __int_to_str");
            }
            Expr::BinOp { op: BinOp::Add, left, right } => {
                self.compile_expr_as_str(left)?;
                self.emit_indent("push rax");
                self.compile_expr_as_str(right)?;
                self.emit_indent("mov rdx, rax");
                self.emit_indent("pop rcx");
                self.emit_indent("call __concat");
            }
            _ => {
                self.compile_expr(expr)?;
                self.emit_indent("mov rcx, rax");
                self.emit_indent("call __int_to_str");
            }
        }
        Ok(())
    }

    // Компиляция выражения — результат в RAX
    fn compile_expr(&mut self, expr: &Expr) -> Result<(), String> {
        match expr {
            Expr::IntLit(n) => {
                self.emit_indent(&format!("mov rax, {}", n));
            }
            Expr::BoolLit(b) => {
                self.emit_indent(&format!("mov rax, {}", if *b { 1 } else { 0 }));
            }
            Expr::StrLit(s) => {
                let label = self.add_string(s);
                self.emit_indent(&format!("lea rax, [rel {}]", label));
            }
            Expr::Ident(name) => {
                let offset = self.get_var_offset(name);
                self.emit_indent(&format!("mov rax, {}", offset));
            }
			Expr::BinOp { op, left, right } => {
				// Для Add проверяем строки ПЕРЕД вычислением операндов
				match op {
					BinOp::Add => {
						let has_str = self.expr_might_be_string(left)
							|| self.expr_might_be_string(right);

						if has_str {
							self.compile_expr_as_str(left)?;
							self.emit_indent("push rax");
							self.compile_expr_as_str(right)?;
							self.emit_indent("mov rdx, rax");
							self.emit_indent("pop rcx");
							self.emit_indent("call __concat");
							return Ok(());
						}

						// Числовое сложение
						self.compile_expr(right)?;
						self.emit_indent("push rax");
						self.compile_expr(left)?;
						self.emit_indent("pop rbx");
						self.emit_indent("add rax, rbx");
					}
					BinOp::Sub => {
						self.compile_expr(right)?;
						self.emit_indent("push rax");
						self.compile_expr(left)?;
						self.emit_indent("pop rbx");
						self.emit_indent("sub rax, rbx");
					}
					BinOp::Mul => {
						self.compile_expr(right)?;
						self.emit_indent("push rax");
						self.compile_expr(left)?;
						self.emit_indent("pop rbx");
						self.emit_indent("imul rax, rbx");
					}
					BinOp::Div => {
						self.compile_expr(right)?;
						self.emit_indent("push rax");
						self.compile_expr(left)?;
						self.emit_indent("pop rbx");
						self.emit_indent("cqo");
						self.emit_indent("idiv rbx");
					}
					BinOp::Mod => {
						self.compile_expr(right)?;
						self.emit_indent("push rax");
						self.compile_expr(left)?;
						self.emit_indent("pop rbx");
						self.emit_indent("cqo");
						self.emit_indent("idiv rbx");
						self.emit_indent("mov rax, rdx");
					}
					BinOp::Eq => {
						self.compile_expr(right)?;
						self.emit_indent("push rax");
						self.compile_expr(left)?;
						self.emit_indent("pop rbx");
						self.emit_indent("cmp rax, rbx");
						self.emit_indent("sete al");
						self.emit_indent("movzx rax, al");
					}
					BinOp::Neq => {
						self.compile_expr(right)?;
						self.emit_indent("push rax");
						self.compile_expr(left)?;
						self.emit_indent("pop rbx");
						self.emit_indent("cmp rax, rbx");
						self.emit_indent("setne al");
						self.emit_indent("movzx rax, al");
					}
					BinOp::Lt => {
						self.compile_expr(right)?;
						self.emit_indent("push rax");
						self.compile_expr(left)?;
						self.emit_indent("pop rbx");
						self.emit_indent("cmp rax, rbx");
						self.emit_indent("setl al");
						self.emit_indent("movzx rax, al");
					}
					BinOp::Gt => {
						self.compile_expr(right)?;
						self.emit_indent("push rax");
						self.compile_expr(left)?;
						self.emit_indent("pop rbx");
						self.emit_indent("cmp rax, rbx");
						self.emit_indent("setg al");
						self.emit_indent("movzx rax, al");
					}
					BinOp::Lte => {
						self.compile_expr(right)?;
						self.emit_indent("push rax");
						self.compile_expr(left)?;
						self.emit_indent("pop rbx");
						self.emit_indent("cmp rax, rbx");
						self.emit_indent("setle al");
						self.emit_indent("movzx rax, al");
					}
					BinOp::Gte => {
						self.compile_expr(right)?;
						self.emit_indent("push rax");
						self.compile_expr(left)?;
						self.emit_indent("pop rbx");
						self.emit_indent("cmp rax, rbx");
						self.emit_indent("setge al");
						self.emit_indent("movzx rax, al");
					}
					BinOp::And => {
						self.compile_expr(right)?;
						self.emit_indent("push rax");
						self.compile_expr(left)?;
						self.emit_indent("pop rbx");
						self.emit_indent("test rax, rax");
						self.emit_indent("setne al");
						self.emit_indent("test rbx, rbx");
						self.emit_indent("setne bl");
						self.emit_indent("and al, bl");
						self.emit_indent("movzx rax, al");
					}
					BinOp::Or => {
						self.compile_expr(right)?;
						self.emit_indent("push rax");
						self.compile_expr(left)?;
						self.emit_indent("pop rbx");
						self.emit_indent("test rax, rax");
						self.emit_indent("setne al");
						self.emit_indent("test rbx, rbx");
						self.emit_indent("setne bl");
						self.emit_indent("or al, bl");
						self.emit_indent("movzx rax, al");
					}
				}
            }
            Expr::UnaryOp { op, expr } => {
                self.compile_expr(expr)?;
                match op {
                    UnaryOp::Neg => {
                        self.emit_indent("neg rax");
                    }
                    UnaryOp::Not => {
                        self.emit_indent("test rax, rax");
                        self.emit_indent("sete al");
                        self.emit_indent("movzx rax, al");
                    }
                }
            }
            Expr::Call { name, args } => {
                if name == "print" && args.len() == 1 {
                    self.compile_print(&args[0])?;
                    return Ok(());
                }

                // Пользовательская функция
                let arg_regs = ["rcx", "rdx", "r8", "r9"];
                if args.len() > 4 {
                    return Err(format!("Функция '{}': максимум 4 аргумента", name));
                }

                // Вычисляем аргументы на стек
                for arg in args.iter().rev() {
                    self.compile_expr(arg)?;
                    self.emit_indent("push rax");
                }

                // Достаём в регистры
                for i in 0..args.len() {
                    self.emit_indent(&format!("pop {}", arg_regs[i]));
                }

                self.emit_indent("sub rsp, 32  ; shadow space");
                self.emit_indent(&format!("call __fn_{}", name));
                self.emit_indent("add rsp, 32");
            }
        }
        Ok(())
    }

	// Найдите fn compile_print и ЗАМЕНИТЕ ЦЕЛИКОМ:
	fn compile_print(&mut self, expr: &Expr) -> Result<(), String> {
		if self.expr_might_be_string(expr) {
			self.compile_expr_as_str(expr)?;
			self.emit_indent("mov rcx, rax");
			self.emit_indent("call __print_str");
		} else {
			match expr {
				Expr::StrLit(s) => {
					let label = self.add_string(s);
					self.emit_indent(&format!("lea rcx, [rel {}]", label));
					self.emit_indent("call __print_str");
				}
				Expr::IntLit(n) => {
					self.emit_indent(&format!("mov rcx, {}", n));
					self.emit_indent("call __print_int");
				}
				_ => {
					self.compile_expr(expr)?;
					self.emit_indent("mov rcx, rax");
					self.emit_indent("call __print_int");
				}
			}
		}
		Ok(())
	}

    fn compile_stmt(&mut self, stmt: &Stmt, loop_start: Option<&str>, loop_end: Option<&str>) -> Result<(), String> {
        match stmt {
            Stmt::Let { name, value } => {
                self.compile_expr(value)?;
                let offset = self.get_var_offset(name);
                self.emit_indent(&format!("mov {}, rax", offset));
            }
            Stmt::Assign { name, value } => {
                self.compile_expr(value)?;
                let offset = self.get_var_offset(name);
                self.emit_indent(&format!("mov {}, rax", offset));
            }
            Stmt::Print(expr) => {
                self.compile_print(expr)?;
            }
            Stmt::If { condition, then_block, else_block } => {
                let else_label = self.new_label("else");
                let end_label = self.new_label("endif");

                self.compile_expr(condition)?;
                self.emit_indent("test rax, rax");
                self.emit_indent(&format!("je {}", else_label));

                for s in then_block {
                    self.compile_stmt(s, loop_start, loop_end)?;
                }

                if else_block.is_some() {
                    self.emit_indent(&format!("jmp {}", end_label));
                }

                self.emit(&format!("{}:", else_label));

                if let Some(else_stmts) = else_block {
                    for s in else_stmts {
                        self.compile_stmt(s, loop_start, loop_end)?;
                    }
                }

                self.emit(&format!("{}:", end_label));
            }
            Stmt::Loop { body } => {
                let start = self.new_label("loop");
                let end = self.new_label("endloop");

                self.emit(&format!("{}:", start));

                for s in body {
                    self.compile_stmt(s, Some(&start), Some(&end))?;
                }

                self.emit_indent(&format!("jmp {}", start));
                self.emit(&format!("{}:", end));
            }
            Stmt::Break => {
                if let Some(end) = loop_end {
                    self.emit_indent(&format!("jmp {}", end));
                } else {
                    return Err("break вне цикла".to_string());
                }
            }
            Stmt::Return(expr) => {
                if let Some(e) = expr {
                    self.compile_expr(e)?;
                }
                self.emit_indent("mov rsp, rbp");
                self.emit_indent("pop rbp");
                self.emit_indent("ret");
            }
            Stmt::FnDef { name, params, body } => {
                let skip_label = self.new_label("skip_fn");
                self.emit_indent(&format!("jmp {}", skip_label));

                let arg_regs = ["rcx", "rdx", "r8", "r9"];
                self.functions.insert(name.clone(), params.len());

                self.emit(&format!("__fn_{}:", name));
                self.emit_indent("push rbp");
                self.emit_indent("mov rbp, rsp");
                self.emit_indent(&format!("sub rsp, {}", (params.len().max(1) + 16) * 8));

                // Сохраняем старые переменные
                let saved_vars = self.variables.clone();
                let saved_count = self.var_count;
                self.variables.clear();
                self.var_count = 0;

                for (i, param) in params.iter().enumerate() {
                    if i < 4 {
                        let offset = self.get_var_offset(param);
                        self.emit_indent(&format!("mov {}, {}", offset, arg_regs[i]));
                    }
                }

                for s in body {
                    self.compile_stmt(s, None, None)?;
                }

                self.emit_indent("xor rax, rax");
                self.emit_indent("mov rsp, rbp");
                self.emit_indent("pop rbp");
                self.emit_indent("ret");

                // Восстанавливаем
                self.variables = saved_vars;
                self.var_count = saved_count;

                self.emit(&format!("{}:", skip_label));
            }
            Stmt::ExprStmt(expr) => {
                self.compile_expr(expr)?;
            }
        }
        Ok(())
    }
	fn generate_print_str_routine(&mut self) {
		self.emit("");
		self.emit("; === Print string routine ===");
		self.emit("; Input: RCX = pointer to null-terminated string");
		self.emit("__print_str:");
		self.emit_indent("push rbp");
		self.emit_indent("mov rbp, rsp");
		self.emit_indent("sub rsp, 96");
		self.emit_indent("mov [rbp-8], rcx     ; save string ptr");
		self.emit_indent("");
		self.emit_indent("; strlen");
		self.emit_indent("xor rax, rax");
		self.emit_indent("mov rdi, rcx");
		self.emit("ps_strlen:");
		self.emit_indent("cmp byte [rdi+rax], 0");
		self.emit_indent("je ps_got_len");
		self.emit_indent("inc rax");
		self.emit_indent("jmp ps_strlen");
		self.emit("ps_got_len:");
		self.emit_indent("mov [rbp-16], rax    ; length");
		self.emit_indent("");
		self.emit_indent("sub rsp, 48");
		self.emit_indent("mov rcx, -11");
		self.emit_indent("call GetStdHandle");
		self.emit_indent("mov rcx, rax");
		self.emit_indent("mov rdx, [rbp-8]");
		self.emit_indent("mov r8, [rbp-16]");
		self.emit_indent("lea r9, [rbp-24]");
		self.emit_indent("mov qword [rsp+32], 0");
		self.emit_indent("call WriteFile");
		self.emit_indent("add rsp, 48");
		self.emit_indent("");
		self.emit_indent("; newline");
		self.emit_indent("sub rsp, 48");
		self.emit_indent("mov rcx, -11");
		self.emit_indent("call GetStdHandle");
		self.emit_indent("mov rcx, rax");
		self.emit_indent("lea rdx, [rel __newline]");
		self.emit_indent("mov r8, 1");
		self.emit_indent("lea r9, [rbp-24]");
		self.emit_indent("mov qword [rsp+32], 0");
		self.emit_indent("call WriteFile");
		self.emit_indent("add rsp, 48");
		self.emit_indent("");
		self.emit_indent("mov rsp, rbp");
		self.emit_indent("pop rbp");
		self.emit_indent("ret");
	}

	fn generate_int_to_str_routine(&mut self) {
		self.emit("");
		self.emit("; === Int to string routine ===");
		self.emit("; Input: RCX = number");
		self.emit("; Output: RAX = pointer to string in __fmt_int");
		self.emit("__int_to_str:");
		self.emit_indent("push rbp");
		self.emit_indent("mov rbp, rsp");
		self.emit_indent("sub rsp, 32");
		self.emit_indent("");
		self.emit_indent("mov rax, rcx");
		self.emit_indent("lea rdi, [rel __fmt_int+22]  ; end of buffer");
		self.emit_indent("mov byte [rdi], 0            ; null terminator");
		self.emit_indent("dec rdi");
		self.emit_indent("xor r8, r8                   ; negative flag");
		self.emit_indent("");
		self.emit_indent("test rax, rax");
		self.emit_indent("jnz its_not_zero");
		self.emit_indent("mov byte [rdi], '0'");
		self.emit_indent("mov rax, rdi");
		self.emit_indent("jmp its_done");
		self.emit("");
		self.emit("its_not_zero:");
		self.emit_indent("jns its_positive");
		self.emit_indent("neg rax");
		self.emit_indent("mov r8, 1");
		self.emit("");
		self.emit("its_positive:");
		self.emit("its_loop:");
		self.emit_indent("xor rdx, rdx");
		self.emit_indent("mov rbx, 10");
		self.emit_indent("div rbx");
		self.emit_indent("add dl, '0'");
		self.emit_indent("mov [rdi], dl");
		self.emit_indent("dec rdi");
		self.emit_indent("test rax, rax");
		self.emit_indent("jnz its_loop");
		self.emit_indent("");
		self.emit_indent("test r8, r8");
		self.emit_indent("jz its_no_minus");
		self.emit_indent("mov byte [rdi], '-'");
		self.emit_indent("dec rdi");
		self.emit("its_no_minus:");
		self.emit_indent("lea rax, [rdi+1]   ; start of string");
		self.emit("");
		self.emit("its_done:");
		self.emit_indent("mov rsp, rbp");
		self.emit_indent("pop rbp");
		self.emit_indent("ret");
	}

	fn generate_concat_routine(&mut self) {
		self.emit("");
		self.emit("; === String concat routine ===");
		self.emit("; Input: RCX = str1 ptr, RDX = str2 ptr");
		self.emit("; Output: RAX = pointer to __concat_buf");
		self.emit("__concat:");
		self.emit_indent("push rbp");
		self.emit_indent("mov rbp, rsp");
		self.emit_indent("sub rsp, 32");
		self.emit_indent("");
		self.emit_indent("mov rsi, rcx         ; src1");
		self.emit_indent("lea rdi, [rel __concat_buf]");
		self.emit_indent("");
		self.emit("; copy str1");
		self.emit("cc_copy1:");
		self.emit_indent("lodsb");
		self.emit_indent("test al, al");
		self.emit_indent("jz cc_str2");
		self.emit_indent("stosb");
		self.emit_indent("jmp cc_copy1");
		self.emit("");
		self.emit("cc_str2:");
		self.emit_indent("mov rsi, rdx         ; src2");
		self.emit("cc_copy2:");
		self.emit_indent("lodsb");
		self.emit_indent("test al, al");
		self.emit_indent("jz cc_end");
		self.emit_indent("stosb");
		self.emit_indent("jmp cc_copy2");
		self.emit("");
		self.emit("cc_end:");
		self.emit_indent("mov byte [rdi], 0    ; null terminator");
		self.emit_indent("lea rax, [rel __concat_buf]");
		self.emit_indent("");
		self.emit_indent("mov rsp, rbp");
		self.emit_indent("pop rbp");
		self.emit_indent("ret");
	}

	fn generate_print_value_routine(&mut self) {
		// Пустая — используем __print_str и __print_int напрямую
	}

	fn generate_print_int_routine(&mut self) {
		self.emit("");
		self.emit("; === Print integer routine ===");
		self.emit("; Input: RCX = number to print");
		self.emit("__print_int:");
		self.emit_indent("push rbp");
		self.emit_indent("mov rbp, rsp");
		self.emit_indent("sub rsp, 96");
		self.emit_indent("");
		self.emit_indent("mov rax, rcx");
		self.emit_indent("lea rdi, [rbp-48]");
		self.emit_indent("mov byte [rdi], 10");
		self.emit_indent("dec rdi");
		self.emit_indent("xor rcx, rcx");
		self.emit_indent("xor r8, r8");
		self.emit_indent("");
		self.emit_indent("test rax, rax");
		self.emit_indent("jnz pi_not_zero");
		self.emit_indent("mov byte [rdi], '0'");
		self.emit_indent("dec rdi");
		self.emit_indent("mov rcx, 2");
		self.emit_indent("jmp pi_write");
		self.emit("");
		self.emit("pi_not_zero:");
		self.emit_indent("jns pi_positive");
		self.emit_indent("neg rax");
		self.emit_indent("mov r8, 1");
		self.emit("");
		self.emit("pi_positive:");
		self.emit("pi_div_loop:");
		self.emit_indent("xor rdx, rdx");
		self.emit_indent("mov rbx, 10");
		self.emit_indent("div rbx");
		self.emit_indent("add dl, '0'");
		self.emit_indent("mov [rdi], dl");
		self.emit_indent("dec rdi");
		self.emit_indent("inc rcx");
		self.emit_indent("test rax, rax");
		self.emit_indent("jnz pi_div_loop");
		self.emit_indent("");
		self.emit_indent("test r8, r8");
		self.emit_indent("jz pi_no_minus");
		self.emit_indent("mov byte [rdi], '-'");
		self.emit_indent("dec rdi");
		self.emit_indent("inc rcx");
		self.emit("pi_no_minus:");
		self.emit_indent("inc rcx");
		self.emit("");
		self.emit("pi_write:");
		self.emit_indent("inc rdi");
		self.emit_indent("mov [rbp-64], rdi");
		self.emit_indent("mov [rbp-72], rcx");
		self.emit_indent("");
		self.emit_indent("sub rsp, 48");
		self.emit_indent("mov rcx, -11");
		self.emit_indent("call GetStdHandle");
		self.emit_indent("mov rcx, rax");
		self.emit_indent("mov rdx, [rbp-64]");
		self.emit_indent("mov r8, [rbp-72]");
		self.emit_indent("lea r9, [rbp-80]");
		self.emit_indent("mov qword [rsp+32], 0");
		self.emit_indent("call WriteFile");
		self.emit_indent("add rsp, 48");
		self.emit_indent("");
		self.emit_indent("mov rsp, rbp");
		self.emit_indent("pop rbp");
		self.emit_indent("ret");
	}

	pub fn compile(&mut self, stmts: &[Stmt]) -> Result<String, String> {
		self.asm.clear();
		self.data_section.clear();
		self.bss_section.clear();

		self.data_section.push_str("    __newline db 10\n");
		self.data_section.push_str("    __numbuf times 32 db 0\n");
		self.data_section.push_str("    __fmt_int times 24 db 0\n");

		let mut code = String::new();
		std::mem::swap(&mut self.asm, &mut code);

		for stmt in stmts {
			self.compile_stmt(stmt, None, None)?;
		}

		let mut body_code = String::new();
		std::mem::swap(&mut self.asm, &mut body_code);
		self.asm = code;

		let stack_size = (self.var_count.max(1) + 8) * 8;
		let stack_size = (stack_size + 15) & !15;

		let mut result = String::new();

		result.push_str("; Generated by Rust-- Compiler\n");
		result.push_str("; Assemble: nasm -f win64 output.asm -o output.obj\n");
		result.push_str("; Link: golink /entry:_start /console output.obj kernel32.dll\n");
		result.push_str("\n");
		result.push_str("bits 64\n");
		result.push_str("\n");

		result.push_str("extern GetStdHandle\n");
		result.push_str("extern WriteFile\n");
		result.push_str("extern ExitProcess\n");
		result.push_str("extern SetConsoleOutputCP\n");
		result.push_str("\n");

		result.push_str("section .data\n");
		result.push_str(&self.data_section);
		result.push_str("\n");

		result.push_str("section .bss\n");
		result.push_str(&self.bss_section);
		result.push_str("    __written resq 1\n");
		// Буфер для конкатенации строк (1024 байт)
		result.push_str("    __concat_buf resb 1024\n");
		result.push_str("\n");

		result.push_str("section .text\n");
		result.push_str("global _start\n");
		result.push_str("\n");

		result.push_str("_start:\n");
		result.push_str("    push rbp\n");
		result.push_str("    mov rbp, rsp\n");
		result.push_str(&format!("    sub rsp, {}\n", stack_size));
		result.push_str("\n");
		// Устанавливаем UTF-8 кодировку консоли
		result.push_str("    ; SetConsoleOutputCP(65001) - UTF-8\n");
		result.push_str("    sub rsp, 32\n");
		result.push_str("    mov rcx, 65001\n");
		result.push_str("    call SetConsoleOutputCP\n");
		result.push_str("    add rsp, 32\n");
		result.push_str("\n");

		result.push_str(&body_code);

		result.push_str("\n");
		result.push_str("    ; ExitProcess(0)\n");
		result.push_str("    sub rsp, 32\n");
		result.push_str("    xor rcx, rcx\n");
		result.push_str("    call ExitProcess\n");
		result.push_str("    add rsp, 32\n");
		result.push_str("    mov rsp, rbp\n");
		result.push_str("    pop rbp\n");
		result.push_str("    ret\n");

		// Runtime routines
		self.asm.clear();
		self.generate_print_int_routine();
		self.generate_print_str_routine();
		self.generate_int_to_str_routine();
		self.generate_concat_routine();
		self.generate_print_value_routine();
		result.push_str(&self.asm);

		result.push_str("\n");

		Ok(result)
	}
}

/// Генерирует NASM ассемблерный код из RS-- исходника
pub fn compile_to_asm(source: &str) -> Result<String, String> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().map_err(|e| e.to_string())?;
    let mut parser = Parser::new(tokens);
    let stmts = parser.parse_program().map_err(|e| e.to_string())?;

    let mut compiler = NasmCompiler::new();
    compiler.compile(&stmts)
}

/// Собирает .exe из RS-- через NASM + link
pub fn build_native_exe(source: &str, output_path: &std::path::Path) -> Result<String, String> {
    let asm_code = compile_to_asm(source)?;

    let temp_dir = std::env::temp_dir().join("rsmm_nasm_build");
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&temp_dir)
        .map_err(|e| format!("Не могу создать папку: {}", e))?;

    let asm_path = temp_dir.join("output.asm");
    let obj_path = temp_dir.join("output.obj");

    std::fs::write(&asm_path, &asm_code)
        .map_err(|e| format!("Не могу записать .asm: {}", e))?;

    let asm_debug_path = output_path.with_extension("asm");
    let _ = std::fs::write(&asm_debug_path, &asm_code);

    let nasm_output = std::process::Command::new("nasm")
        .arg("-f").arg("win64")
        .arg("-o").arg(&obj_path)
        .arg(&asm_path)
        .output()
        .map_err(|e| format!(
            "Не могу запустить NASM: {}\n\n\
             Установите NASM: https://www.nasm.us/\n\
             И добавьте в PATH", e
        ))?;

    if !nasm_output.status.success() {
        let stderr = String::from_utf8_lossy(&nasm_output.stderr);
        // НЕ удаляем при ошибке — для отладки
        return Err(format!(
            "Ошибка NASM:\n{}\n\n\
             ASM файл сохранён: {}",
            stderr,
            asm_debug_path.display()
        ));
    }

    let link_result = try_link(&obj_path, output_path);

    // ← ДОБАВЛЕНО: очищаем temp после линковки (независимо от результата)
    let _ = std::fs::remove_dir_all(&temp_dir);

    match link_result {
        Ok(_) => {
            let size = std::fs::metadata(output_path)
                .map(|m| m.len())
                .unwrap_or(0);

            Ok(format!(
                "✅ Нативный EXE собран!\n\
                 Путь: {}\n\
                 Размер: {} байт ({:.1} KB)\n\
                 ASM: {}\n\n\
                 Использован: NASM + Linker",
                output_path.display(),
                size,
                size as f64 / 1024.0,
                asm_debug_path.display()
            ))
        }
        Err(link_err) => {
            Err(format!(
                "NASM скомпилировал успешно, но линковка не удалась:\n{}\n\n\
                 ASM файл: {}\n\n\
                 Вы можете слинковать вручную:\n\
                 link output.obj /subsystem:console /entry:_start kernel32.lib\n\
                 или:\n\
                 golink /entry:_start output.obj kernel32.dll",
                link_err,
                asm_debug_path.display()
            ))
        }
    }
}

fn try_link(obj_path: &std::path::Path, output_path: &std::path::Path) -> Result<(), String> {
    // 1. Пробуем GoLink (простой, не требует VS)
    if let Ok(output) = std::process::Command::new("golink")
        .arg("/entry:_start")
        .arg("/console")
        .arg(obj_path)
        .arg("kernel32.dll")
        .arg(&format!("/fo:{}", output_path.display()))
        .output()
    {
        if output.status.success() {
            return Ok(());
        }
    }

    // 2. Пробуем найти MSVC link.exe по стандартным путям
    if let Some(msvc_link) = find_msvc_link() {
        if let Ok(output) = std::process::Command::new(&msvc_link)
            .arg(obj_path)
            .arg("/subsystem:console")
            .arg("/entry:_start")
            .arg("kernel32.lib")
            .arg(&format!("/out:{}", output_path.display()))
            .arg("/nodefaultlib")
            .output()
        {
            if output.status.success() {
                return Ok(());
            }
            let stderr = String::from_utf8_lossy(&output.stderr);
            if !stderr.is_empty() && stderr.contains("LNK") {
                return Err(format!("MSVC link.exe ошибка:\n{}", stderr));
            }
        }
    }

    // 3. Пробуем lld-link (LLVM)
    if let Ok(output) = std::process::Command::new("lld-link")
        .arg(obj_path)
        .arg("/subsystem:console")
        .arg("/entry:_start")
        .arg("kernel32.lib")
        .arg(&format!("/out:{}", output_path.display()))
        .arg("/nodefaultlib")
        .output()
    {
        if output.status.success() {
            return Ok(());
        }
    }

    // 4. Пробуем ld из MinGW
    if let Ok(output) = std::process::Command::new("ld")
        .arg("-e").arg("_start")
        .arg("-subsystem").arg("console")
        .arg(obj_path)
        .arg("-o").arg(output_path)
        .arg("-lkernel32")
        .output()
    {
        if output.status.success() {
            return Ok(());
        }
    }

    Err(
        "Линкер не найден!\n\n\
         Установите один из вариантов:\n\n\
         1. GoLink (рекомендуется — простой и лёгкий):\n\
            https://www.godevtool.com/\n\
            Скачайте, распакуйте, добавьте папку в PATH\n\n\
         2. Visual Studio Build Tools (link.exe):\n\
            https://visualstudio.microsoft.com/visual-cpp-build-tools/\n\n\
         3. LLVM (lld-link):\n\
            https://releases.llvm.org/\n\n\
         После установки перезапустите IDE.".to_string()
    )
}

fn find_msvc_link() -> Option<String> {
    // Проверяем через vswhere
    let vswhere_paths = vec![
        "C:\\Program Files (x86)\\Microsoft Visual Studio\\Installer\\vswhere.exe".to_string(),
        "C:\\Program Files\\Microsoft Visual Studio\\Installer\\vswhere.exe".to_string(),
    ];

    for vswhere in &vswhere_paths {
        if std::path::Path::new(vswhere).exists() {
            if let Ok(output) = std::process::Command::new(vswhere)
                .arg("-latest")
                .arg("-find")
                .arg("VC\\Tools\\MSVC\\*\\bin\\Hostx64\\x64\\link.exe")
                .output()
            {
                if output.status.success() {
                    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if !path.is_empty() {
                        // Берём первую найденную строку
                        if let Some(first_line) = path.lines().last() {
                            if std::path::Path::new(first_line).exists() {
                                return Some(first_line.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    // Проверяем стандартные пути вручную
    let years = ["2022", "2019", "2017"];
    let editions = ["Community", "Professional", "Enterprise", "BuildTools"];
    
    for year in &years {
        for edition in &editions {
            let base = format!(
                "C:\\Program Files\\Microsoft Visual Studio\\{}\\{}\\VC\\Tools\\MSVC",
                year, edition
            );
            let base2 = format!(
                "C:\\Program Files (x86)\\Microsoft Visual Studio\\{}\\{}\\VC\\Tools\\MSVC",
                year, edition
            );
            
            for base_path in &[&base, &base2] {
                if let Ok(entries) = std::fs::read_dir(base_path) {
                    for entry in entries.flatten() {
                        let link_path = entry.path()
                            .join("bin")
                            .join("Hostx64")
                            .join("x64")
                            .join("link.exe");
                        if link_path.exists() {
                            return Some(link_path.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }
    }

    None
}

/// Только генерация ASM (без сборки)
pub fn generate_asm(source: &str, output_path: &std::path::Path) -> Result<String, String> {
    let asm_code = compile_to_asm(source)?;

    std::fs::write(output_path, &asm_code)
        .map_err(|e| format!("Не могу записать файл: {}", e))?;

    Ok(format!(
        "✅ ASM файл создан!\n\
         Путь: {}\n\
         Строк: {}\n\n\
         Для сборки:\n\
         nasm -f win64 output.asm\n\
         golink /entry:_start /console output.obj kernel32.dll",
        output_path.display(),
        asm_code.lines().count()
    ))
}
// ======================== NASM + RUST HYBRID ========================

/// Компилирует проект с поддержкой extern_fn через Rust DLL
pub fn build_native_hybrid(
    source: &str,
    output_path: &std::path::Path,
) -> Result<String, String> {
    // Парсим расширенный синтаксис
    let mut ext_parser = ExtParser::new(source);
    let ext_stmts = ext_parser.parse_extended().map_err(|e| e.to_string())?;

    // Собираем зависимости, rust блоки и extern_fn
    let mut dependencies: Vec<RustDependency> = Vec::new();
    let mut rust_blocks: Vec<String> = Vec::new();
    let mut extern_fns: Vec<(String, Vec<String>, String)> = Vec::new();
    let mut normal_stmts: Vec<Stmt> = Vec::new();

    for stmt in &ext_stmts {
        match stmt {
            ExtStmt::UseCrate { name, version, features } => {
                let ver = version.clone().unwrap_or_else(|| "*".to_string());
                if features.is_empty() {
                    dependencies.push(RustDependency::new(name, &ver));
                } else {
                    let feats: Vec<&str> = features.iter().map(|s| s.as_str()).collect();
                    dependencies.push(RustDependency::with_features(name, &ver, feats));
                }
            }
            ExtStmt::RustBlock(code) => {
                rust_blocks.push(code.clone());
            }
            ExtStmt::ExternFn { name, params, rust_body } => {
                extern_fns.push((name.clone(), params.clone(), rust_body.clone()));
            }
            ExtStmt::Normal(s) => {
                normal_stmts.push(s.clone());
            }
            ExtStmt::Import(_) => {
                // TODO: обработка импортов
            }
        }
    }

    // Если нет extern_fn и зависимостей — используем чистый NASM
    if extern_fns.is_empty() && dependencies.is_empty() && rust_blocks.is_empty() {
        return build_native_exe(source, output_path);
    }

    // Иначе — гибридная сборка через Rust с inline asm
    build_hybrid_exe(&normal_stmts, &dependencies, &rust_blocks, &extern_fns, output_path)
}

/// Собирает гибридный exe: RS-- код + Rust extern_fn
fn build_hybrid_exe(
    stmts: &[Stmt],
    dependencies: &[RustDependency],
    rust_blocks: &[String],
    extern_fns: &[(String, Vec<String>, String)],
    output_path: &std::path::Path,
) -> Result<String, String> {
    let temp_dir = std::env::temp_dir().join("rsmm_hybrid_build");
    let src_dir = temp_dir.join("src");
    
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&src_dir)
        .map_err(|e| format!("Не могу создать папку: {}", e))?;

    // Генерируем Rust код
    let rust_code = generate_hybrid_rust(stmts, rust_blocks, extern_fns)?;
    
    // Cargo.toml
    let mut cargo_toml = r#"[package]
name = "rsmm_hybrid"
version = "0.1.0"
edition = "2021"

[dependencies]
"#.to_string();

    for dep in dependencies {
        cargo_toml.push_str(&dep.to_toml_line());
        cargo_toml.push('\n');
    }

    cargo_toml.push_str(r#"
[profile.release]
opt-level = "z"
lto = true
strip = true
"#);

    std::fs::write(temp_dir.join("Cargo.toml"), &cargo_toml)
        .map_err(|e| format!("Не могу записать Cargo.toml: {}", e))?;
    
    std::fs::write(src_dir.join("main.rs"), &rust_code)
        .map_err(|e| format!("Не могу записать main.rs: {}", e))?;

    // Сборка
    let output = std::process::Command::new("cargo")
        .arg("build")
        .arg("--release")
        .arg("--manifest-path")
        .arg(temp_dir.join("Cargo.toml"))
        .output()
        .map_err(|e| format!("Не могу запустить cargo: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "Ошибка компиляции гибридного exe:\n{}\n\nmain.rs: {}",
            stderr,
            src_dir.join("main.rs").display()
        ));
    }

    #[cfg(windows)]
    let exe_name = "rsmm_hybrid.exe";
    #[cfg(not(windows))]
    let exe_name = "rsmm_hybrid";

    let built_exe = temp_dir.join("target").join("release").join(exe_name);
    std::fs::copy(&built_exe, output_path)
        .map_err(|e| format!("Не могу скопировать exe: {}", e))?;

    let size = std::fs::metadata(output_path)
        .map(|m| m.len())
        .unwrap_or(0);

    let _ = std::fs::remove_dir_all(&temp_dir);

    Ok(format!(
        "✅ Гибридный EXE собран!\n\
         Путь: {}\n\
         Размер: {} KB\n\
         Режим: Rust + extern_fn\n\
         Зависимости: {}",
        output_path.display(),
        size / 1024,
        if dependencies.is_empty() { "нет".to_string() } 
        else { dependencies.iter().map(|d| d.name.clone()).collect::<Vec<_>>().join(", ") }
    ))
}

/// Генерирует Rust код с inline ASM для критичных частей
fn generate_hybrid_rust(
    stmts: &[Stmt],
    rust_blocks: &[String],
    extern_fns: &[(String, Vec<String>, String)],
) -> Result<String, String> {
    let mut code = String::new();

    code.push_str("#![allow(unused_variables, unused_mut, unused_assignments, dead_code)]\n\n");

    // Rust блоки (use statements)
    for block in rust_blocks {
        code.push_str(block);
        code.push_str("\n");
    }
    code.push_str("\n");

    // Runtime
    code.push_str(HYBRID_RUNTIME);
    code.push_str("\n");

    // Extern функции
    for (name, params, body) in extern_fns {
        let params_str = params
            .iter()
            .map(|p| format!("{}: RsmmVal", p))
            .collect::<Vec<_>>()
            .join(", ");
        
        code.push_str(&format!("fn {}({}) -> RsmmVal {{\n", name, params_str));
        code.push_str("    ");
        code.push_str(body);
        code.push_str("\n}\n\n");
    }

    // Функции пользователя
    let mut functions = Vec::new();
    let mut main_stmts = Vec::new();

    for stmt in stmts {
        match stmt {
            Stmt::FnDef { .. } => functions.push(stmt.clone()),
            _ => main_stmts.push(stmt.clone()),
        }
    }

    let transpiler = Transpiler::new();
    
    for func in &functions {
        if let Stmt::FnDef { name, params, body } = func {
            let params_str = params
                .iter()
                .map(|p| format!("{}: RsmmVal", p))
                .collect::<Vec<_>>()
                .join(", ");
            
            code.push_str(&format!("fn {}({}) {{\n", name, params_str));
            
            for p in params {
                code.push_str(&format!("    let mut {} = {};\n", p, p));
            }
            
            for s in body {
                let stmt_code = transpile_stmt_to_string(s, &transpiler)?;
                for line in stmt_code.lines() {
                    code.push_str("    ");
                    code.push_str(line);
                    code.push_str("\n");
                }
            }
            
            code.push_str("}\n\n");
        }
    }

    // Main
    code.push_str("fn main() {\n");
    for stmt in &main_stmts {
        let stmt_code = transpile_stmt_to_string(stmt, &transpiler)?;
        for line in stmt_code.lines() {
            code.push_str("    ");
            code.push_str(line);
            code.push_str("\n");
        }
    }
    code.push_str("}\n");

    Ok(code)
}

fn transpile_stmt_to_string(stmt: &Stmt, t: &Transpiler) -> Result<String, String> {
    let mut result = String::new();
    
    match stmt {
        Stmt::Let { name, value } => {
            let expr = t.transpile_expr(value)?;
            result.push_str(&format!("let mut {} = {};", name, expr));
        }
        Stmt::Assign { name, value } => {
            let expr = t.transpile_expr(value)?;
            result.push_str(&format!("{} = {};", name, expr));
        }
        Stmt::Print(expr) => {
            let e = t.transpile_expr(expr)?;
            result.push_str(&format!("println!(\"{{}}\", {});", e));
        }
        Stmt::If { condition, then_block, else_block } => {
            let cond = t.transpile_expr(condition)?;
            result.push_str(&format!("if ({}).as_bool() {{\n", cond));
            for s in then_block {
                let inner = transpile_stmt_to_string(s, t)?;
                result.push_str("    ");
                result.push_str(&inner);
                result.push_str("\n");
            }
            if let Some(else_stmts) = else_block {
                result.push_str("} else {\n");
                for s in else_stmts {
                    let inner = transpile_stmt_to_string(s, t)?;
                    result.push_str("    ");
                    result.push_str(&inner);
                    result.push_str("\n");
                }
            }
            result.push_str("}");
        }
        Stmt::Loop { body } => {
            result.push_str("loop {\n");
            for s in body {
                let inner = transpile_stmt_to_string(s, t)?;
                result.push_str("    ");
                result.push_str(&inner);
                result.push_str("\n");
            }
            result.push_str("}");
        }
        Stmt::Break => {
            result.push_str("break;");
        }
        Stmt::Return(expr) => {
            if let Some(e) = expr {
                let val = t.transpile_expr(e)?;
                result.push_str(&format!("return {};", val));
            } else {
                result.push_str("return;");
            }
        }
        Stmt::ExprStmt(expr) => {
            let e = t.transpile_expr(expr)?;
            result.push_str(&format!("{};", e));
        }
        Stmt::FnDef { .. } => {
            // Уже обработано отдельно
        }
    }
    
    Ok(result)
}

const HYBRID_RUNTIME: &str = r#"
#[derive(Clone, Debug)]
enum RsmmVal {
    Int(i64),
    Str(String),
    Bool(bool),
    None,
}

impl std::fmt::Display for RsmmVal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RsmmVal::Int(n) => write!(f, "{}", n),
            RsmmVal::Str(s) => write!(f, "{}", s),
            RsmmVal::Bool(b) => write!(f, "{}", b),
            RsmmVal::None => write!(f, "none"),
        }
    }
}

impl RsmmVal {
    fn as_int(&self) -> i64 {
        match self {
            RsmmVal::Int(n) => *n,
            RsmmVal::Bool(b) => if *b { 1 } else { 0 },
            RsmmVal::Str(s) => s.parse().unwrap_or(0),
            RsmmVal::None => 0,
        }
    }
    fn as_str(&self) -> String {
        format!("{}", self)
    }
    fn as_bool(&self) -> bool {
        match self {
            RsmmVal::Int(n) => *n != 0,
            RsmmVal::Str(s) => !s.is_empty(),
            RsmmVal::Bool(b) => *b,
            RsmmVal::None => false,
        }
    }
}

fn rsmm_add_impl(a: &RsmmVal, b: &RsmmVal) -> RsmmVal {
    match (a, b) {
        (RsmmVal::Int(x), RsmmVal::Int(y)) => RsmmVal::Int(x + y),
        (RsmmVal::Str(x), other) => RsmmVal::Str(format!("{}{}", x, other)),
        (other, RsmmVal::Str(y)) => RsmmVal::Str(format!("{}{}", other, y)),
        _ => RsmmVal::Str(format!("{}{}", a, b)),
    }
}

fn rsmm_sub(a: &RsmmVal, b: &RsmmVal) -> RsmmVal {
    RsmmVal::Int(a.as_int() - b.as_int())
}

fn rsmm_mul(a: &RsmmVal, b: &RsmmVal) -> RsmmVal {
    RsmmVal::Int(a.as_int() * b.as_int())
}

fn rsmm_div(a: &RsmmVal, b: &RsmmVal) -> RsmmVal {
    RsmmVal::Int(a.as_int() / b.as_int())
}

fn rsmm_mod(a: &RsmmVal, b: &RsmmVal) -> RsmmVal {
    RsmmVal::Int(a.as_int() % b.as_int())
}

fn rsmm_eq(a: &RsmmVal, b: &RsmmVal) -> RsmmVal {
    match (a, b) {
        (RsmmVal::Int(x), RsmmVal::Int(y)) => RsmmVal::Bool(x == y),
        (RsmmVal::Str(x), RsmmVal::Str(y)) => RsmmVal::Bool(x == y),
        (RsmmVal::Bool(x), RsmmVal::Bool(y)) => RsmmVal::Bool(x == y),
        _ => RsmmVal::Bool(false),
    }
}

fn rsmm_neq(a: &RsmmVal, b: &RsmmVal) -> RsmmVal {
    RsmmVal::Bool(!rsmm_eq(a, b).as_bool())
}

fn rsmm_lt(a: &RsmmVal, b: &RsmmVal) -> RsmmVal {
    RsmmVal::Bool(a.as_int() < b.as_int())
}

fn rsmm_gt(a: &RsmmVal, b: &RsmmVal) -> RsmmVal {
    RsmmVal::Bool(a.as_int() > b.as_int())
}

fn rsmm_lte(a: &RsmmVal, b: &RsmmVal) -> RsmmVal {
    RsmmVal::Bool(a.as_int() <= b.as_int())
}

fn rsmm_gte(a: &RsmmVal, b: &RsmmVal) -> RsmmVal {
    RsmmVal::Bool(a.as_int() >= b.as_int())
}
"#;
// ======================== NASM + RUST HYBRID ДЛЯ ПРОЕКТА ========================

/// Компилирует проект с поддержкой extern_fn через Rust
pub fn build_project_native_hybrid(
    project: &Project,
    output_path: &std::path::Path,
) -> Result<String, String> {
    // Собираем весь код проекта
    let mut all_stmts: Vec<ExtStmt> = Vec::new();
    let mut dependencies: Vec<RustDependency> = project.dependencies.clone();
    let mut rust_blocks: Vec<String> = project.rust_blocks.clone();
    let mut extern_fns: Vec<(String, Vec<String>, String)> = Vec::new();
    let mut imported_files: std::collections::HashSet<String> = std::collections::HashSet::new();

    // Находим main файл
    let main_file = project
        .files
        .iter()
        .find(|f| f.is_main)
        .ok_or("Нет главного файла (main)")?;

    // Парсим все файлы, начиная с main
    parse_project_files_recursive(
        &main_file.content,
        &main_file.name,
        project,
        &mut all_stmts,
        &mut dependencies,
        &mut rust_blocks,
        &mut extern_fns,
        &mut imported_files,
    )?;

    // Проверяем нужна ли гибридная сборка
    let needs_hybrid = !extern_fns.is_empty() || !dependencies.is_empty() || !rust_blocks.is_empty();

    if !needs_hybrid {
        // Чистый NASM — собираем все стейтменты
        let mut normal_stmts: Vec<Stmt> = Vec::new();
        for stmt in &all_stmts {
            if let ExtStmt::Normal(s) = stmt {
                normal_stmts.push(s.clone());
            }
        }
        
        // Компилируем в ASM
        let mut compiler = NasmCompiler::new();
        let asm_code = compiler.compile(&normal_stmts)?;
        
        // Сохраняем и собираем
        let temp_dir = std::env::temp_dir().join("rsmm_project_nasm");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir)
            .map_err(|e| format!("Не могу создать папку: {}", e))?;

        let asm_path = temp_dir.join("output.asm");
        let obj_path = temp_dir.join("output.obj");

        std::fs::write(&asm_path, &asm_code)
            .map_err(|e| format!("Не могу записать .asm: {}", e))?;

        // Сохраняем копию рядом с exe
        let asm_debug_path = output_path.with_extension("asm");
        let _ = std::fs::write(&asm_debug_path, &asm_code);

        // NASM
        let nasm_output = std::process::Command::new("nasm")
            .arg("-f").arg("win64")
            .arg("-o").arg(&obj_path)
            .arg(&asm_path)
            .output()
            .map_err(|e| format!("Не могу запустить NASM: {}", e))?;

        if !nasm_output.status.success() {
            let stderr = String::from_utf8_lossy(&nasm_output.stderr);
            return Err(format!("Ошибка NASM:\n{}", stderr));
        }

        // Линковка
        let link_result = try_link(&obj_path, output_path);
        let _ = std::fs::remove_dir_all(&temp_dir);

        match link_result {
            Ok(_) => {
                let size = std::fs::metadata(output_path).map(|m| m.len()).unwrap_or(0);
                Ok(format!(
                    "✅ Нативный EXE (проект) собран!\n\
                     Путь: {}\n\
                     Размер: {} байт ({:.1} KB)\n\
                     Файлов: {}\n\
                     ASM: {}",
                    output_path.display(),
                    size,
                    size as f64 / 1024.0,
                    imported_files.len() + 1,
                    asm_debug_path.display()
                ))
            }
            Err(e) => Err(e)
        }
    } else {
        // Гибридная сборка через Rust
        build_project_hybrid_exe(
            &all_stmts,
            &dependencies,
            &rust_blocks,
            &extern_fns,
            &project.name,
            output_path,
        )
    }
}

fn parse_project_files_recursive(
    content: &str,
    filename: &str,
    project: &Project,
    all_stmts: &mut Vec<ExtStmt>,
    dependencies: &mut Vec<RustDependency>,
    rust_blocks: &mut Vec<String>,
    extern_fns: &mut Vec<(String, Vec<String>, String)>,
    imported_files: &mut std::collections::HashSet<String>,
) -> Result<(), String> {
    let mut parser = ExtParser::new(content);
    let stmts = parser.parse_extended().map_err(|e| format!("[{}] {}", filename, e))?;

    for stmt in stmts {
        match &stmt {
            ExtStmt::UseCrate { name, version, features } => {
                let ver = version.clone().unwrap_or_else(|| "*".to_string());
                let dep = if features.is_empty() {
                    RustDependency::new(name, &ver)
                } else {
                    let feats: Vec<&str> = features.iter().map(|s| s.as_str()).collect();
                    RustDependency::with_features(name, &ver, feats)
                };
                // Избегаем дубликатов
                if !dependencies.iter().any(|d| d.name == dep.name) {
                    dependencies.push(dep);
                }
            }
            ExtStmt::RustBlock(code) => {
                if !rust_blocks.contains(code) {
                    rust_blocks.push(code.clone());
                }
            }
            ExtStmt::ExternFn { name, params, rust_body } => {
                if !extern_fns.iter().any(|(n, _, _)| n == name) {
                    extern_fns.push((name.clone(), params.clone(), rust_body.clone()));
                }
            }
            ExtStmt::Import(path) => {
                if imported_files.insert(path.clone()) {
                    if let Some(imported) = project.files.iter().find(|f| f.name == *path) {
                        parse_project_files_recursive(
                            &imported.content,
                            &imported.name,
                            project,
                            all_stmts,
                            dependencies,
                            rust_blocks,
                            extern_fns,
                            imported_files,
                        )?;
                    } else {
                        return Err(format!("Файл '{}' не найден в проекте", path));
                    }
                }
            }
            ExtStmt::Normal(_) => {
                all_stmts.push(stmt);
            }
        }
    }

    Ok(())
}

fn build_project_hybrid_exe(
    stmts: &[ExtStmt],
    dependencies: &[RustDependency],
    rust_blocks: &[String],
    extern_fns: &[(String, Vec<String>, String)],
    project_name: &str,
    output_path: &std::path::Path,
) -> Result<String, String> {
    let temp_dir = std::env::temp_dir().join(format!("rsmm_hybrid_{}", project_name));
    let src_dir = temp_dir.join("src");
    
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&src_dir)
        .map_err(|e| format!("Не могу создать папку: {}", e))?;

    // Собираем только Normal стейтменты
    let normal_stmts: Vec<Stmt> = stmts
        .iter()
        .filter_map(|s| {
            if let ExtStmt::Normal(stmt) = s {
                Some(stmt.clone())
            } else {
                None
            }
        })
        .collect();

    // Генерируем Rust код
    let rust_code = generate_hybrid_rust(&normal_stmts, rust_blocks, extern_fns)?;
    
    // Cargo.toml
    let mut cargo_toml = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
"#,
        project_name
    );

    let mut seen_deps = std::collections::HashSet::new();
    for dep in dependencies {
        if seen_deps.insert(dep.name.clone()) {
            cargo_toml.push_str(&dep.to_toml_line());
            cargo_toml.push('\n');
        }
    }

    cargo_toml.push_str(r#"
[profile.release]
opt-level = "z"
lto = true
strip = true
"#);

    std::fs::write(temp_dir.join("Cargo.toml"), &cargo_toml)
        .map_err(|e| format!("Не могу записать Cargo.toml: {}", e))?;
    
    std::fs::write(src_dir.join("main.rs"), &rust_code)
        .map_err(|e| format!("Не могу записать main.rs: {}", e))?;

    // Сборка
    let output = std::process::Command::new("cargo")
        .arg("build")
        .arg("--release")
        .arg("--manifest-path")
        .arg(temp_dir.join("Cargo.toml"))
        .output()
        .map_err(|e| format!("Не могу запустить cargo: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "Ошибка компиляции:\n{}\n\nmain.rs: {}",
            stderr,
            src_dir.join("main.rs").display()
        ));
    }

    #[cfg(windows)]
    let exe_name = format!("{}.exe", project_name);
    #[cfg(not(windows))]
    let exe_name = project_name.to_string();

    let built_exe = temp_dir.join("target").join("release").join(&exe_name);
    std::fs::copy(&built_exe, output_path)
        .map_err(|e| format!("Не могу скопировать exe: {}", e))?;

    let size = std::fs::metadata(output_path)
        .map(|m| m.len())
        .unwrap_or(0);

    let _ = std::fs::remove_dir_all(&temp_dir);

    let deps_str = if dependencies.is_empty() {
        "нет".to_string()
    } else {
        dependencies.iter().map(|d| d.name.clone()).collect::<Vec<_>>().join(", ")
    };

    Ok(format!(
        "✅ Гибридный EXE (проект) собран!\n\
         Путь: {}\n\
         Размер: {} KB\n\
         Режим: Rust + extern_fn\n\
         Зависимости: {}",
        output_path.display(),
        size / 1024,
        deps_str
    ))
}
// ======================== ЧЕСТНАЯ ГИБРИДНАЯ СБОРКА ========================
// NASM для основного кода + Rust staticlib для extern_fn

/// Собирает проект: NASM код + Rust библиотека
pub fn build_true_hybrid(
    source: &str,
    output_path: &std::path::Path,
) -> Result<String, String> {
    // Парсим расширенный синтаксис
    let mut ext_parser = ExtParser::new(source);
    let ext_stmts = ext_parser.parse_extended().map_err(|e| e.to_string())?;

    // Разделяем на компоненты
    let mut dependencies: Vec<RustDependency> = Vec::new();
    let mut rust_blocks: Vec<String> = Vec::new();
    let mut extern_fns: Vec<(String, Vec<String>, String)> = Vec::new();
    let mut normal_stmts: Vec<Stmt> = Vec::new();

    for stmt in &ext_stmts {
        match stmt {
            ExtStmt::UseCrate { name, version, features } => {
                let ver = version.clone().unwrap_or_else(|| "*".to_string());
                if features.is_empty() {
                    dependencies.push(RustDependency::new(name, &ver));
                } else {
                    let feats: Vec<&str> = features.iter().map(|s| s.as_str()).collect();
                    dependencies.push(RustDependency::with_features(name, &ver, feats));
                }
            }
            ExtStmt::RustBlock(code) => {
                rust_blocks.push(code.clone());
            }
            ExtStmt::ExternFn { name, params, rust_body } => {
                extern_fns.push((name.clone(), params.clone(), rust_body.clone()));
            }
            ExtStmt::Normal(s) => {
                normal_stmts.push(s.clone());
            }
            ExtStmt::Import(_) => {}
        }
    }

    // Если нет extern функций — чистый NASM
    if extern_fns.is_empty() && dependencies.is_empty() {
        return build_native_exe(source, output_path);
    }

    // Создаём рабочую директорию
    let temp_dir = std::env::temp_dir().join("rsmm_true_hybrid");
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&temp_dir)
        .map_err(|e| format!("Не могу создать папку: {}", e))?;

    // 1. Генерируем Rust bridge библиотеку
    let bridge_result = generate_rust_bridge(
        &temp_dir,
        &dependencies,
        &rust_blocks,
        &extern_fns,
    )?;

    // 2. Генерируем NASM код с вызовами bridge функций
    let asm_code = generate_hybrid_asm(&normal_stmts, &extern_fns)?;
    
    let asm_path = temp_dir.join("main.asm");
    let obj_path = temp_dir.join("main.obj");
    
    std::fs::write(&asm_path, &asm_code)
        .map_err(|e| format!("Не могу записать ASM: {}", e))?;

    // Сохраняем копию для отладки
    let debug_asm = output_path.with_extension("asm");
    let _ = std::fs::write(&debug_asm, &asm_code);

    // 3. Компилируем ASM в OBJ
    let nasm_output = std::process::Command::new("nasm")
        .arg("-f").arg("win64")
        .arg("-o").arg(&obj_path)
        .arg(&asm_path)
        .output()
        .map_err(|e| format!("NASM не найден: {}", e))?;

    if !nasm_output.status.success() {
        let stderr = String::from_utf8_lossy(&nasm_output.stderr);
        return Err(format!("Ошибка NASM:\n{}\n\nASM: {}", stderr, debug_asm.display()));
    }

    // 4. Линкуем OBJ + LIB
    let link_result = link_hybrid(
        &obj_path,
        &bridge_result.lib_path,
        output_path,
        &bridge_result.additional_libs,
    );

    // Очистка (оставляем при ошибке для отладки)
    if link_result.is_ok() {
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    match link_result {
        Ok(_) => {
            let size = std::fs::metadata(output_path).map(|m| m.len()).unwrap_or(0);
            Ok(format!(
                "✅ Гибридный EXE собран!\n\
                 Путь: {}\n\
                 Размер: {} байт ({:.1} KB)\n\
                 Режим: NASM + Rust bridge\n\
                 Extern функций: {}\n\
                 Зависимостей: {}\n\
                 ASM: {}",
                output_path.display(),
                size,
                size as f64 / 1024.0,
                extern_fns.len(),
                dependencies.len(),
                debug_asm.display()
            ))
        }
        Err(e) => Err(format!(
            "Ошибка линковки:\n{}\n\n\
             ASM: {}\n\
             Bridge lib: {}",
            e,
            debug_asm.display(),
            bridge_result.lib_path.display()
        ))
    }
}

struct BridgeResult {
    lib_path: std::path::PathBuf,
    additional_libs: Vec<String>,
}

/// Генерирует Rust staticlib с extern "C" функциями
fn generate_rust_bridge(
    temp_dir: &std::path::Path,
    dependencies: &[RustDependency],
    rust_blocks: &[String],
    extern_fns: &[(String, Vec<String>, String)],
) -> Result<BridgeResult, String> {
    let bridge_dir = temp_dir.join("bridge");
    let src_dir = bridge_dir.join("src");
    std::fs::create_dir_all(&src_dir)
        .map_err(|e| format!("Не могу создать bridge папку: {}", e))?;

    // Cargo.toml для staticlib
    let mut cargo_toml = r#"[package]
name = "rsmm_bridge"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["staticlib"]

[dependencies]
"#.to_string();

    for dep in dependencies {
        cargo_toml.push_str(&dep.to_toml_line());
        cargo_toml.push('\n');
    }

    cargo_toml.push_str(r#"
[profile.release]
opt-level = "z"
lto = true
"#);

    std::fs::write(bridge_dir.join("Cargo.toml"), &cargo_toml)
        .map_err(|e| format!("Не могу записать Cargo.toml: {}", e))?;

    // lib.rs с extern "C" функциями
    let lib_code = generate_bridge_lib(rust_blocks, extern_fns);
    
    std::fs::write(src_dir.join("lib.rs"), &lib_code)
        .map_err(|e| format!("Не могу записать lib.rs: {}", e))?;

    // Сохраняем для отладки
    let debug_lib = temp_dir.join("bridge_lib.rs");
    let _ = std::fs::write(&debug_lib, &lib_code);

    // Собираем staticlib
    let output = std::process::Command::new("cargo")
        .arg("build")
        .arg("--release")
        .arg("--manifest-path")
        .arg(bridge_dir.join("Cargo.toml"))
        .output()
        .map_err(|e| format!("Cargo не найден: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "Ошибка компиляции bridge:\n{}\n\nlib.rs: {}",
            stderr,
            debug_lib.display()
        ));
    }

    let lib_path = bridge_dir
        .join("target")
        .join("release")
        .join("rsmm_bridge.lib");

    if !lib_path.exists() {
        // На некоторых системах может быть .a
        let alt_path = bridge_dir
            .join("target")
            .join("release")
            .join("librsmm_bridge.a");
        if alt_path.exists() {
            return Ok(BridgeResult {
                lib_path: alt_path,
                additional_libs: get_rust_system_libs(),
            });
        }
        return Err(format!("Bridge библиотека не создана: {}", lib_path.display()));
    }

    Ok(BridgeResult {
        lib_path,
        additional_libs: get_rust_system_libs(),
    })
}

/// Генерирует код Rust библиотеки с extern "C" экспортами
fn generate_bridge_lib(
    rust_blocks: &[String],
    extern_fns: &[(String, Vec<String>, String)],
) -> String {
    let mut code = String::new();

    code.push_str("#![allow(unused, non_snake_case)]\n\n");

    // Use statements
    for block in rust_blocks {
        code.push_str(block);
        code.push_str("\n");
    }
    code.push_str("\n");

    // Тип для передачи значений через FFI
    code.push_str(r#"
/// FFI-безопасное представление значения
#[repr(C)]
pub struct FfiValue {
    pub tag: i32,      // 0 = Int, 1 = Str, 2 = Bool, 3 = None
    pub int_val: i64,
    pub str_ptr: *mut i8,
    pub str_len: usize,
    pub bool_val: i32,
}

impl FfiValue {
    pub fn from_int(n: i64) -> Self {
        FfiValue {
            tag: 0,
            int_val: n,
            str_ptr: std::ptr::null_mut(),
            str_len: 0,
            bool_val: 0,
        }
    }
    
    pub fn from_str(s: &str) -> Self {
        let bytes = s.as_bytes();
        let ptr = unsafe {
            let layout = std::alloc::Layout::from_size_align(bytes.len() + 1, 1).unwrap();
            let ptr = std::alloc::alloc(layout);
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, bytes.len());
            *ptr.add(bytes.len()) = 0; // null terminator
            ptr as *mut i8
        };
        FfiValue {
            tag: 1,
            int_val: 0,
            str_ptr: ptr,
            str_len: bytes.len(),
            bool_val: 0,
        }
    }
    
    pub fn from_bool(b: bool) -> Self {
        FfiValue {
            tag: 2,
            int_val: 0,
            str_ptr: std::ptr::null_mut(),
            str_len: 0,
            bool_val: if b { 1 } else { 0 },
        }
    }
    
    pub fn none() -> Self {
        FfiValue {
            tag: 3,
            int_val: 0,
            str_ptr: std::ptr::null_mut(),
            str_len: 0,
            bool_val: 0,
        }
    }
}

/// Внутренний тип для работы в Rust коде
#[derive(Clone, Debug)]
pub enum RsmmVal {
    Int(i64),
    Str(String),
    Bool(bool),
    None,
}

impl RsmmVal {
    pub fn as_int(&self) -> i64 {
        match self {
            RsmmVal::Int(n) => *n,
            RsmmVal::Bool(b) => if *b { 1 } else { 0 },
            RsmmVal::Str(s) => s.parse().unwrap_or(0),
            RsmmVal::None => 0,
        }
    }
    
    pub fn as_str(&self) -> String {
        match self {
            RsmmVal::Int(n) => n.to_string(),
            RsmmVal::Str(s) => s.clone(),
            RsmmVal::Bool(b) => b.to_string(),
            RsmmVal::None => "none".to_string(),
        }
    }
    
    pub fn as_bool(&self) -> bool {
        match self {
            RsmmVal::Int(n) => *n != 0,
            RsmmVal::Str(s) => !s.is_empty(),
            RsmmVal::Bool(b) => *b,
            RsmmVal::None => false,
        }
    }
    
    pub fn to_ffi(&self) -> FfiValue {
        match self {
            RsmmVal::Int(n) => FfiValue::from_int(*n),
            RsmmVal::Str(s) => FfiValue::from_str(s),
            RsmmVal::Bool(b) => FfiValue::from_bool(*b),
            RsmmVal::None => FfiValue::none(),
        }
    }
    
    pub unsafe fn from_ffi(ffi: &FfiValue) -> Self {
        match ffi.tag {
            0 => RsmmVal::Int(ffi.int_val),
            1 => {
                if ffi.str_ptr.is_null() {
                    RsmmVal::Str(String::new())
                } else {
                    let slice = std::slice::from_raw_parts(ffi.str_ptr as *const u8, ffi.str_len);
                    RsmmVal::Str(String::from_utf8_lossy(slice).to_string())
                }
            }
            2 => RsmmVal::Bool(ffi.bool_val != 0),
            _ => RsmmVal::None,
        }
    }
}

"#);

    // Генерируем extern "C" обёртки для каждой функции
    for (name, params, body) in extern_fns {
        // Внутренняя функция
        let params_str = params
            .iter()
            .map(|p| format!("{}: RsmmVal", p))
            .collect::<Vec<_>>()
            .join(", ");
        
        code.push_str(&format!(
            "fn {}_impl({}) -> RsmmVal {{\n    {}\n}}\n\n",
            name, params_str, body
        ));

        // Extern "C" обёртка
        let ffi_params: Vec<String> = params
            .iter()
            .enumerate()
            .map(|(i, _)| format!("arg{}: *const FfiValue", i))
            .collect();
        
        code.push_str(&format!(
            "#[no_mangle]\npub extern \"C\" fn rsmm_{}({}) -> FfiValue {{\n",
            name,
            ffi_params.join(", ")
        ));
        
        code.push_str("    unsafe {\n");
        
        // Конвертируем аргументы
        for (i, param) in params.iter().enumerate() {
            code.push_str(&format!(
                "        let {} = RsmmVal::from_ffi(&*arg{});\n",
                param, i
            ));
        }
        
        // Вызываем внутреннюю функцию
        let args_call = params.join(", ");
        code.push_str(&format!(
            "        let result = {}_impl({});\n",
            name, args_call
        ));
        code.push_str("        result.to_ffi()\n");
        code.push_str("    }\n");
        code.push_str("}\n\n");
    }

    // Функция для освобождения строки
    code.push_str(r#"
#[no_mangle]
pub extern "C" fn rsmm_free_str(ptr: *mut i8) {
    if !ptr.is_null() {
        unsafe {
            // Находим длину строки
            let mut len = 0;
            while *ptr.add(len) != 0 {
                len += 1;
            }
            let layout = std::alloc::Layout::from_size_align(len + 1, 1).unwrap();
            std::alloc::dealloc(ptr as *mut u8, layout);
        }
    }
}
"#);

    code
}

/// Системные библиотеки нужные для Rust staticlib
fn get_rust_system_libs() -> Vec<String> {
    vec![
        "advapi32.lib".to_string(),
        "bcrypt.lib".to_string(),
        "kernel32.lib".to_string(),
        "ntdll.lib".to_string(),
        "userenv.lib".to_string(),
        "ws2_32.lib".to_string(),
        "msvcrt.lib".to_string(),
    ]
}

/// Специальный NASM компилятор с поддержкой вызовов extern функций
/// Специальный NASM компилятор с поддержкой вызовов extern функций
struct HybridNasmCompiler {
    asm: String,
    data_section: String,
    bss_section: String,
    label_counter: usize,
    variables: HashMap<String, usize>,
    var_count: usize,
    string_counter: usize,
    functions: HashMap<String, usize>,
    extern_fns: HashMap<String, usize>, // имя -> кол-во параметров
    ffi_value_counter: usize,
}

impl HybridNasmCompiler {
    fn new(extern_fns_list: &[(String, Vec<String>, String)]) -> Self {
        let mut extern_map = HashMap::new();
        for (name, params, _) in extern_fns_list {
            extern_map.insert(name.clone(), params.len());
        }
        HybridNasmCompiler {
            asm: String::new(),
            data_section: String::new(),
            bss_section: String::new(),
            label_counter: 0,
            variables: HashMap::new(),
            var_count: 0,
            string_counter: 0,
            functions: HashMap::new(),
            extern_fns: extern_map,
            ffi_value_counter: 0,
        }
    }

    fn new_label(&mut self, prefix: &str) -> String {
        self.label_counter += 1;
        format!("L_{}_{}", prefix, self.label_counter)
    }

    fn emit(&mut self, line: &str) {
        self.asm.push_str(line);
        self.asm.push('\n');
    }

    fn emit_indent(&mut self, line: &str) {
        self.asm.push_str("    ");
        self.asm.push_str(line);
        self.asm.push('\n');
    }

    fn add_string(&mut self, s: &str) -> String {
        self.string_counter += 1;
        let label = format!("__str_{}", self.string_counter);
        
        let mut nasm_str = String::new();
        let mut in_quote = false;
        
        for ch in s.chars() {
            match ch {
                '\n' => {
                    if in_quote {
                        nasm_str.push_str("\", ");
                        in_quote = false;
                    }
                    nasm_str.push_str("10, ");
                }
                '\r' => {
                    if in_quote {
                        nasm_str.push_str("\", ");
                        in_quote = false;
                    }
                    nasm_str.push_str("13, ");
                }
                '\t' => {
                    if in_quote {
                        nasm_str.push_str("\", ");
                        in_quote = false;
                    }
                    nasm_str.push_str("9, ");
                }
                _ => {
                    if !in_quote {
                        nasm_str.push('"');
                        in_quote = true;
                    }
                    nasm_str.push(ch);
                }
            }
        }
        if in_quote {
            nasm_str.push('"');
        }
        
        let nasm_str = nasm_str.trim_end_matches(", ").to_string();
        
        if nasm_str.is_empty() {
            self.data_section.push_str(&format!("    {} db 0\n", label));
        } else {
            self.data_section.push_str(&format!("    {} db {}, 0\n", label, nasm_str));
        }
        
        label
    }

    fn get_var_offset(&mut self, name: &str) -> String {
        if let Some(&idx) = self.variables.get(name) {
            format!("[rbp-{}]", (idx + 1) * 8)
        } else {
            self.var_count += 1;
            self.variables.insert(name.to_string(), self.var_count - 1);
            format!("[rbp-{}]", self.var_count * 8)
        }
    }

    fn is_extern_fn(&self, name: &str) -> bool {
        self.extern_fns.contains_key(name)
    }

    fn compile_expr(&mut self, expr: &Expr) -> Result<(), String> {
        match expr {
            Expr::IntLit(n) => {
                self.emit_indent(&format!("mov rax, {}", n));
            }
            Expr::BoolLit(b) => {
                self.emit_indent(&format!("mov rax, {}", if *b { 1 } else { 0 }));
            }
            Expr::StrLit(s) => {
                let label = self.add_string(s);
                self.emit_indent(&format!("lea rax, [rel {}]", label));
            }
            Expr::Ident(name) => {
                let offset = self.get_var_offset(name);
                self.emit_indent(&format!("mov rax, {}", offset));
            }
            Expr::BinOp { op, left, right } => {
                self.compile_expr(right)?;
                self.emit_indent("push rax");
                self.compile_expr(left)?;
                self.emit_indent("pop rbx");
                
                match op {
                    BinOp::Add => self.emit_indent("add rax, rbx"),
                    BinOp::Sub => self.emit_indent("sub rax, rbx"),
                    BinOp::Mul => self.emit_indent("imul rax, rbx"),
                    BinOp::Div => {
                        self.emit_indent("cqo");
                        self.emit_indent("idiv rbx");
                    }
                    BinOp::Mod => {
                        self.emit_indent("cqo");
                        self.emit_indent("idiv rbx");
                        self.emit_indent("mov rax, rdx");
                    }
                    BinOp::Eq => {
                        self.emit_indent("cmp rax, rbx");
                        self.emit_indent("sete al");
                        self.emit_indent("movzx rax, al");
                    }
                    BinOp::Neq => {
                        self.emit_indent("cmp rax, rbx");
                        self.emit_indent("setne al");
                        self.emit_indent("movzx rax, al");
                    }
                    BinOp::Lt => {
                        self.emit_indent("cmp rax, rbx");
                        self.emit_indent("setl al");
                        self.emit_indent("movzx rax, al");
                    }
                    BinOp::Gt => {
                        self.emit_indent("cmp rax, rbx");
                        self.emit_indent("setg al");
                        self.emit_indent("movzx rax, al");
                    }
                    BinOp::Lte => {
                        self.emit_indent("cmp rax, rbx");
                        self.emit_indent("setle al");
                        self.emit_indent("movzx rax, al");
                    }
                    BinOp::Gte => {
                        self.emit_indent("cmp rax, rbx");
                        self.emit_indent("setge al");
                        self.emit_indent("movzx rax, al");
                    }
                    BinOp::And => {
                        self.emit_indent("test rax, rax");
                        self.emit_indent("setne al");
                        self.emit_indent("test rbx, rbx");
                        self.emit_indent("setne bl");
                        self.emit_indent("and al, bl");
                        self.emit_indent("movzx rax, al");
                    }
                    BinOp::Or => {
                        self.emit_indent("test rax, rax");
                        self.emit_indent("setne al");
                        self.emit_indent("test rbx, rbx");
                        self.emit_indent("setne bl");
                        self.emit_indent("or al, bl");
                        self.emit_indent("movzx rax, al");
                    }
                }
            }
            Expr::UnaryOp { op, expr } => {
                self.compile_expr(expr)?;
                match op {
                    UnaryOp::Neg => self.emit_indent("neg rax"),
                    UnaryOp::Not => {
                        self.emit_indent("test rax, rax");
                        self.emit_indent("sete al");
                        self.emit_indent("movzx rax, al");
                    }
                }
            }
            Expr::Call { name, args } => {
                if name == "print" && args.len() == 1 {
                    self.compile_print(&args[0])?;
                    return Ok(());
                }

                // Проверяем это extern_fn или обычная функция
                if self.is_extern_fn(name) {
                    self.compile_extern_call(name, args)?;
                } else {
                    // Обычная функция
                    let arg_regs = ["rcx", "rdx", "r8", "r9"];
                    if args.len() > 4 {
                        return Err(format!("Функция '{}': максимум 4 аргумента", name));
                    }

                    for arg in args.iter().rev() {
                        self.compile_expr(arg)?;
                        self.emit_indent("push rax");
                    }

                    for i in 0..args.len() {
                        self.emit_indent(&format!("pop {}", arg_regs[i]));
                    }

                    self.emit_indent("sub rsp, 32");
                    self.emit_indent(&format!("call __fn_{}", name));
                    self.emit_indent("add rsp, 32");
                }
            }
        }
        Ok(())
    }

    /// Компилирует вызов extern функции через bridge
    fn compile_extern_call(&mut self, name: &str, args: &[Expr]) -> Result<(), String> {
        let param_count = *self.extern_fns.get(name).unwrap_or(&0);
        
        if args.len() != param_count {
            return Err(format!(
                "Функция '{}' ожидает {} аргументов, получено {}",
                name, param_count, args.len()
            ));
        }

        // Для каждого аргумента создаём FfiValue на стеке
        // FfiValue: 40 байт (tag:4, padding:4, int_val:8, str_ptr:8, str_len:8, bool_val:4, padding:4)
        let ffi_size = 48; // выравнивание до 8 байт
        let total_ffi_space = args.len() * ffi_size;
        
        if total_ffi_space > 0 {
            self.emit_indent(&format!("sub rsp, {}", total_ffi_space));
        }

        // Заполняем FfiValue для каждого аргумента
        for (i, arg) in args.iter().enumerate() {
            let ffi_offset = i * ffi_size;
            
            match arg {
                Expr::IntLit(n) => {
                    // tag = 0 (Int)
                    self.emit_indent(&format!("mov dword [rsp+{}], 0", ffi_offset));
                    // int_val
                    self.emit_indent(&format!("mov qword [rsp+{}], {}", ffi_offset + 8, n));
                    // str_ptr = 0
                    self.emit_indent(&format!("mov qword [rsp+{}], 0", ffi_offset + 16));
                    // str_len = 0
                    self.emit_indent(&format!("mov qword [rsp+{}], 0", ffi_offset + 24));
                    // bool_val = 0
                    self.emit_indent(&format!("mov dword [rsp+{}], 0", ffi_offset + 32));
                }
                Expr::StrLit(s) => {
                    let label = self.add_string(s);
                    // tag = 1 (Str)
                    self.emit_indent(&format!("mov dword [rsp+{}], 1", ffi_offset));
                    // int_val = 0
                    self.emit_indent(&format!("mov qword [rsp+{}], 0", ffi_offset + 8));
                    // str_ptr
                    self.emit_indent(&format!("lea rax, [rel {}]", label));
                    self.emit_indent(&format!("mov qword [rsp+{}], rax", ffi_offset + 16));
                    // str_len
                    self.emit_indent(&format!("mov qword [rsp+{}], {}", ffi_offset + 24, s.len()));
                    // bool_val = 0
                    self.emit_indent(&format!("mov dword [rsp+{}], 0", ffi_offset + 32));
                }
                Expr::BoolLit(b) => {
                    // tag = 2 (Bool)
                    self.emit_indent(&format!("mov dword [rsp+{}], 2", ffi_offset));
                    // int_val = 0
                    self.emit_indent(&format!("mov qword [rsp+{}], 0", ffi_offset + 8));
                    // str_ptr = 0
                    self.emit_indent(&format!("mov qword [rsp+{}], 0", ffi_offset + 16));
                    // str_len = 0
                    self.emit_indent(&format!("mov qword [rsp+{}], 0", ffi_offset + 24));
                    // bool_val
                    self.emit_indent(&format!("mov dword [rsp+{}], {}", ffi_offset + 32, if *b { 1 } else { 0 }));
                }
                Expr::Ident(var_name) => {
                    // Получаем значение переменной
                    let offset = self.get_var_offset(var_name);
                    self.emit_indent(&format!("mov rax, {}", offset));
                    // Предполагаем Int
                    self.emit_indent(&format!("mov dword [rsp+{}], 0", ffi_offset)); // tag = Int
                    self.emit_indent(&format!("mov qword [rsp+{}], rax", ffi_offset + 8)); // int_val
                    self.emit_indent(&format!("mov qword [rsp+{}], 0", ffi_offset + 16)); // str_ptr
                    self.emit_indent(&format!("mov qword [rsp+{}], 0", ffi_offset + 24)); // str_len
                    self.emit_indent(&format!("mov dword [rsp+{}], 0", ffi_offset + 32)); // bool_val
                }
                _ => {
                    // Сложное выражение — вычисляем и кладём как Int
                    self.compile_expr(arg)?;
                    self.emit_indent(&format!("mov dword [rsp+{}], 0", ffi_offset)); // tag = Int
                    self.emit_indent(&format!("mov qword [rsp+{}], rax", ffi_offset + 8)); // int_val
                    self.emit_indent(&format!("mov qword [rsp+{}], 0", ffi_offset + 16));
                    self.emit_indent(&format!("mov qword [rsp+{}], 0", ffi_offset + 24));
                    self.emit_indent(&format!("mov dword [rsp+{}], 0", ffi_offset + 32));
                }
            }
        }

        // Готовим аргументы для вызова (указатели на FfiValue)
        let arg_regs = ["rcx", "rdx", "r8", "r9"];
        for i in 0..args.len().min(4) {
            let ffi_offset = i * ffi_size;
            self.emit_indent(&format!("lea {}, [rsp+{}]", arg_regs[i], ffi_offset));
        }

        // Shadow space + вызов
        self.emit_indent("sub rsp, 32");
        self.emit_indent(&format!("call rsmm_{}", name));
        self.emit_indent("add rsp, 32");

        // Очищаем FfiValue со стека
        if total_ffi_space > 0 {
            self.emit_indent(&format!("add rsp, {}", total_ffi_space));
        }

        // Результат в rax — это FfiValue, извлекаем int_val
        // (результат уже в структуре, надо правильно обработать)
        // Для простоты берём int_val из результата
        self.emit_indent("mov rax, [rax+8]"); // int_val смещение 8

        Ok(())
    }

    fn compile_print(&mut self, expr: &Expr) -> Result<(), String> {
        match expr {
            Expr::StrLit(s) => {
                let label = self.add_string(s);
                self.emit_indent(&format!("lea rcx, [rel {}]", label));
                self.emit_indent("call __print_str");
            }
            Expr::IntLit(n) => {
                self.emit_indent(&format!("mov rcx, {}", n));
                self.emit_indent("call __print_int");
            }
            _ => {
                self.compile_expr(expr)?;
                self.emit_indent("mov rcx, rax");
                self.emit_indent("call __print_int");
            }
        }
        Ok(())
    }

    fn compile_stmt(&mut self, stmt: &Stmt, loop_start: Option<&str>, loop_end: Option<&str>) -> Result<(), String> {
        match stmt {
            Stmt::Let { name, value } => {
                self.compile_expr(value)?;
                let offset = self.get_var_offset(name);
                self.emit_indent(&format!("mov {}, rax", offset));
            }
            Stmt::Assign { name, value } => {
                self.compile_expr(value)?;
                let offset = self.get_var_offset(name);
                self.emit_indent(&format!("mov {}, rax", offset));
            }
            Stmt::Print(expr) => {
                self.compile_print(expr)?;
            }
            Stmt::If { condition, then_block, else_block } => {
                let else_label = self.new_label("else");
                let end_label = self.new_label("endif");

                self.compile_expr(condition)?;
                self.emit_indent("test rax, rax");
                self.emit_indent(&format!("je {}", else_label));

                for s in then_block {
                    self.compile_stmt(s, loop_start, loop_end)?;
                }

                if else_block.is_some() {
                    self.emit_indent(&format!("jmp {}", end_label));
                }

                self.emit(&format!("{}:", else_label));

                if let Some(else_stmts) = else_block {
                    for s in else_stmts {
                        self.compile_stmt(s, loop_start, loop_end)?;
                    }
                }

                self.emit(&format!("{}:", end_label));
            }
            Stmt::Loop { body } => {
                let start = self.new_label("loop");
                let end = self.new_label("endloop");

                self.emit(&format!("{}:", start));

                for s in body {
                    self.compile_stmt(s, Some(&start), Some(&end))?;
                }

                self.emit_indent(&format!("jmp {}", start));
                self.emit(&format!("{}:", end));
            }
            Stmt::Break => {
                if let Some(end) = loop_end {
                    self.emit_indent(&format!("jmp {}", end));
                } else {
                    return Err("break вне цикла".to_string());
                }
            }
            Stmt::Return(expr) => {
                if let Some(e) = expr {
                    self.compile_expr(e)?;
                }
                self.emit_indent("mov rsp, rbp");
                self.emit_indent("pop rbp");
                self.emit_indent("ret");
            }
            Stmt::FnDef { name, params, body } => {
                // Пропускаем если это extern_fn (они в bridge)
                if self.is_extern_fn(name) {
                    return Ok(());
                }

                let skip_label = self.new_label("skip_fn");
                self.emit_indent(&format!("jmp {}", skip_label));

                let arg_regs = ["rcx", "rdx", "r8", "r9"];
                self.functions.insert(name.clone(), params.len());

                self.emit(&format!("__fn_{}:", name));
                self.emit_indent("push rbp");
                self.emit_indent("mov rbp, rsp");
                self.emit_indent(&format!("sub rsp, {}", (params.len().max(1) + 16) * 8));

                let saved_vars = self.variables.clone();
                let saved_count = self.var_count;
                self.variables.clear();
                self.var_count = 0;

                for (i, param) in params.iter().enumerate() {
                    if i < 4 {
                        let offset = self.get_var_offset(param);
                        self.emit_indent(&format!("mov {}, {}", offset, arg_regs[i]));
                    }
                }

                for s in body {
                    self.compile_stmt(s, None, None)?;
                }

                self.emit_indent("xor rax, rax");
                self.emit_indent("mov rsp, rbp");
                self.emit_indent("pop rbp");
                self.emit_indent("ret");

                self.variables = saved_vars;
                self.var_count = saved_count;

                self.emit(&format!("{}:", skip_label));
            }
            Stmt::ExprStmt(expr) => {
                self.compile_expr(expr)?;
            }
        }
        Ok(())
    }

    fn generate_runtime(&mut self) {
        // __print_int
        self.emit("");
        self.emit("__print_int:");
        self.emit_indent("push rbp");
        self.emit_indent("mov rbp, rsp");
        self.emit_indent("sub rsp, 96");
        self.emit_indent("mov rax, rcx");
        self.emit_indent("lea rdi, [rbp-48]");
        self.emit_indent("mov byte [rdi], 10");
        self.emit_indent("dec rdi");
        self.emit_indent("xor rcx, rcx");
        self.emit_indent("xor r8, r8");
        self.emit_indent("test rax, rax");
        self.emit_indent("jnz .pi_not_zero");
        self.emit_indent("mov byte [rdi], '0'");
        self.emit_indent("dec rdi");
        self.emit_indent("mov rcx, 2");
        self.emit_indent("jmp .pi_write");
        self.emit(".pi_not_zero:");
        self.emit_indent("jns .pi_positive");
        self.emit_indent("neg rax");
        self.emit_indent("mov r8, 1");
        self.emit(".pi_positive:");
        self.emit(".pi_div_loop:");
        self.emit_indent("xor rdx, rdx");
        self.emit_indent("mov rbx, 10");
        self.emit_indent("div rbx");
        self.emit_indent("add dl, '0'");
        self.emit_indent("mov [rdi], dl");
        self.emit_indent("dec rdi");
        self.emit_indent("inc rcx");
        self.emit_indent("test rax, rax");
        self.emit_indent("jnz .pi_div_loop");
        self.emit_indent("test r8, r8");
        self.emit_indent("jz .pi_no_minus");
        self.emit_indent("mov byte [rdi], '-'");
        self.emit_indent("dec rdi");
        self.emit_indent("inc rcx");
        self.emit(".pi_no_minus:");
        self.emit_indent("inc rcx");
        self.emit(".pi_write:");
        self.emit_indent("inc rdi");
        self.emit_indent("mov [rbp-64], rdi");
        self.emit_indent("mov [rbp-72], rcx");
        self.emit_indent("sub rsp, 48");
        self.emit_indent("mov rcx, -11");
        self.emit_indent("call GetStdHandle");
        self.emit_indent("mov rcx, rax");
        self.emit_indent("mov rdx, [rbp-64]");
        self.emit_indent("mov r8, [rbp-72]");
        self.emit_indent("lea r9, [rbp-80]");
        self.emit_indent("mov qword [rsp+32], 0");
        self.emit_indent("call WriteFile");
        self.emit_indent("add rsp, 48");
        self.emit_indent("mov rsp, rbp");
        self.emit_indent("pop rbp");
        self.emit_indent("ret");

        // __print_str
        self.emit("");
        self.emit("__print_str:");
        self.emit_indent("push rbp");
        self.emit_indent("mov rbp, rsp");
        self.emit_indent("sub rsp, 96");
        self.emit_indent("mov [rbp-8], rcx");
        self.emit_indent("xor rax, rax");
        self.emit_indent("mov rdi, rcx");
        self.emit(".ps_strlen:");
        self.emit_indent("cmp byte [rdi+rax], 0");
        self.emit_indent("je .ps_got_len");
        self.emit_indent("inc rax");
        self.emit_indent("jmp .ps_strlen");
        self.emit(".ps_got_len:");
        self.emit_indent("mov [rbp-16], rax");
        self.emit_indent("sub rsp, 48");
        self.emit_indent("mov rcx, -11");
        self.emit_indent("call GetStdHandle");
        self.emit_indent("mov rcx, rax");
        self.emit_indent("mov rdx, [rbp-8]");
        self.emit_indent("mov r8, [rbp-16]");
        self.emit_indent("lea r9, [rbp-24]");
        self.emit_indent("mov qword [rsp+32], 0");
        self.emit_indent("call WriteFile");
        self.emit_indent("add rsp, 48");
        self.emit_indent("sub rsp, 48");
        self.emit_indent("mov rcx, -11");
        self.emit_indent("call GetStdHandle");
        self.emit_indent("mov rcx, rax");
        self.emit_indent("lea rdx, [rel __newline]");
        self.emit_indent("mov r8, 1");
        self.emit_indent("lea r9, [rbp-24]");
        self.emit_indent("mov qword [rsp+32], 0");
        self.emit_indent("call WriteFile");
        self.emit_indent("add rsp, 48");
        self.emit_indent("mov rsp, rbp");
        self.emit_indent("pop rbp");
        self.emit_indent("ret");
    }

    pub fn compile(&mut self, stmts: &[Stmt]) -> Result<String, String> {
        self.asm.clear();
        self.data_section.clear();
        self.bss_section.clear();

        self.data_section.push_str("    __newline db 10\n");

        // Компилируем стейтменты
        let mut body_code = String::new();
        std::mem::swap(&mut self.asm, &mut body_code);

        for stmt in stmts {
            self.compile_stmt(stmt, None, None)?;
        }

        std::mem::swap(&mut self.asm, &mut body_code);

        let stack_size = (self.var_count.max(1) + 16) * 8;
        let stack_size = (stack_size + 15) & !15;

        // Собираем результат
        let mut result = String::new();

        result.push_str("; Generated by Rust-- Hybrid Compiler\n");
        result.push_str("bits 64\n\n");

        result.push_str("extern GetStdHandle\n");
        result.push_str("extern WriteFile\n");
        result.push_str("extern ExitProcess\n");
        result.push_str("extern SetConsoleOutputCP\n");

        // Bridge функции
        for (name, _) in &self.extern_fns {
            result.push_str(&format!("extern rsmm_{}\n", name));
        }
        result.push_str("\n");

        result.push_str("section .data\n");
        result.push_str(&self.data_section);
        result.push_str("\n");

        result.push_str("section .bss\n");
        result.push_str("    __written resq 1\n");
        result.push_str("\n");

        result.push_str("section .text\n");
        result.push_str("global _start\n\n");

        result.push_str("_start:\n");
        result.push_str("    push rbp\n");
        result.push_str("    mov rbp, rsp\n");
        result.push_str(&format!("    sub rsp, {}\n", stack_size));
        result.push_str("    sub rsp, 32\n");
        result.push_str("    mov rcx, 65001\n");
        result.push_str("    call SetConsoleOutputCP\n");
        result.push_str("    add rsp, 32\n\n");

        result.push_str(&body_code);

        result.push_str("\n    sub rsp, 32\n");
        result.push_str("    xor rcx, rcx\n");
        result.push_str("    call ExitProcess\n");

        // Runtime
        self.asm.clear();
        self.generate_runtime();
        result.push_str(&self.asm);

        Ok(result)
    }
}

/// Генерирует NASM код с вызовами bridge функций
fn generate_hybrid_asm(
    stmts: &[Stmt],
    extern_fns: &[(String, Vec<String>, String)],
) -> Result<String, String> {
    let mut compiler = HybridNasmCompiler::new(extern_fns);
    compiler.compile(stmts)
}

/// Линковка OBJ + LIB
fn link_hybrid(
    obj_path: &std::path::Path,
    lib_path: &std::path::Path,
    output_path: &std::path::Path,
    additional_libs: &[String],
) -> Result<(), String> {
    // Пробуем разные линкеры

    // 1. MSVC link.exe
    if let Some(link) = find_msvc_link() {
        let mut args = vec![
            obj_path.to_string_lossy().to_string(),
            lib_path.to_string_lossy().to_string(),
            "/subsystem:console".to_string(),
            "/entry:_start".to_string(),
            format!("/out:{}", output_path.display()),
            "/nodefaultlib".to_string(),
        ];
        
        for lib in additional_libs {
            args.push(lib.clone());
        }
        
        let output = std::process::Command::new(&link)
            .args(&args)
            .output()
            .map_err(|e| format!("link.exe ошибка: {}", e))?;
        
        if output.status.success() {
            return Ok(());
        }
        
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stderr.trim().is_empty() {
            return Err(format!("MSVC link ошибка:\n{}", stderr));
        }
    }

    // 2. LLD (LLVM linker)
    {
        let mut args = vec![
            obj_path.to_string_lossy().to_string(),
            lib_path.to_string_lossy().to_string(),
            "/subsystem:console".to_string(),
            "/entry:_start".to_string(),
            format!("/out:{}", output_path.display()),
        ];
        
        for lib in additional_libs {
            args.push(lib.clone());
        }
        
        if let Ok(output) = std::process::Command::new("lld-link")
            .args(&args)
            .output()
        {
            if output.status.success() {
                return Ok(());
            }
        }
    }

    // 3. GoLink (не поддерживает staticlib, но попробуем)
    // GoLink не работает с Rust staticlib, пропускаем

    Err(
        "Не найден подходящий линкер!\n\n\
         Для гибридной сборки нужен:\n\
         • MSVC link.exe (Visual Studio Build Tools)\n\
         • или lld-link (LLVM)\n\n\
         GoLink не поддерживает линковку с Rust staticlib.".to_string()
    )
}
// ======================== LLVM КОМПИЛЯЦИЯ ========================

/// Компилятор RS-- в LLVM IR
pub struct LlvmCompiler {
    output: String,
    string_counter: usize,
    label_counter: usize,
    var_counter: usize,
    variables: HashMap<String, String>,
    functions: HashMap<String, usize>,
    extern_fns: HashMap<String, usize>,
    strings: Vec<(String, String)>,
}

impl LlvmCompiler {
    pub fn new() -> Self {
        LlvmCompiler {
            output: String::new(),
            string_counter: 0,
            label_counter: 0,
            var_counter: 0,
            variables: HashMap::new(),
            functions: HashMap::new(),
            extern_fns: HashMap::new(),
            strings: Vec::new(),
        }
    }

    fn new_reg(&mut self) -> String {
        self.var_counter += 1;
        format!("%t{}", self.var_counter)
    }

    fn new_label(&mut self, prefix: &str) -> String {
        self.label_counter += 1;
        format!("{}.{}", prefix, self.label_counter)
    }

    fn add_string(&mut self, s: &str) -> String {
        self.string_counter += 1;
        let label = format!("@.str.{}", self.string_counter);
        self.strings.push((label.clone(), s.to_string()));
        label
    }

    fn emit(&mut self, line: &str) {
        self.output.push_str(line);
        self.output.push('\n');
    }

    fn emit_indent(&mut self, line: &str) {
        self.output.push_str("  ");
        self.output.push_str(line);
        self.output.push('\n');
    }

    fn compile_expr(&mut self, expr: &Expr) -> Result<String, String> {
        match expr {
            Expr::IntLit(n) => Ok(format!("{}", n)),
            
            Expr::BoolLit(b) => Ok(if *b { "1".to_string() } else { "0".to_string() }),
            
            Expr::StrLit(s) => {
                let label = self.add_string(s);
                Ok(format!("STR:{}", label))
            }
            
            Expr::Ident(name) => {
                if let Some(var_reg) = self.variables.get(name).cloned() {
                    let reg = self.new_reg();
                    self.emit_indent(&format!("{} = load i64, i64* {}", reg, var_reg));
                    Ok(reg)
                } else {
                    Err(format!("Переменная '{}' не определена", name))
                }
            }
            
            Expr::BinOp { op, left, right } => {
                let l = self.compile_expr(left)?;
                let r = self.compile_expr(right)?;
                
                let l_is_str = l.starts_with("STR:");
                let r_is_str = r.starts_with("STR:");
                
                if matches!(op, BinOp::Add) && (l_is_str || r_is_str) {
                    return Ok("0".to_string());
                }
                
                let reg = self.new_reg();

                match op {
                    BinOp::Add => self.emit_indent(&format!("{} = add i64 {}, {}", reg, l, r)),
                    BinOp::Sub => self.emit_indent(&format!("{} = sub i64 {}, {}", reg, l, r)),
                    BinOp::Mul => self.emit_indent(&format!("{} = mul i64 {}, {}", reg, l, r)),
                    BinOp::Div => self.emit_indent(&format!("{} = sdiv i64 {}, {}", reg, l, r)),
                    BinOp::Mod => self.emit_indent(&format!("{} = srem i64 {}, {}", reg, l, r)),
                    BinOp::Eq => {
                        let cmp = self.new_reg();
                        self.emit_indent(&format!("{} = icmp eq i64 {}, {}", cmp, l, r));
                        self.emit_indent(&format!("{} = zext i1 {} to i64", reg, cmp));
                    }
                    BinOp::Neq => {
                        let cmp = self.new_reg();
                        self.emit_indent(&format!("{} = icmp ne i64 {}, {}", cmp, l, r));
                        self.emit_indent(&format!("{} = zext i1 {} to i64", reg, cmp));
                    }
                    BinOp::Lt => {
                        let cmp = self.new_reg();
                        self.emit_indent(&format!("{} = icmp slt i64 {}, {}", cmp, l, r));
                        self.emit_indent(&format!("{} = zext i1 {} to i64", reg, cmp));
                    }
                    BinOp::Gt => {
                        let cmp = self.new_reg();
                        self.emit_indent(&format!("{} = icmp sgt i64 {}, {}", cmp, l, r));
                        self.emit_indent(&format!("{} = zext i1 {} to i64", reg, cmp));
                    }
                    BinOp::Lte => {
                        let cmp = self.new_reg();
                        self.emit_indent(&format!("{} = icmp sle i64 {}, {}", cmp, l, r));
                        self.emit_indent(&format!("{} = zext i1 {} to i64", reg, cmp));
                    }
                    BinOp::Gte => {
                        let cmp = self.new_reg();
                        self.emit_indent(&format!("{} = icmp sge i64 {}, {}", cmp, l, r));
                        self.emit_indent(&format!("{} = zext i1 {} to i64", reg, cmp));
                    }
                    BinOp::And => {
                        let a = self.new_reg();
                        let b = self.new_reg();
                        let c = self.new_reg();
                        self.emit_indent(&format!("{} = icmp ne i64 {}, 0", a, l));
                        self.emit_indent(&format!("{} = icmp ne i64 {}, 0", b, r));
                        self.emit_indent(&format!("{} = and i1 {}, {}", c, a, b));
                        self.emit_indent(&format!("{} = zext i1 {} to i64", reg, c));
                    }
                    BinOp::Or => {
                        let a = self.new_reg();
                        let b = self.new_reg();
                        let c = self.new_reg();
                        self.emit_indent(&format!("{} = icmp ne i64 {}, 0", a, l));
                        self.emit_indent(&format!("{} = icmp ne i64 {}, 0", b, r));
                        self.emit_indent(&format!("{} = or i1 {}, {}", c, a, b));
                        self.emit_indent(&format!("{} = zext i1 {} to i64", reg, c));
                    }
                }
                Ok(reg)
            }
            
            Expr::UnaryOp { op, expr } => {
                let e = self.compile_expr(expr)?;
                let reg = self.new_reg();
                match op {
                    UnaryOp::Neg => self.emit_indent(&format!("{} = sub i64 0, {}", reg, e)),
                    UnaryOp::Not => {
                        let cmp = self.new_reg();
                        self.emit_indent(&format!("{} = icmp eq i64 {}, 0", cmp, e));
                        self.emit_indent(&format!("{} = zext i1 {} to i64", reg, cmp));
                    }
                }
                Ok(reg)
            }
            
            Expr::Call { name, args } => {
                if name == "print" && args.len() == 1 {
                    return self.compile_print(&args[0]);
                }

                let mut arg_strs = Vec::new();
                for arg in args {
                    let a = self.compile_expr(arg)?;
                    if !a.starts_with("STR:") {
                        arg_strs.push(format!("i64 {}", a));
                    } else {
                        arg_strs.push("i64 0".to_string());
                    }
                }

                let reg = self.new_reg();
                if self.extern_fns.contains_key(name) {
                    self.emit_indent(&format!(
                        "{} = call i64 @rsmm_{}({})",
                        reg, name, arg_strs.join(", ")
                    ));
                } else {
                    self.emit_indent(&format!(
                        "{} = call i64 @__fn_{}({})",
                        reg, name, arg_strs.join(", ")
                    ));
                }
                Ok(reg)
            }
        }
    }

    fn compile_print(&mut self, expr: &Expr) -> Result<String, String> {
        match expr {
            Expr::StrLit(s) => {
                let label = self.add_string(s);
                let len = s.len() + 1;
                let ptr = self.new_reg();
                self.emit_indent(&format!(
                    "{} = getelementptr [{} x i8], [{} x i8]* {}, i64 0, i64 0",
                    ptr, len, len, label
                ));
                self.emit_indent(&format!("call void @__print_str(i8* {})", ptr));
            }
            Expr::IntLit(n) => {
                self.emit_indent(&format!("call void @__print_int(i64 {})", n));
            }
            Expr::BoolLit(b) => {
                self.emit_indent(&format!("call void @__print_int(i64 {})", if *b { 1 } else { 0 }));
            }
            Expr::Ident(name) => {
                if let Some(var_reg) = self.variables.get(name).cloned() {
                    let reg = self.new_reg();
                    self.emit_indent(&format!("{} = load i64, i64* {}", reg, var_reg));
                    self.emit_indent(&format!("call void @__print_int(i64 {})", reg));
                } else {
                    return Err(format!("Переменная '{}' не определена", name));
                }
            }
            Expr::BinOp { op: BinOp::Add, left, right } => {
                self.compile_print_part(left)?;
                self.compile_print_part(right)?;
                self.emit_indent("call void @__print_newline()");
            }
            _ => {
                let val = self.compile_expr(expr)?;
                if !val.starts_with("STR:") {
                    self.emit_indent(&format!("call void @__print_int(i64 {})", val));
                }
            }
        }
        Ok("0".to_string())
    }

    fn compile_print_part(&mut self, expr: &Expr) -> Result<(), String> {
        match expr {
            Expr::StrLit(s) => {
                let label = self.add_string(s);
                let len = s.len() + 1;
                let ptr = self.new_reg();
                self.emit_indent(&format!(
                    "{} = getelementptr [{} x i8], [{} x i8]* {}, i64 0, i64 0",
                    ptr, len, len, label
                ));
                self.emit_indent(&format!("call void @__print_str_no_nl(i8* {})", ptr));
            }
            Expr::IntLit(n) => {
                self.emit_indent(&format!("call void @__print_int_no_nl(i64 {})", n));
            }
            Expr::Ident(name) => {
                if let Some(var_reg) = self.variables.get(name).cloned() {
                    let reg = self.new_reg();
                    self.emit_indent(&format!("{} = load i64, i64* {}", reg, var_reg));
                    self.emit_indent(&format!("call void @__print_int_no_nl(i64 {})", reg));
                }
            }
            Expr::BinOp { op: BinOp::Add, left, right } => {
                self.compile_print_part(left)?;
                self.compile_print_part(right)?;
            }
            _ => {
                let val = self.compile_expr(expr)?;
                if !val.starts_with("STR:") {
                    self.emit_indent(&format!("call void @__print_int_no_nl(i64 {})", val));
                }
            }
        }
        Ok(())
    }

    fn compile_stmt(&mut self, stmt: &Stmt, loop_start: Option<&str>, loop_end: Option<&str>) -> Result<(), String> {
        match stmt {
            Stmt::Let { name, value } => {
                let val = self.compile_expr(value)?;
                let var_reg = format!("%var.{}", name);
                if !self.variables.contains_key(name) {
                    self.emit_indent(&format!("{} = alloca i64", var_reg));
                    self.variables.insert(name.clone(), var_reg.clone());
                }
                if !val.starts_with("STR:") {
                    self.emit_indent(&format!("store i64 {}, i64* {}", val, var_reg));
                } else {
                    self.emit_indent(&format!("store i64 0, i64* {}", var_reg));
                }
            }
            Stmt::Assign { name, value } => {
                let val = self.compile_expr(value)?;
                if let Some(var_reg) = self.variables.get(name).cloned() {
                    if !val.starts_with("STR:") {
                        self.emit_indent(&format!("store i64 {}, i64* {}", val, var_reg));
                    }
                } else {
                    return Err(format!("Переменная '{}' не определена", name));
                }
            }
            Stmt::Print(expr) => {
                self.compile_print(expr)?;
            }
            Stmt::If { condition, then_block, else_block } => {
                let cond = self.compile_expr(condition)?;
                let cond_bool = self.new_reg();
                self.emit_indent(&format!("{} = icmp ne i64 {}, 0", cond_bool, cond));

                let then_label = self.new_label("then");
                let else_label = self.new_label("else");
                let end_label = self.new_label("endif");

                if else_block.is_some() {
                    self.emit_indent(&format!("br i1 {}, label %{}, label %{}", cond_bool, then_label, else_label));
                } else {
                    self.emit_indent(&format!("br i1 {}, label %{}, label %{}", cond_bool, then_label, end_label));
                }

                self.emit(&format!("{}:", then_label));
                for s in then_block {
                    self.compile_stmt(s, loop_start, loop_end)?;
                }
                self.emit_indent(&format!("br label %{}", end_label));

                if let Some(else_stmts) = else_block {
                    self.emit(&format!("{}:", else_label));
                    for s in else_stmts {
                        self.compile_stmt(s, loop_start, loop_end)?;
                    }
                    self.emit_indent(&format!("br label %{}", end_label));
                }

                self.emit(&format!("{}:", end_label));
            }
            Stmt::Loop { body } => {
                let start_label = self.new_label("loop");
                let end_label = self.new_label("endloop");

                self.emit_indent(&format!("br label %{}", start_label));
                self.emit(&format!("{}:", start_label));

                for s in body {
                    self.compile_stmt(s, Some(&start_label), Some(&end_label))?;
                }

                self.emit_indent(&format!("br label %{}", start_label));
                self.emit(&format!("{}:", end_label));
            }
            Stmt::Break => {
                if let Some(end) = loop_end {
                    self.emit_indent(&format!("br label %{}", end));
                    let unreachable_label = self.new_label("unreachable");
                    self.emit(&format!("{}:", unreachable_label));
                } else {
                    return Err("break вне цикла".to_string());
                }
            }
            Stmt::Return(expr) => {
                if let Some(e) = expr {
                    let val = self.compile_expr(e)?;
                    if !val.starts_with("STR:") {
                        self.emit_indent(&format!("ret i64 {}", val));
                    } else {
                        self.emit_indent("ret i64 0");
                    }
                } else {
                    self.emit_indent("ret i64 0");
                }
            }
            Stmt::FnDef { name, params, .. } => {
                self.functions.insert(name.clone(), params.len());
            }
            Stmt::ExprStmt(expr) => {
                self.compile_expr(expr)?;
            }
        }
        Ok(())
    }

    fn compile_function(&mut self, name: &str, params: &[String], body: &[Stmt]) -> Result<String, String> {
        let mut func_output = String::new();
        let params_str = params.iter().map(|p| format!("i64 %param.{}", p)).collect::<Vec<_>>().join(", ");

        func_output.push_str(&format!("define i64 @__fn_{}({}) {{\n", name, params_str));
        func_output.push_str("entry:\n");

        let saved_output = std::mem::take(&mut self.output);
        let saved_vars = std::mem::take(&mut self.variables);
        let saved_var_counter = self.var_counter;

        for param in params {
            let var_reg = format!("%var.{}", param);
            self.emit_indent(&format!("{} = alloca i64", var_reg));
            self.emit_indent(&format!("store i64 %param.{}, i64* {}", param, var_reg));
            self.variables.insert(param.clone(), var_reg);
        }

        for stmt in body {
            self.compile_stmt(stmt, None, None)?;
        }

        if !self.output.contains("ret i64") {
            self.emit_indent("ret i64 0");
        }

        func_output.push_str(&self.output);
        func_output.push_str("}\n\n");

        self.output = saved_output;
        self.variables = saved_vars;
        self.var_counter = saved_var_counter;

        Ok(func_output)
    }

    pub fn compile(&mut self, stmts: &[Stmt], extern_fns: &[(String, Vec<String>, String)]) -> Result<String, String> {
        for (name, params, _) in extern_fns {
            self.extern_fns.insert(name.clone(), params.len());
        }

        let mut functions_code = String::new();
        let mut main_stmts = Vec::new();

        for stmt in stmts {
            if let Stmt::FnDef { name, params, body } = stmt {
                let func_code = self.compile_function(name, params, body)?;
                functions_code.push_str(&func_code);
            } else {
                main_stmts.push(stmt.clone());
            }
        }

        self.emit("entry:");
        for stmt in &main_stmts {
            self.compile_stmt(stmt, None, None)?;
        }
        self.emit_indent("ret i64 0");

        let main_body = std::mem::take(&mut self.output);

        let mut result = String::new();
        result.push_str("; Generated by Rust-- LLVM Compiler\n");
        result.push_str("target triple = \"x86_64-pc-windows-msvc\"\n\n");

        for (label, content) in &self.strings {
            let escaped = content.replace('\\', "\\5C").replace('\n', "\\0A").replace('"', "\\22");
            let len = content.len() + 1;
            result.push_str(&format!("{} = private constant [{} x i8] c\"{}\\00\"\n", label, len, escaped));
        }
        result.push('\n');

        result.push_str("declare void @__print_int(i64)\n");
        result.push_str("declare void @__print_str(i8*)\n");
        result.push_str("declare void @__print_int_no_nl(i64)\n");
        result.push_str("declare void @__print_str_no_nl(i8*)\n");
        result.push_str("declare void @__print_newline()\n");

        for (name, param_count) in &self.extern_fns {
            let params = (0..*param_count).map(|_| "i64").collect::<Vec<_>>().join(", ");
            result.push_str(&format!("declare i64 @rsmm_{}({})\n", name, params));
        }
        result.push('\n');

        result.push_str(&functions_code);
        result.push_str("define i64 @main() {\n");
        result.push_str(&main_body);
        result.push_str("}\n");

        Ok(result)
    }
}

// ======================== СБОРКА ЧЕРЕЗ LLVM ========================

pub fn build_llvm_exe(source: &str, output_path: &std::path::Path) -> Result<String, String> {
    let mut ext_parser = ExtParser::new(source);
    let ext_stmts = ext_parser.parse_extended().map_err(|e| e.to_string())?;

    let mut dependencies: Vec<RustDependency> = Vec::new();
    let mut rust_blocks: Vec<String> = Vec::new();
    let mut extern_fns: Vec<(String, Vec<String>, String)> = Vec::new();
    let mut normal_stmts: Vec<Stmt> = Vec::new();

    for stmt in &ext_stmts {
        match stmt {
            ExtStmt::UseCrate { name, version, features } => {
                let ver = version.clone().unwrap_or_else(|| "*".to_string());
                if features.is_empty() {
                    dependencies.push(RustDependency::new(name, &ver));
                } else {
                    let feats: Vec<&str> = features.iter().map(|s| s.as_str()).collect();
                    dependencies.push(RustDependency::with_features(name, &ver, feats));
                }
            }
            ExtStmt::RustBlock(code) => rust_blocks.push(code.clone()),
            ExtStmt::ExternFn { name, params, rust_body } => {
                extern_fns.push((name.clone(), params.clone(), rust_body.clone()));
            }
            ExtStmt::Normal(s) => normal_stmts.push(s.clone()),
            ExtStmt::Import(_) => {}
        }
    }

    let temp_dir = std::env::temp_dir().join("rsmm_llvm_build");
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&temp_dir).map_err(|e| format!("Не могу создать папку: {}", e))?;

    // Генерируем LLVM IR
    let mut compiler = LlvmCompiler::new();
    let llvm_ir = compiler.compile(&normal_stmts, &extern_fns)?;

    let ir_path = temp_dir.join("main.ll");
    std::fs::write(&ir_path, &llvm_ir).map_err(|e| format!("Не могу записать IR: {}", e))?;

    // Сохраняем копию для отладки
    let debug_ir = output_path.with_extension("ll");
    let _ = std::fs::write(&debug_ir, &llvm_ir);

    // Собираем runtime
    let runtime_lib = match build_llvm_runtime(&temp_dir, &dependencies, &rust_blocks, &extern_fns) {
        Ok(lib) => lib,
        Err(e) => {
            return Err(format!("Ошибка сборки runtime:\n{}\n\nIR сохранён: {}", e, debug_ir.display()));
        }
    };

    // Находим clang
    let clang = find_clang().ok_or_else(|| {
        "Clang не найден!\n\n\
         Установите LLVM: https://releases.llvm.org/\n\
         Скачайте LLVM-XX.X.X-win64.exe\n\
         После установки перезапустите IDE.".to_string()
    })?;

    // Компилируем IR в OBJ
    let obj_path = temp_dir.join("main.obj");
    
    let clang_output = std::process::Command::new(&clang)
        .arg("-c")
        .arg("-O2")
        .arg("-o").arg(&obj_path)
        .arg(&ir_path)
        .output()
        .map_err(|e| format!("Не могу запустить Clang: {}\nПуть: {}", e, clang))?;

    if !clang_output.status.success() {
        let stderr = String::from_utf8_lossy(&clang_output.stderr);
        let stdout = String::from_utf8_lossy(&clang_output.stdout);
        return Err(format!(
            "Clang ошибка:\n{}\n{}\n\nIR файл: {}",
            stderr, stdout, debug_ir.display()
        ));
    }

    // Проверяем что OBJ создан
    if !obj_path.exists() {
        return Err(format!("Clang не создал OBJ файл: {}", obj_path.display()));
    }

    // Линкуем
    if let Err(e) = link_llvm_exe(&obj_path, &runtime_lib, output_path) {
        // Не удаляем temp для отладки
        return Err(format!(
            "{}\n\nФайлы для отладки:\n  IR: {}\n  OBJ: {}\n  LIB: {}",
            e, debug_ir.display(), obj_path.display(), runtime_lib.display()
        ));
    }

    let size = std::fs::metadata(output_path).map(|m| m.len()).unwrap_or(0);
    
    // Очищаем temp только при успехе
    let _ = std::fs::remove_dir_all(&temp_dir);

    Ok(format!(
        "✅ LLVM EXE собран!\n\
         Путь: {}\n\
         Размер: {:.1} KB\n\
         IR: {}",
        output_path.display(),
        size as f64 / 1024.0,
        debug_ir.display()
    ))
}
fn build_llvm_runtime(
    temp_dir: &std::path::Path,
    dependencies: &[RustDependency],
    rust_blocks: &[String],
    extern_fns: &[(String, Vec<String>, String)],
) -> Result<std::path::PathBuf, String> {
    let runtime_dir = temp_dir.join("runtime");
    let src_dir = runtime_dir.join("src");
    std::fs::create_dir_all(&src_dir).map_err(|e| format!("Runtime dir: {}", e))?;

    let mut cargo_toml = r#"[package]
name = "rsmm_runtime"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["staticlib"]

[dependencies]
"#.to_string();

    for dep in dependencies {
        cargo_toml.push_str(&dep.to_toml_line());
        cargo_toml.push('\n');
    }

    cargo_toml.push_str("\n[profile.release]\nopt-level = \"z\"\nlto = true\n");
    std::fs::write(runtime_dir.join("Cargo.toml"), &cargo_toml).map_err(|e| format!("Cargo.toml: {}", e))?;

    let mut lib_code = String::from("#![allow(unused)]\n\n");
    for block in rust_blocks {
        lib_code.push_str(block);
        lib_code.push('\n');
    }

	lib_code.push_str(r#"
use std::io::Write;

#[no_mangle]
pub extern "C" fn __print_int(n: i64) { 
    println!("{}", n); 
}

#[no_mangle]
pub extern "C" fn __print_str(s: *const i8) {
    if !s.is_null() {
        unsafe { println!("{}", std::ffi::CStr::from_ptr(s).to_string_lossy()); }
    } else {
        println!();
    }
}

#[no_mangle]
pub extern "C" fn __print_int_no_nl(n: i64) { 
    print!("{}", n);
    let _ = std::io::stdout().flush();
}

#[no_mangle]
pub extern "C" fn __print_str_no_nl(s: *const i8) {
    if !s.is_null() {
        unsafe { print!("{}", std::ffi::CStr::from_ptr(s).to_string_lossy()); }
        let _ = std::io::stdout().flush();
    }
}

#[no_mangle]
pub extern "C" fn __print_newline() {
    println!();
}

#[derive(Clone)]
pub enum RsmmVal { Int(i64), Str(String), Bool(bool), None }
impl RsmmVal {
    pub fn as_int(&self) -> i64 { match self { RsmmVal::Int(n) => *n, _ => 0 } }
    pub fn as_str(&self) -> String { match self { RsmmVal::Str(s) => s.clone(), RsmmVal::Int(n) => n.to_string(), _ => String::new() } }
    pub fn as_bool(&self) -> bool { match self { RsmmVal::Bool(b) => *b, RsmmVal::Int(n) => *n != 0, _ => false } }
}
"#);

    for (name, params, body) in extern_fns {
        let params_inner = params.iter().map(|p| format!("{}: RsmmVal", p)).collect::<Vec<_>>().join(", ");
        lib_code.push_str(&format!("fn {}_impl({}) -> RsmmVal {{ {} }}\n", name, params_inner, body));

        let params_extern = params.iter().enumerate().map(|(i, _)| format!("arg{}: i64", i)).collect::<Vec<_>>().join(", ");
        lib_code.push_str(&format!("#[no_mangle]\npub extern \"C\" fn rsmm_{}({}) -> i64 {{\n", name, params_extern));
        for (i, param) in params.iter().enumerate() {
            lib_code.push_str(&format!("    let {} = RsmmVal::Int(arg{});\n", param, i));
        }
        lib_code.push_str(&format!("    {}_impl({}).as_int()\n}}\n", name, params.join(", ")));
    }

    std::fs::write(src_dir.join("lib.rs"), &lib_code).map_err(|e| format!("lib.rs: {}", e))?;

    let output = std::process::Command::new("cargo")
        .arg("build").arg("--release")
        .arg("--manifest-path").arg(runtime_dir.join("Cargo.toml"))
        .output()
        .map_err(|e| format!("Cargo: {}", e))?;

    if !output.status.success() {
        return Err(format!("Runtime build:\n{}", String::from_utf8_lossy(&output.stderr)));
    }

    let lib_path = runtime_dir.join("target/release/rsmm_runtime.lib");
    if lib_path.exists() { Ok(lib_path) }
    else {
        let alt = runtime_dir.join("target/release/librsmm_runtime.a");
        if alt.exists() { Ok(alt) } else { Err("Runtime lib not found".into()) }
    }
}

fn find_clang() -> Option<String> {
    if std::process::Command::new("clang").arg("--version").output().is_ok() {
        return Some("clang".into());
    }
    for path in &["C:\\Program Files\\LLVM\\bin\\clang.exe", "C:\\Program Files (x86)\\LLVM\\bin\\clang.exe"] {
        if std::path::Path::new(path).exists() { return Some(path.to_string()); }
    }
    None
}

fn link_llvm_exe(obj: &std::path::Path, lib: &std::path::Path, out: &std::path::Path) -> Result<(), String> {
    if !obj.exists() {
        return Err(format!("OBJ файл не найден: {}", obj.display()));
    }
    if !lib.exists() {
        return Err(format!("Runtime библиотека не найдена: {}", lib.display()));
    }

	let args = vec![
		obj.to_string_lossy().to_string(),
		lib.to_string_lossy().to_string(),
		"/subsystem:console".into(),
		"/entry:main".into(),
		format!("/out:{}", out.display()),
		"/errorlimit:0".into(),
		"/nodefaultlib:libcmt".into(),
		// C Runtime (для memcpy, memset, sin, cos и т.д.)
		"msvcrt.lib".into(),
		"vcruntime.lib".into(),
		"ucrt.lib".into(),
		"libcmt.lib".into(),
		"libvcruntime.lib".into(),
		"libucrt.lib".into(),
		"legacy_stdio_definitions.lib".into(),
		// Базовые Windows
		"kernel32.lib".into(),
		"advapi32.lib".into(),
		"bcrypt.lib".into(),
		"ntdll.lib".into(),
		"userenv.lib".into(),
		"ws2_32.lib".into(),
		// Windows GUI
		"user32.lib".into(),
		"gdi32.lib".into(),
		"shell32.lib".into(),
		"ole32.lib".into(),
		"oleaut32.lib".into(),
		"uuid.lib".into(),
		"comdlg32.lib".into(),
		"dwmapi.lib".into(),
		"opengl32.lib".into(),
		"imm32.lib".into(),
		"winspool.lib".into(),
		"setupapi.lib".into(),
		"cfgmgr32.lib".into(),
		"credui.lib".into(),
		"crypt32.lib".into(),
		"secur32.lib".into(),
		"propsys.lib".into(),
		"runtimeobject.lib".into(),
		// DirectX
		"d3d11.lib".into(),
		"dxgi.lib".into(),
		"d3dcompiler.lib".into(),
		// Сетевые
		"winhttp.lib".into(),
		"winmm.lib".into(),
		// Accessibility (UIA)
		"uiautomationcore.lib".into(),
		"oleacc.lib".into(),
		// Дополнительные
		"comctl32.lib".into(),
		"shlwapi.lib".into(),
		"version.lib".into(),
		"synchronization.lib".into(),
	];

    // 1. Пробуем lld-link
    match std::process::Command::new("lld-link").args(&args).output() {
        Ok(o) => {
            if o.status.success() {
                return Ok(());
            }
            let stderr = String::from_utf8_lossy(&o.stderr);
            let stdout = String::from_utf8_lossy(&o.stdout);
            if !stderr.is_empty() || !stdout.is_empty() {
                return Err(format!(
                    "lld-link ошибка:\n{}\n{}",
                    stderr, stdout
                ));
            }
        }
        Err(_) => {}
    }

    // 2. Пробуем MSVC link.exe
    if let Some(link) = find_msvc_link() {
        match std::process::Command::new(&link).args(&args).output() {
            Ok(o) => {
                if o.status.success() {
                    return Ok(());
                }
                let stderr = String::from_utf8_lossy(&o.stderr);
                let stdout = String::from_utf8_lossy(&o.stdout);
                return Err(format!(
                    "MSVC link.exe ошибка:\n{}\n{}",
                    stderr, stdout
                ));
            }
            Err(e) => {
                return Err(format!("Не могу запустить link.exe: {}", e));
            }
        }
    }

    Err(
        "Линкер не найден!\n\n\
         Установите LLVM: https://releases.llvm.org/\n\
         Или Visual Studio Build Tools".to_string()
    )
}

/// Проект через LLVM
pub fn build_project_llvm(project: &Project, output_path: &std::path::Path) -> Result<String, String> {
    let mut full_source = String::new();
    for file in &project.files {
        if !file.is_main {
            full_source.push_str(&file.content);
            full_source.push_str("\n\n");
        }
    }
    for file in &project.files {
        if file.is_main {
            full_source.push_str(&file.content);
        }
    }
    build_llvm_exe(&full_source, output_path)
}
