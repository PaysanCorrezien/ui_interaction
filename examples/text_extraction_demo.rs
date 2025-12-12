//! Text Extraction Demo
//!
//! This example demonstrates how to extract text elements from various Windows applications
//! including Outlook, Cursor IDE, and Claude desktop app.
//!
//! Usage:
//!   cargo run --example text_extraction_demo
//!   cargo run --example text_extraction_demo -- --app outlook
//!   cargo run --example text_extraction_demo -- --app cursor
//!   cargo run --example text_extraction_demo -- --app claude

use std::env;
use std::error::Error;
use std::thread;
use std::time::Duration;

use ui_interaction::{
    ApplicationManagerFactory, TextElementInfo, TextExtractionOptions, UIAutomationFactory,
};

/// Pretty print a TextElementInfo
fn print_text_element(elem: &TextElementInfo, index: usize) {
    println!("\n--- Element {} ---", index);
    println!("  Type: {}", elem.control_type);
    println!("  Name: '{}'", elem.name);
    println!(
        "  Text: '{}'",
        if elem.text.len() > 100 {
            format!("{}...", &elem.text[..100])
        } else {
            elem.text.clone()
        }
    );

    if let Some(bounds) = &elem.bounds {
        println!(
            "  Bounds: ({}, {}) - {}x{}",
            bounds.left,
            bounds.top,
            bounds.width(),
            bounds.height()
        );
    }

    if let Some(id) = &elem.automation_id {
        println!("  AutomationId: {}", id);
    }

    println!(
        "  Flags: visible={}, enabled={}, editable={}",
        elem.is_visible, elem.is_enabled, elem.is_editable
    );
}

/// Extract text from the currently focused window
fn extract_from_focused_window() -> Result<(), Box<dyn Error>> {
    println!("Extracting text from focused window...\n");

    let automation = UIAutomationFactory::new()?;
    let window = automation.get_active_window()?;

    println!("Window: {}", window.get_title()?);
    println!("Process: {}", window.get_process_name()?);
    println!("Class: {}", window.get_class_name()?);

    // Extract text with default options
    let options = TextExtractionOptions::default();
    let elements = window.get_text_elements(&options)?;

    println!("\nFound {} text elements:", elements.len());

    for (i, elem) in elements.iter().enumerate().take(30) {
        print_text_element(elem, i + 1);
    }

    if elements.len() > 30 {
        println!("\n... and {} more elements", elements.len() - 30);
    }

    // Check for selected text
    println!("\n--- Checking for selected text ---");
    if let Ok(Some(selection)) = window.get_selected_text() {
        println!("Selected text: '{}'", selection.text);
        if let Some(bounds) = &selection.bounds {
            println!(
                "Selection bounds: ({}, {}) - {}x{}",
                bounds.left,
                bounds.top,
                bounds.width(),
                bounds.height()
            );
        }
    } else {
        println!("No text currently selected.");
    }

    Ok(())
}

/// Find and extract text from a specific application by name
fn extract_from_app(app_name: &str) -> Result<(), Box<dyn Error>> {
    println!("Looking for application: {}...\n", app_name);

    let app_manager = ApplicationManagerFactory::new()?;

    // Try different search strategies
    let mut window = None;

    // Search by process name
    let apps = app_manager.find_applications_by_name(app_name)?;
    if !apps.is_empty() {
        println!("Found by process name: {} (PID: {})", apps[0].process_name, apps[0].process_id);
        window = Some(app_manager.get_window_by_process_id(apps[0].process_id)?);
    }

    // If not found, try by title
    if window.is_none() {
        let apps = app_manager.find_applications_by_title(app_name)?;
        if !apps.is_empty() {
            println!(
                "Found by title: '{}' (PID: {})",
                apps[0].main_window_title, apps[0].process_id
            );
            window = Some(app_manager.get_window_by_process_id(apps[0].process_id)?);
        }
    }

    let window = window.ok_or_else(|| format!("Application '{}' not found", app_name))?;

    println!("\nWindow: {}", window.get_title()?);
    println!("Process: {}", window.get_process_name()?);

    // Extract text elements
    let options = TextExtractionOptions::default();
    let elements = window.get_text_elements(&options)?;

    println!("\nFound {} text elements:", elements.len());

    for (i, elem) in elements.iter().enumerate().take(30) {
        print_text_element(elem, i + 1);
    }

    if elements.len() > 30 {
        println!("\n... and {} more elements", elements.len() - 30);
    }

    // Check for selected text
    println!("\n--- Checking for selected text ---");
    if let Ok(Some(selection)) = window.get_selected_text() {
        println!("Selected text: '{}'", selection.text);
    } else {
        println!("No text currently selected (or window not focused).");
    }

    Ok(())
}

