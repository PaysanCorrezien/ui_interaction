use anyhow::Result;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyModule};

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
    // Initialize Python
    pyo3::prepare_freethreaded_python();

    // Example Python code that interacts with UI and returns data
    let python_code = r#"
# Direct usage of automation object
def collect_ui_info():
    # Get the focused window
    window = automation.focused_window()
    
    # Get the complete UI tree
    tree = window.get_ui_tree()
    
    # Print the tree structure
    def print_tree(node, indent=0):
        print(f"{' ' * indent}{node.name} ({node.control_type})")
        for child in node.children:
            print_tree(child, indent + 2)
    
    print("UI Tree Structure:")
    print_tree(tree.root)
    
    # Create query objects properly using PyUIQuery
    from uia_interaction import PyUIQuery
    
    # Create individual queries
    button_type_query = PyUIQuery.by_type("Button")
    
    # Find all buttons first
    buttons = window.find_elements(button_type_query)
    
    # Filter enabled buttons
    enabled_buttons = [button for button in buttons if button.is_enabled]
    
    # Collect information about found buttons
    button_info = []
    for button in enabled_buttons:
        button_info.append({
            "name": button.name,
            "type": button.control_type,
            "properties": button.get_properties()
        })
    
    return {
        "window_title": tree.window_title,
        "window_class": tree.window_class,
        "timestamp": tree.timestamp,
        "buttons": button_info
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
