use log::{error, info};
use std::{thread, time::Duration};

// Import our library modules
use uia_interaction::{
    UIAutomation,
    ElementFinder,
    TextHandler,
    WindowManager,
    AppContext,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the logger
    env_logger::init();

    // Wait for 3 seconds to give you time to switch to your target window
    info!("Waiting for 3 seconds to switch to your target window...");
    thread::sleep(Duration::from_secs(3));

    // Create UIAutomation instance
    let automation = UIAutomation::new()?;
    
    // Create specialized handlers
    let window_manager = WindowManager::new(&automation);
    let element_finder = ElementFinder::new(&automation);
    let text_handler = TextHandler::new(&automation);

    // Get the currently focused window info using the WindowManager
    match window_manager.get_focused_window() {
        Ok((window, title, control_type, classname)) => {
            info!(
                "Focused window - Title: {}, ControlType: {}, ClassName: {}",
                title, control_type, classname
            );
        }
        Err(e) => error!("Failed to get focused window: {}", e),
    }

    // Get the currently focused element info
    match element_finder.get_focused_element() {
        Ok((_, name, control_type)) => {
            info!("Focused Element - Name: {}, Type: {}", name, control_type);
        }
        Err(e) => error!("Failed to get focused element: {}", e),
    }

    // Try to get the focused input element and interact with it
    match element_finder.get_focused_input_element() {
        Ok(input_field) => {
            // Capture input field text
            match text_handler.get_text(&input_field) {
                Ok(text) => info!("Current input text: {}", text),
                Err(e) => error!("Failed to capture input text: {}", e),
            }

            // Set new input text - using a multiline text example
            let multiline_text = "This is a longer text input\nwith multiple lines\nand more content to test the functionality.";
            if let Err(e) = text_handler.set_text(&input_field, multiline_text) {
                error!("Failed to set input text: {}", e);
            }
        }
        Err(e) => {
            info!("No focused input element found. Make sure your cursor is in a text field.");
            error!("Error finding focused input element: {}", e);
        }
    }

    // Append text to the input field
    if let Ok(input_field) = element_finder.get_focused_input_element() {
        if let Err(e) = text_handler.append_text(&input_field, " - Appended text from uia_automation") {
            error!("Failed to append text: {}", e);
        }
    }

    // Get the app context and print the report
    match AppContext::new(&element_finder, &text_handler) {
        Ok(context) => {
            let report = context.generate_report(&text_handler);
            info!("App Context Report:\n{}", report);
        }
        Err(e) => error!("Failed to get app context: {}", e),
    }

    info!("Done!");
    Ok(())
}
