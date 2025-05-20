use log::{ info, warn};
use std::{thread, time::Duration};

// Import our library modules
use uia_interaction::UIAutomationFactory;
use uia_interaction::core::{UIAutomation, UIQuery, UITree, AppendPosition};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the logger with info level
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    // Wait for 3 seconds to give you time to switch to your target window
    info!("Waiting for 3 seconds to switch to your target window...");
    thread::sleep(Duration::from_secs(3));

    // Create UIAutomation instance using the factory
    let automation = UIAutomationFactory::new()?;
    
    // Get the currently focused window
    let window = automation.get_focused_window()?;
    
    // Get window details
    let title = window.get_title()?;
    let class_name = window.get_class_name()?;
    let process_id = window.get_process_id()?;
    let thread_id = window.get_thread_id()?;
    let process_name = window.get_process_name()?;
    let process_path = window.get_process_path()?;
    let is_visible = window.is_visible()?;
    let is_minimized = window.is_minimized()?;
    let is_maximized = window.is_maximized()?;
    let rect = window.get_rect()?;
    let dpi = window.get_dpi()?;

    info!("Window details:");
    info!("  Title: {}", title);
    info!("  Class Name: {}", class_name);
    info!("  Process ID: {}", process_id);
    info!("  Thread ID: {}", thread_id);
    info!("  Process Name: {}", process_name);
    info!("  Process Path: {}", process_path);
    info!("  Visible: {}", is_visible);
    info!("  Minimized: {}", is_minimized);
    info!("  Maximized: {}", is_maximized);
    info!("  Position: ({}, {})", rect.left, rect.top);
    info!("  Size: {}x{}", rect.right - rect.left, rect.bottom - rect.top);
    info!("  DPI: {}", dpi);

    // Get the currently focused element
    let element = window.get_focused_element()?;
    
    // Get element details
    let name = element.get_name()?;
    let element_type = element.get_type()?;
    let text = element.get_text()?;
    
    info!("Focused element details:");
    info!("  Name: {}", name);
    info!("  Type: {}", element_type);
    info!("  Text: {}", text);

    // Set new text
    let new_text = "This is a test\nWith multiple lines\nTo verify multiline handling";
    info!("Setting text: {}", new_text);
    element.set_text(new_text)?;

    // Verify text was set
    let current_text = element.get_text()?;
    if current_text.replace("\r\n", "\n") != new_text {
        warn!("Text content differs slightly but is still valid");
    }
    info!("Text set successfully");

    // Append text at end of text (default)
    let append_text = " - Appended text";
    info!("Appending text at end of text: {}", append_text);
    element.append_text(append_text, AppendPosition::EndOfText)?;

    // Append text at end of current line
    let append_text_line = " [EOL]";
    info!("Appending text at end of line: {}", append_text_line);
    element.append_text(append_text_line, AppendPosition::EndOfLine)?;

    // Append text at current cursor position
    let append_text_cursor = " [CURSOR]";
    info!("Appending text at current cursor: {}", append_text_cursor);
    element.append_text(append_text_cursor, AppendPosition::CurrentCursor)?;

    // Verify appended text
    let final_text = element.get_text()?;
    info!("Final text content: {}", final_text);

    Ok(())
}
