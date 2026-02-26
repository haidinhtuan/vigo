//! Terminal UI for standalone Vietnamese input.
//!
//! Provides a floating input box that captures keystrokes,
//! transforms them to Vietnamese, and copies to clipboard on Enter.

use std::io::{self, Write};
use crossterm::{
    cursor::{self, MoveTo},
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use arboard::Clipboard;

use crate::syllable_engine::SyllableEngine;
use crate::action::InputMethod;

/// Run the TUI input mode.
pub fn run(method: InputMethod) -> io::Result<()> {
    let mut stdout = io::stdout();
    
    // Setup terminal
    terminal::enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen, cursor::Hide)?;
    
    let result = run_inner(&mut stdout, method);
    
    // Restore terminal
    execute!(stdout, LeaveAlternateScreen, cursor::Show)?;
    terminal::disable_raw_mode()?;
    
    result
}

fn run_inner(stdout: &mut io::Stdout, method: InputMethod) -> io::Result<()> {
    let mut engine = SyllableEngine::new(method);
    let mut clipboard = Clipboard::new().ok();
    let mut message: Option<String> = None;
    
    loop {
        draw(stdout, &engine, method, message.as_deref())?;
        message = None;
        
        if let Event::Key(key) = event::read()? {
            match key {
                // Quit
                KeyEvent { code: KeyCode::Esc, .. } |
                KeyEvent { code: KeyCode::Char('c'), modifiers: KeyModifiers::CONTROL, .. } => {
                    break;
                }
                
                // Copy to clipboard and clear
                KeyEvent { code: KeyCode::Enter, .. } => {
                    let text = engine.output();
                    if !text.is_empty() {
                        if let Some(ref mut cb) = clipboard {
                            if cb.set_text(&text).is_ok() {
                                message = Some(format!("Copied: {}", text));
                            }
                        }
                        engine.clear();
                    }
                }
                
                // Backspace
                KeyEvent { code: KeyCode::Backspace, .. } => {
                    engine.backspace();
                }
                
                // Clear all
                KeyEvent { code: KeyCode::Char('u'), modifiers: KeyModifiers::CONTROL, .. } => {
                    engine.clear();
                }
                
                // Toggle input method
                KeyEvent { code: KeyCode::Tab, .. } => {
                    let new_method = match method {
                        InputMethod::Telex => InputMethod::Vni,
                        InputMethod::Vni => InputMethod::Telex,
                    };
                    return run_inner(stdout, new_method);
                }
                
                // Regular character
                KeyEvent { code: KeyCode::Char(c), modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT, .. } => {
                    engine.feed(c);
                }
                
                _ => {}
            }
        }
    }
    
    Ok(())
}

fn draw(stdout: &mut io::Stdout, engine: &SyllableEngine, method: InputMethod, message: Option<&str>) -> io::Result<()> {
    let (width, height) = terminal::size()?;
    
    // Box dimensions
    let box_width = 50.min(width as usize - 4);
    let box_height = 7;
    let start_x = (width as usize - box_width) / 2;
    let start_y = (height as usize - box_height) / 2;
    
    execute!(stdout, Clear(ClearType::All))?;
    
    // Draw box
    let method_str = match method {
        InputMethod::Telex => "TELEX",
        InputMethod::Vni => "VNI",
    };
    let title = format!(" vigo [{}] ", method_str);
    
    // Top border
    execute!(
        stdout,
        MoveTo(start_x as u16, start_y as u16),
        SetForegroundColor(Color::Cyan),
        Print("╭"),
        Print("─".repeat((box_width - title.len() - 2) / 2)),
        SetForegroundColor(Color::Yellow),
        Print(&title),
        SetForegroundColor(Color::Cyan),
        Print("─".repeat((box_width - title.len() - 1) / 2)),
        Print("╮"),
        ResetColor
    )?;
    
    // Empty line
    execute!(
        stdout,
        MoveTo(start_x as u16, (start_y + 1) as u16),
        SetForegroundColor(Color::Cyan),
        Print("│"),
        Print(" ".repeat(box_width - 2)),
        Print("│"),
        ResetColor
    )?;
    
    // Input line (raw)
    let raw = engine.raw_input();
    let raw_display = if raw.len() > box_width - 10 {
        format!("...{}", &raw[raw.len() - (box_width - 13)..])
    } else {
        raw.to_string()
    };
    execute!(
        stdout,
        MoveTo(start_x as u16, (start_y + 2) as u16),
        SetForegroundColor(Color::Cyan),
        Print("│"),
        ResetColor,
        Print(format!(" Raw: {}", raw_display)),
        MoveTo((start_x + box_width - 1) as u16, (start_y + 2) as u16),
        SetForegroundColor(Color::Cyan),
        Print("│"),
        ResetColor
    )?;
    
    // Output line
    let output = engine.output();
    let out_display = if output.len() > box_width - 10 {
        format!("...{}", &output[output.chars().count() - (box_width - 13)..])
    } else {
        output.clone()
    };
    execute!(
        stdout,
        MoveTo(start_x as u16, (start_y + 3) as u16),
        SetForegroundColor(Color::Cyan),
        Print("│"),
        ResetColor,
        Print(" Out: "),
        SetForegroundColor(Color::Green),
        Print(&out_display),
        ResetColor,
        MoveTo((start_x + box_width - 1) as u16, (start_y + 3) as u16),
        SetForegroundColor(Color::Cyan),
        Print("│"),
        ResetColor
    )?;
    
    // Message or empty line
    execute!(
        stdout,
        MoveTo(start_x as u16, (start_y + 4) as u16),
        SetForegroundColor(Color::Cyan),
        Print("│"),
        ResetColor
    )?;
    if let Some(msg) = message {
        execute!(
            stdout,
            Print(" "),
            SetForegroundColor(Color::Yellow),
            Print(msg),
            ResetColor
        )?;
    }
    execute!(
        stdout,
        MoveTo((start_x + box_width - 1) as u16, (start_y + 4) as u16),
        SetForegroundColor(Color::Cyan),
        Print("│"),
        ResetColor
    )?;
    
    // Bottom border
    execute!(
        stdout,
        MoveTo(start_x as u16, (start_y + 5) as u16),
        SetForegroundColor(Color::Cyan),
        Print("╰"),
        Print("─".repeat(box_width - 2)),
        Print("╯"),
        ResetColor
    )?;
    
    // Help text
    execute!(
        stdout,
        MoveTo(start_x as u16, (start_y + 6) as u16),
        SetForegroundColor(Color::DarkGrey),
        Print(" Enter: copy │ Tab: toggle │ Ctrl+U: clear │ Esc: quit"),
        ResetColor
    )?;
    
    stdout.flush()?;
    Ok(())
}