/// Extract text from Outlook
fn extract_from_outlook() -> Result<(), Box<dyn Error>> {
    println!("=== Outlook Text Extraction ===\n");

    let app_manager = ApplicationManagerFactory::new()?;

    // Outlook can have various process names
    let outlook_names = ["OUTLOOK.EXE", "outlook.exe", "Outlook", "olk.exe"];

    let mut window = None;
    for name in &outlook_names {
        if let Ok(apps) = app_manager.find_applications_by_name(name) {
            if !apps.is_empty() {
                println!("Found Outlook: {} (PID: {})", apps[0].process_name, apps[0].process_id);
                window = Some(app_manager.get_window_by_process_id(apps[0].process_id)?);
                break;
            }
        }
    }

    // Also try by title
    if window.is_none() {
        if let Ok(apps) = app_manager.find_applications_by_title("Outlook") {
            if !apps.is_empty() {
                println!("Found Outlook by title: '{}'", apps[0].main_window_title);
                window = Some(app_manager.get_window_by_process_id(apps[0].process_id)?);
            }
        }
    }

    let window = window.ok_or("Outlook not found. Please start Outlook and try again.")?;

    // Use options focused on document/text content
    let options = TextExtractionOptions {
        include_hidden: false,
        include_disabled: true,
        min_text_length: 1,
        control_types: None, // Get all types to capture email content
        max_depth: Some(25),
        include_names_as_text: true,
    };

    let elements = window.get_text_elements(&options)?;

    println!("\n=== Email Content ===");
    println!("Found {} text elements\n", elements.len());

    // Filter for likely email content
    let email_content: Vec<_> = elements
        .iter()
        .filter(|e| {
            // Focus on text, document, and edit controls
            matches!(
                e.control_type.as_str(),
                "Text" | "Document" | "Edit" | "Pane"
            ) && e.text.len() > 10
        })
        .collect();

    println!("Email-related content ({} items):", email_content.len());
    for (i, elem) in email_content.iter().enumerate().take(20) {
        print_text_element(elem, i + 1);
    }

    Ok(())
}

/// Extract text from Cursor IDE
fn extract_from_cursor() -> Result<(), Box<dyn Error>> {
    println!("=== Cursor IDE Text Extraction ===\n");

    let app_manager = ApplicationManagerFactory::new()?;

    // Cursor uses electron-based window
    let cursor_names = ["Cursor.exe", "cursor.exe", "Cursor"];

    let mut window = None;
    for name in &cursor_names {
        if let Ok(apps) = app_manager.find_applications_by_name(name) {
            if !apps.is_empty() {
                println!("Found Cursor: {} (PID: {})", apps[0].process_name, apps[0].process_id);
                window = Some(app_manager.get_window_by_process_id(apps[0].process_id)?);
                break;
            }
        }
    }

    if window.is_none() {
        if let Ok(apps) = app_manager.find_applications_by_title("Cursor") {
            if !apps.is_empty() {
                println!("Found Cursor by title: '{}'", apps[0].main_window_title);
                window = Some(app_manager.get_window_by_process_id(apps[0].process_id)?);
            }
        }
    }

    let window = window.ok_or("Cursor not found. Please start Cursor and try again.")?;

    // Options for code editor
    let options = TextExtractionOptions {
        include_hidden: false,
        include_disabled: true,
        min_text_length: 1,
        control_types: None,
        max_depth: Some(30),
        include_names_as_text: true,
    };

    let elements = window.get_text_elements(&options)?;

    println!("\n=== Editor Content ===");
    println!("Found {} text elements\n", elements.len());

    // Filter for editor-related content
    let code_content: Vec<_> = elements
        .iter()
        .filter(|e| e.text.len() > 5)
        .collect();

    println!("Text content ({} items):", code_content.len());
    for (i, elem) in code_content.iter().enumerate().take(20) {
        print_text_element(elem, i + 1);
    }

    Ok(())
}

/// Extract text from Claude desktop app
fn extract_from_claude() -> Result<(), Box<dyn Error>> {
    println!("=== Claude Desktop Text Extraction ===\n");

    let app_manager = ApplicationManagerFactory::new()?;

    // Claude desktop app
    let claude_names = ["claude.exe", "Claude.exe", "Claude"];

    let mut window = None;
    for name in &claude_names {
        if let Ok(apps) = app_manager.find_applications_by_name(name) {
            if !apps.is_empty() {
                println!("Found Claude: {} (PID: {})", apps[0].process_name, apps[0].process_id);
                window = Some(app_manager.get_window_by_process_id(apps[0].process_id)?);
                break;
            }
        }
    }

    if window.is_none() {
        if let Ok(apps) = app_manager.find_applications_by_title("Claude") {
            if !apps.is_empty() {
                println!("Found Claude by title: '{}'", apps[0].main_window_title);
                window = Some(app_manager.get_window_by_process_id(apps[0].process_id)?);
            }
        }
    }

    let window = window.ok_or("Claude not found. Please start Claude desktop app and try again.")?;

    let options = TextExtractionOptions {
        include_hidden: false,
        include_disabled: true,
        min_text_length: 1,
        control_types: None,
        max_depth: Some(30),
        include_names_as_text: true,
    };

    let elements = window.get_text_elements(&options)?;

    println!("\n=== Chat Content ===");
    println!("Found {} text elements\n", elements.len());

    // Filter for chat content
    let chat_content: Vec<_> = elements
        .iter()
        .filter(|e| e.text.len() > 10)
        .collect();

    println!("Chat content ({} items):", chat_content.len());
    for (i, elem) in chat_content.iter().enumerate().take(20) {
        print_text_element(elem, i + 1);
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logging
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    let args: Vec<String> = env::args().collect();

    // Parse command line arguments
    let app = if args.len() > 2 && args[1] == "--app" {
        Some(args[2].as_str())
    } else {
        None
    };

    match app {
        Some("outlook") => {
            extract_from_outlook()?;
        }
        Some("cursor") => {
            extract_from_cursor()?;
        }
        Some("claude") => {
            extract_from_claude()?;
        }
        Some(app_name) => {
            extract_from_app(app_name)?;
        }
        None => {
            println!("=== Text Extraction Demo ===\n");
            println!("No app specified. Extracting from focused window...\n");
            println!("You have 3 seconds to focus on your target window...\n");
            thread::sleep(Duration::from_secs(3));
            extract_from_focused_window()?;
        }
    }

    Ok(())
}
