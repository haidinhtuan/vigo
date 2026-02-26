//! REPL (Read-Eval-Print Loop) mode for testing Vietnamese input.
//!
//! This module provides an interactive terminal interface for testing
//! the Vietnamese input engine.

use crate::{Engine, InputMethod};
use std::io::{self, BufRead, Write};

/// Runs the interactive REPL.
///
/// # Arguments
/// * `method` - The input method to use (Telex or VNI)
///
/// # Controls
/// - Type characters to see real-time transformation
/// - Press Enter to commit and start a new word
/// - Type "quit" or "exit" to exit
/// - Type "telex" or "vni" to switch input methods
pub fn run(method: InputMethod) {
    let mut engine = Engine::new(method);
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    println!("=== Vigo Vietnamese Input Engine ===");
    println!("Input method: {:?}", method);
    println!();
    println!("Commands:");
    println!("  Type characters to see transformation");
    println!("  Enter     - commit and start new word");
    println!("  telex/vni - switch input method");
    println!("  quit/exit - exit");
    println!();

    loop {
        print!("> ");
        stdout.flush().unwrap();

        let mut line = String::new();
        if stdin.lock().read_line(&mut line).is_err() {
            break;
        }

        let input = line.trim();

        match input.to_lowercase().as_str() {
            "quit" | "exit" | "q" => {
                println!("Goodbye!");
                break;
            }
            "telex" => {
                engine.set_input_method(InputMethod::Telex);
                engine.clear();
                println!("Switched to Telex");
                continue;
            }
            "vni" => {
                engine.set_input_method(InputMethod::Vni);
                engine.clear();
                println!("Switched to VNI");
                continue;
            }
            "clear" => {
                engine.clear();
                println!("Buffer cleared");
                continue;
            }
            "" => {
                if !engine.is_empty() {
                    let result = engine.commit();
                    println!("Committed: {}", result);
                }
                continue;
            }
            _ => {}
        }

        // Process each character and show transformation
        engine.clear();
        for ch in input.chars() {
            let output = engine.feed(ch);
            println!("  {} -> {}", ch, output);
        }

        println!();
        println!("Final: {}", engine.output());
        println!();
    }
}

/// Runs the REPL in batch mode, transforming input from stdin.
///
/// Each line is transformed independently and printed to stdout.
pub fn batch(method: InputMethod) {
    let stdin = io::stdin();
    
    for line in stdin.lock().lines() {
        if let Ok(input) = line {
            let output = crate::transform::transform_buffer_with_method(&input, method);
            println!("{}", output);
        }
    }
}
