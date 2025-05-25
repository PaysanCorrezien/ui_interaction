use anyhow::Result;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyModule};
use env_logger::Env;

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

        // Create and inject the application manager object
        let app_manager = Py::new(py, python_bindings::PyApplicationManager::new()?)?;
        globals.set_item("app_manager", app_manager)?;

        // Register the uia_interaction module
        let module = PyModule::new_bound(py, "uia_interaction")?;
        module.add_class::<python_bindings::PyAutomation>()?;
        module.add_class::<python_bindings::PyWindow>()?;
        module.add_class::<python_bindings::PyUIElement>()?;
        module.add_class::<python_bindings::PyUITree>()?;
        module.add_class::<python_bindings::PyUITreeNode>()?;
        module.add_class::<python_bindings::PyUIQuery>()?;
        module.add_class::<python_bindings::PyApplicationInfo>()?;
        module.add_class::<python_bindings::PyApplicationManager>()?;
        py.import_bound("sys")?.getattr("modules")?.set_item("uia_interaction", module)?;

        // Run the Python code
        py.run_bound(code, Some(&globals), None)?;
        
        // Get the result from the globals
        let result = globals.get_item("result")?.unwrap();
        Ok(result.to_string())
    })
}

fn main() -> Result<()> {
    // Initialize logging with error level only (minimal)
    env_logger::Builder::from_env(Env::default().default_filter_or("error")).init();

    // Initialize Python
    pyo3::prepare_freethreaded_python();

    println!("🚀 Vesktop Demo");
    println!("Make sure Vesktop/Discord is running, then press Enter...");
    std::io::stdin().read_line(&mut String::new()).ok();

    // Python code that demonstrates simple Vesktop interaction
    let python_code = r#"
import time
from uia_interaction import PyUIQuery

def vesktop_demo():
    # Find Vesktop application
    vesktop_apps = app_manager.find_applications_by_name("vesktop")
    if not vesktop_apps:
        return {"success": False, "error": "Vesktop not found. Please start Vesktop/Discord."}
    
    vesktop_app = vesktop_apps[0]
    print(f"Found: {vesktop_app.process_name} - '{vesktop_app.main_window_title}'")
    
    # Get and activate the window
    window = app_manager.get_window_by_process_id(vesktop_app.process_id)
    window.activate()
    time.sleep(1)
    
    print(f"Connected to: {window.title}")
    
    # Find message input (try Edit first, then Document)
    message_input = None
    
    # Try Edit controls first
    edit_elements = window.find_elements(PyUIQuery.by_type("Edit"))
    for element in edit_elements:
        if element.is_enabled:
            message_input = element
            break
    
    # If no Edit found, try Document controls  
    if not message_input:
        doc_elements = window.find_elements(PyUIQuery.by_type("Document"))
        for element in doc_elements:
            if element.is_enabled:
                message_input = element
                break
    
    if not message_input:
        return {"success": False, "error": "Message input field not found"}
    
    # Send message
    test_message = "Hello from Rust UI Automation! 🚀"
    message_input.click()
    time.sleep(0.5)
    message_input.set_text(test_message)
    
    print("✅ Message ready to send! Press Enter in Discord to send it.")
    return {"success": True, "message": "Message placed in Discord input field"}

# Run the demo
result = vesktop_demo()
print(f"Result: {result}")
"#;

    // Run the demo
    let _result = run_python_code(python_code)?;

    Ok(())
} 