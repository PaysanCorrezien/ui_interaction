use anyhow::Result;
use pyo3::prelude::*;
use pyo3::types::PyDict;

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

        // Run the Python code
        py.run_bound(code, Some(&globals), None)?;
        
        // Get the result from the globals
        let result = globals.get_item("result")?.unwrap();
        Ok(result.to_string())
    })
}

fn main() -> Result<()> {
    // Initialize Python
    pyo3::prepare_freethreaded_python();

    // Example Python code that interacts with UI and returns data
    let python_code = r#"
def collect_ui_info():
    # Get the focused window
    window = automation.focused_window()
    title = window.title
    
    # Get the focused element
    element = window.focused_element()
    element_info = {}
    if element:
        element_info = {
            "name": element.name,
            "type": element.control_type
        }
    
    return {
        "window_title": title,
        "focused_element": element_info
    }

# Collect UI information and store it in the globals
result = collect_ui_info()
"#;

    // Run the Python code and get the result
    let result = run_python_code(python_code)?;
    
    // Display the result
    println!("UI Information from Python:");
    println!("{}", result);

    Ok(())
}
