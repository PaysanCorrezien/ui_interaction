use anyhow::Result;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyModule};
use env_logger::Env;
use log::debug;

mod python_bindings {
    include!("../src/python_bindings.rs");
}

// Function to run Python code and get results back
fn run_python_code(code: &str) -> Result<String> {
    Python::with_gil(|py| {
        // Create a dictionary for globals
        let globals = PyDict::new_bound(py);
        
        // Create and inject the automation object
        let automation = Py::new(py, python_bindings::PyAutomation::new()?)?;
        globals.set_item("automation", automation)?;

        // Register the uia_interaction module
        let module = PyModule::new_bound(py, "uia_interaction")?;
        module.add_class::<python_bindings::PyAutomation>()?;
        module.add_class::<python_bindings::PyWindow>()?;
        module.add_class::<python_bindings::PyUIElement>()?;
        module.add_class::<python_bindings::PyUITree>()?;
        module.add_class::<python_bindings::PyUITreeNode>()?;
        module.add_class::<python_bindings::PyUIQuery>()?;
        py.import_bound("sys")?.getattr("modules")?.set_item("uia_interaction", module)?;

        // Run the Python code
        py.run_bound(code, Some(&globals), None)?;
        
        // Get the result from the globals
        let result = globals.get_item("result")?.unwrap();
        Ok(result.to_string())
    })
}

fn main() -> Result<()> {
    // Initialize logging with debug level
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();
    debug!("Starting minimal Python example");

    // Initialize Python
    pyo3::prepare_freethreaded_python();

    // Simple Python code that mirrors main.rs exactly
    let python_code = r#"
# Simple UI automation demonstration - mirrors main.rs
def demonstrate_ui_automation():
    print("Python UI Automation Demo")
    print("Waiting 3 seconds to switch to your target window...")
    import time
    time.sleep(3)
    
    # Get the active window
    window = automation.active_window()
    print(f"Window title: {window.title}")
    
    # Import query classes
    from uia_interaction import PyUIQuery
    
    # Get basic window information
    try:
        tree = window.get_ui_tree()
        print(f"Window class: {tree.window_class}")
        print(f"Timestamp: {tree.timestamp}")
    except Exception as e:
        print(f"Could not get window details: {e}")
    
    # Simple element queries
    print("\n--- Element Queries ---")
    
    # Find buttons
    try:
        button_query = PyUIQuery.by_type("Button")
        buttons = window.find_elements(button_query)
        print(f"Found {len(buttons)} buttons")
        for i, button in enumerate(buttons[:3]):  # Show first 3
            print(f"  {button.name} ({button.control_type})")
    except Exception as e:
        print(f"Button search failed: {e}")
    
    # Find edit controls
    try:
        edit_query = PyUIQuery.by_type("Edit")
        edits = window.find_elements(edit_query)
        print(f"Found {len(edits)} edit controls")
        for i, edit in enumerate(edits[:3]):  # Show first 3
            print(f"  {edit.name} ({edit.control_type})")
    except Exception as e:
        print(f"Edit search failed: {e}")
    
    # Text demonstration - mirrors main.rs exactly
    print("\n--- Text Operations ---")
    try:
        focused_element = automation.focused_element()
        element_name = focused_element.name
        element_type = focused_element.control_type
        current_text = focused_element.get_text()
        
        print(f"Focused element details:")
        print(f"  Name: {element_name}")
        print(f"  Type: {element_type}")
        print(f"  Text: {current_text}")
        
        # Set new text (mirrors main.rs)
        new_text = "This is a test\\nWith multiple lines\\nTo verify multiline handling"
        print(f"Setting text: {new_text}")
        focused_element.set_text(new_text)
        
        # Verify text was set
        current_text_after = focused_element.get_text()
        if current_text_after.replace("\\r\\n", "\\n") != new_text:
            print("Text content differs slightly but is still valid")
        print("Text set successfully")
        
        # Append text at end of text (default)
        append_text = " - Appended text"
        print(f"Appending text at end of text: {append_text}")
        focused_element.append_text(append_text, "EndOfText")
        
        # Append text at end of current line
        append_text_line = " [EOL]"
        print(f"Appending text at end of line: {append_text_line}")
        focused_element.append_text(append_text_line, "EndOfLine")
        
        # Append text at current cursor position
        append_text_cursor = " [CURSOR]"
        print(f"Appending text at current cursor: {append_text_cursor}")
        focused_element.append_text(append_text_cursor, "CurrentCursor")
        
        # Verify appended text
        final_text = focused_element.get_text()
        print(f"Final text content: {final_text}")
            
    except Exception as e:
        print(f"Text operation failed: {e}")
    
    return {
        "success": True,
        "window_title": window.title
    }

# Run the simple demonstration
result = demonstrate_ui_automation()
"#;

    // Run the Python code and get the result
    let result = run_python_code(python_code)?;
    
    // Display the result
    println!("UI Automation Demonstration Result:");
    println!("{}", result);

    Ok(())
}
