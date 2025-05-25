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
    debug!("Starting debug window example");

    // Initialize Python
    pyo3::prepare_freethreaded_python();

    // Debug Python code to understand the current window
    let python_code = r#"
def debug_current_window():
    # Get the active window
    window = automation.active_window()
    print(f"Active window title: '{window.title}'")
    
    # Get the complete UI tree
    tree = window.get_ui_tree()
    print(f"Window class: '{tree.window_class}'")
    print(f"Window title from tree: '{tree.window_title}'")
    
    # Print the top-level structure
    def print_tree(node, indent=0, max_depth=2):
        if indent > max_depth:
            return
        enabled_str = " (enabled)" if node.is_enabled else " (disabled)"
        visible_str = " (visible)" if node.is_visible else " (hidden)"
        print(f"{' ' * indent}'{node.name}' ({node.control_type}){enabled_str}{visible_str}")
        
        # Show first few children
        for i, child in enumerate(node.children):
            if i >= 10:  # Limit to first 10 children
                print(f"{' ' * (indent + 2)}... and {len(node.children) - 10} more children")
                break
            print_tree(child, indent + 2, max_depth)
    
    print("\nUI Tree Structure:")
    print_tree(tree.root)
    
    # Try to find common menu elements
    from uia_interaction import PyUIQuery
    
    print("\n=== Looking for common menu elements ===")
    common_names = ["File", "Edit", "View", "Help", "Menu", "MenuBar"]
    
    for name in common_names:
        query = PyUIQuery.by_name(name)
        elements = window.find_elements(query)
        if elements:
            print(f"Found {len(elements)} element(s) named '{name}':")
            for i, element in enumerate(elements[:3]):  # Show first 3
                print(f"  {i+1}. {element.name} ({element.control_type}) - enabled: {element.is_enabled}")
        else:
            print(f"No elements found named '{name}'")
    
    # Try to find menu bar by type
    print("\n=== Looking for MenuBar elements ===")
    menubar_query = PyUIQuery.by_type("MenuBar")
    menubars = window.find_elements(menubar_query)
    if menubars:
        print(f"Found {len(menubars)} MenuBar element(s):")
        for i, menubar in enumerate(menubars):
            print(f"  {i+1}. {menubar.name} ({menubar.control_type})")
            # Get children of the menubar
            try:
                children = menubar.get_children()
                print(f"     MenuBar has {len(children)} children:")
                for j, child in enumerate(children[:5]):  # Show first 5
                    print(f"       {j+1}. {child.name} ({child.control_type})")
            except Exception as e:
                print(f"     Error getting MenuBar children: {e}")
    else:
        print("No MenuBar elements found")
    
    return {
        "window_title": tree.window_title,
        "window_class": tree.window_class,
        "analysis_complete": True
    }

# Run the debug analysis
result = debug_current_window()
"#;

    // Run the Python code and get the result
    let result = run_python_code(python_code)?;
    
    // Display the result
    println!("\nDebug Analysis Result:");
    println!("{}", result);

    Ok(())
} 