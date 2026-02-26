//! Vigo CLI - Vietnamese Input Method Engine
//!
//! A command-line interface for testing and using the Vigo Vietnamese input engine.

use std::env;
use vigo::InputMethod;

fn print_help() {
    println!("Vigo - Vietnamese Input Method Engine");
    println!();
    println!("USAGE:");
    println!("    vigo [OPTIONS] [COMMAND]");
    println!();
    println!("COMMANDS:");
    println!("    repl              Interactive REPL mode (default)");
    println!("    batch             Transform stdin line by line");
    println!("    transform <text>  Transform a single text");
    println!();
    println!("OPTIONS:");
    println!("    --telex           Use Telex input method (default)");
    println!("    --vni             Use VNI input method");
    println!("    -h, --help        Print this help message");
    println!("    -V, --version     Print version");
    println!();
    println!("EXAMPLES:");
    println!("    vigo                      # Start interactive REPL");
    println!("    vigo --vni repl           # Start REPL with VNI");
    println!("    vigo transform \"vieetj\"   # Transform text");
    println!("    echo \"chaof\" | vigo batch # Batch transform");
}

fn print_version() {
    println!("vigo {}", env!("CARGO_PKG_VERSION"));
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    let mut method = InputMethod::Telex;
    let mut command = "repl";
    let mut text: Option<&str> = None;
    
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                print_help();
                return;
            }
            "-V" | "--version" => {
                print_version();
                return;
            }
            "--telex" => {
                method = InputMethod::Telex;
            }
            "--vni" => {
                method = InputMethod::Vni;
            }
            "repl" => {
                command = "repl";
            }
            "batch" => {
                command = "batch";
            }
            "transform" => {
                command = "transform";
                if i + 1 < args.len() {
                    i += 1;
                    text = Some(&args[i]);
                }
            }
            arg if !arg.starts_with('-') && command == "repl" => {
                // Treat as transform command with text
                command = "transform";
                text = Some(&args[i]);
            }
            _ => {
                eprintln!("Unknown option: {}", args[i]);
                eprintln!("Use --help for usage information");
                std::process::exit(1);
            }
        }
        i += 1;
    }
    
    match command {
        "repl" => {
            vigo::repl::run(method);
        }
        "batch" => {
            vigo::repl::batch(method);
        }
        "transform" => {
            if let Some(input) = text {
                let output = vigo::transform::transform_buffer_with_method(input, method);
                println!("{}", output);
            } else {
                eprintln!("Error: transform command requires text argument");
                std::process::exit(1);
            }
        }
        _ => unreachable!(),
    }
}
