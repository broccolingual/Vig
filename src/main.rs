// VHDLのlexer・意味解析の使用例

use vig::analyzer;
use vig::generator;
use vig::lexer::{Lexer, TokenKind};

use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("使い方: {} [-d] <VHDLファイル>", args[0]);
        eprintln!("  -d: デバッグモード（構文解析と意味解析の結果を表示）");
        process::exit(1);
    }

    // フラグと引数を解析
    let mut debug_mode = false;
    let mut filename = None;

    for arg in &args[1..] {
        if arg == "-d" {
            debug_mode = true;
        } else {
            filename = Some(arg);
        }
    }

    let filename = match filename {
        Some(f) => f,
        None => {
            eprintln!("エラー: VHDLファイルが指定されていません");
            eprintln!("使い方: {} [-d] <VHDLファイル>", args[0]);
            process::exit(1);
        }
    };

    let vhdl_code = match fs::read_to_string(filename) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("ファイル '{}' の読み込みに失敗しました: {}", filename, err);
            process::exit(1);
        }
    };

    // デバッグモード: トークン解析結果を表示
    if debug_mode {
        eprintln!("=== {} のトークン解析 ===\n", filename);

        let lexer = Lexer::new(&vhdl_code);

        for (index, result) in lexer.enumerate() {
            match result {
                Ok(token) => {
                    if token.kind == TokenKind::Comment {
                        eprintln!(
                            "{:3}: {:20} ({}..{})",
                            index,
                            format!("{:?}", token.kind),
                            token.span.start,
                            token.span.end
                        );
                    } else {
                        eprintln!(
                            "{:3}: {:20} '{}' ({}..{})",
                            index,
                            format!("{:?}", token.kind),
                            token.text,
                            token.span.start,
                            token.span.end
                        );
                    }
                }
                Err(err) => {
                    eprintln!("エラー: {}", err);
                }
            }
        }
    }

    // 意味解析
    let result = match analyzer::analyze_vhdl(&vhdl_code) {
        Ok(result) => {
            if debug_mode {
                eprintln!("\n=== {} の意味解析 ===\n", filename);
                eprint!("{}", result);
            }
            result
        }
        Err(err) => {
            eprintln!("解析エラー: {}", err);
            process::exit(1);
        }
    };

    // テストベンチ生成
    let config = generator::TbConfig::default();
    for entity in &result.entities {
        if debug_mode {
            eprintln!("\n=== {} のテストベンチ ===\n", entity.name);
        }
        let tb = generator::generate_testbench(entity, &config);
        print!("{}", tb);
    }
}
