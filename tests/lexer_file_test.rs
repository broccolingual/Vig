use std::fs;
use vig::lexer::{Lexer, TokenKind};

fn lex_file(path: &str) -> Vec<(TokenKind, String)> {
    let source = fs::read_to_string(path).unwrap_or_else(|_| panic!("failed to read {}", path));
    let lexer = Lexer::new(&source);
    lexer
        .map(|r| {
            let token = r.expect("unexpected lex error");
            (token.kind, token.text)
        })
        .collect()
}

#[test]
fn test_counter_vhd_tokens_not_empty() {
    let tokens = lex_file("testdata/counter.vhd");
    assert!(!tokens.is_empty(), "トークン列が空です");
}

#[test]
fn test_counter_vhd_starts_with_comment() {
    let tokens = lex_file("testdata/counter.vhd");
    assert_eq!(tokens[0].0, TokenKind::Comment);
}

#[test]
fn test_counter_vhd_entity_structure() {
    let tokens = lex_file("testdata/counter.vhd");
    let kinds: Vec<&TokenKind> = tokens.iter().map(|(k, _)| k).collect();

    // library ieee;
    assert_eq!(kinds[1], &TokenKind::Library);
    assert_eq!(kinds[3], &TokenKind::Semicolon);

    // use ieee.std_logic_1164.all;
    assert_eq!(kinds[4], &TokenKind::Use);

    // entity counter is
    let entity_pos = kinds.iter().position(|k| **k == TokenKind::Entity).unwrap();
    assert_eq!(kinds[entity_pos + 1], &TokenKind::Identifier); // counter
    assert_eq!(kinds[entity_pos + 2], &TokenKind::Is);
}

#[test]
fn test_counter_vhd_port_declarations() {
    let tokens = lex_file("testdata/counter.vhd");
    let kinds: Vec<&TokenKind> = tokens.iter().map(|(k, _)| k).collect();

    // port キーワードが存在する
    assert!(kinds.contains(&&TokenKind::Port));

    // in, out キーワードが存在する
    assert!(kinds.contains(&&TokenKind::In));
    assert!(kinds.contains(&&TokenKind::Out));

    // std_logic, std_logic_vector 型が存在する
    assert!(kinds.contains(&&TokenKind::StdLogic));
    assert!(kinds.contains(&&TokenKind::StdLogicVector));
}

#[test]
fn test_counter_vhd_architecture() {
    let tokens = lex_file("testdata/counter.vhd");
    let kinds: Vec<&TokenKind> = tokens.iter().map(|(k, _)| k).collect();

    // architecture キーワードが存在する
    assert!(kinds.contains(&&TokenKind::Architecture));

    // signal, process, begin キーワードが存在する
    assert!(kinds.contains(&&TokenKind::Signal));
    assert!(kinds.contains(&&TokenKind::Process));
    assert!(kinds.contains(&&TokenKind::Begin));
}

#[test]
fn test_counter_vhd_control_flow() {
    let tokens = lex_file("testdata/counter.vhd");
    let kinds: Vec<&TokenKind> = tokens.iter().map(|(k, _)| k).collect();

    // if/then/elsif/end if 構文が存在する
    assert!(kinds.contains(&&TokenKind::If));
    assert!(kinds.contains(&&TokenKind::Then));
    assert!(kinds.contains(&&TokenKind::Elsif));
}

#[test]
fn test_counter_vhd_operators() {
    let tokens = lex_file("testdata/counter.vhd");
    let kinds: Vec<&TokenKind> = tokens.iter().map(|(k, _)| k).collect();

    // <= (信号代入), + (加算), = (比較) が存在する
    assert!(kinds.contains(&&TokenKind::Lte));
    assert!(kinds.contains(&&TokenKind::Plus));
    assert!(kinds.contains(&&TokenKind::Eq));
}

#[test]
fn test_counter_vhd_character_literal() {
    let tokens = lex_file("testdata/counter.vhd");

    // '1' のキャラクタリテラルが存在する
    let char_literals: Vec<&String> = tokens
        .iter()
        .filter(|(k, _)| *k == TokenKind::CharacterLiteral)
        .map(|(_, t)| t)
        .collect();

    assert!(!char_literals.is_empty());
    assert!(char_literals.contains(&&"'1'".to_string()));
}

#[test]
fn test_counter_vhd_no_errors() {
    let source = fs::read_to_string("testdata/counter.vhd").unwrap();
    let lexer = Lexer::new(&source);

    for result in lexer {
        assert!(result.is_ok(), "lexエラー: {:?}", result.err());
    }
}

#[test]
fn test_counter_vhd_downto() {
    let tokens = lex_file("testdata/counter.vhd");
    let kinds: Vec<&TokenKind> = tokens.iter().map(|(k, _)| k).collect();

    // std_logic_vector(7 downto 0) の downto が存在する
    assert!(kinds.contains(&&TokenKind::Downto));
}
