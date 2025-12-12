//! Extract Text Elements as JSON
//!
//! This example extracts text elements from a window and outputs them as JSON,
//! making it easy to process the data programmatically.
//!
//! Usage:
//!   cargo run --example extract_text_json
//!   cargo run --example extract_text_json -- --app outlook > outlook_text.json
//!   cargo run --example extract_text_json -- --focused
//!   cargo run --example extract_text_json -- --selected-only

use std::env;
use std::error::Error;
use std::thread;
use std::time::Duration;

use serde::Serialize;
use ui_interaction::{
    ApplicationManagerFactory, TextElementInfo, TextExtractionOptions, UIAutomationFactory,
    SelectedTextInfo,
};

/// JSON output structure for text extraction results
#[derive(Serialize)]
struct TextExtractionResult {
    window_title: String,
    process_name: String,
    process_id: u32,
    timestamp: String,
    total_elements: usize,
    elements: Vec<TextElementInfo>,
    selected_text: Option<SelectedTextInfo>,
}

fn extract_to_json(app_name: Option<&str>, focused: bool, selected_only: bool) -> Result<(), Box<dyn Error>> {
    let window = if focused || app_name.is_none() {
        // Use focused window
        if !focused {
            eprintln!("Waiting 3 seconds for you to focus on target window...");
            thread::sleep(Duration::from_secs(3));
        }
        let automation = UIAutomationFactory::new()?;
        automation.get_active_window()?
    } else {
        // Find by app name
        let app_manager = ApplicationManagerFactory::new()?;
        let app_name = app_name.unwrap();

        // Try process name first
        let apps = app_manager.find_applications_by_name(app_name)?;
        if !apps.is_empty() {
            app_manager.get_window_by_process_id(apps[0].process_id)?
        } else {
            // Try by title
            let apps = app_manager.find_applications_by_title(app_name)?;
            if !apps.is_empty() {
                app_manager.get_window_by_process_id(apps[0].process_id)?
            } else {
                return Err(format!("Application '{}' not found", app_name).into());
            }
        }
    };

    let window_title = window.get_title()?;
    let process_name = window.get_process_name()?;
    let process_id = window.get_process_id()?;

    // Get selected text
    let selected_text = window.get_selected_text().ok().flatten();

    // Get all text elements (unless only selected text is requested)
    let elements = if selected_only {
        Vec::new()
    } else {
        let options = TextExtractionOptions::default();
        window.get_text_elements(&options)?
    };

    let result = TextExtractionResult {
        window_title,
        process_name,
        process_id,
        timestamp: chrono::Utc::now().to_rfc3339(),
        total_elements: elements.len(),
        elements,
        selected_text,
    };

    // Output as JSON
    let json = serde_json::to_string_pretty(&result)?;
    println!("{}", json);

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    let mut app_name: Option<&str> = None;
    let mut focused = false;
    let mut selected_only = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--app" => {
                if i + 1 < args.len() {
                    app_name = Some(&args[i + 1]);
                    i += 1;
                }
            }
            "--focused" => focused = true,
            "--selected-only" => selected_only = true,
            "--help" | "-h" => {
                eprintln!("Extract text elements from Windows applications as JSON\n");
                eprintln!("Usage:");
                eprintln!("  extract_text_json [options]\n");
                eprintln!("Options:");
                eprintln!("  --app <name>      Find and extract from application by name");
                eprintln!("  --focused         Extract from currently focused window");
                eprintln!("  --selected-only   Only output selected text, not all elements");
                eprintln!("  --help, -h        Show this help message\n");
                eprintln!("Examples:");
                eprintln!("  cargo run --example extract_text_json -- --app outlook");
                eprintln!("  cargo run --example extract_text_json -- --focused");
                eprintln!("  cargo run --example extract_text_json -- --selected-only");
                return Ok(());
            }
            _ => {}
        }
        i += 1;
    }

    extract_to_json(app_name, focused, selected_only)
}
