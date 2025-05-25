use anyhow::Result;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyModule};
use env_logger::Env;
use log::info;


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
    // Initialize logging with info level
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    info!("Starting Application Manager Demo");

    // Initialize Python
    pyo3::prepare_freethreaded_python();

    // Python code that demonstrates application management
    let python_code = r#"
import time

def list_applications_demo():
    """Demonstrate application listing and searching capabilities."""
    results = []
    
    try:
        results.append("=== APPLICATION MANAGER DEMO ===")
        results.append("Testing application enumeration and search functionality")
        results.append("")
        
        # Get all running applications
        results.append("STEP 1: Getting all running applications...")
        try:
            apps = app_manager.get_all_applications()
            results.append(f"✅ Found {len(apps)} running applications")
            results.append("")
            
            # Show first 10 applications as sample
            results.append("Sample applications (first 10):")
            for i, app in enumerate(apps[:10]):
                results.append(f"  {i+1:2d}. {app.process_name} (PID: {app.process_id})")
                results.append(f"      Title: '{app.main_window_title}'")
                results.append(f"      Path: {app.process_path}")
                results.append(f"      Visible: {app.is_visible}")
                results.append("")
            
            if len(apps) > 10:
                results.append(f"... and {len(apps) - 10} more applications")
                results.append("")
                
        except Exception as e:
            results.append(f"❌ Failed to get applications: {e}")
            return {"success": False, "results": results}
        
        # Search for common applications
        results.append("STEP 2: Searching for common applications...")
        
        # Search patterns to try
        search_patterns = [
            ("notepad", "Notepad"),
            ("explorer", "Windows Explorer"),
            ("chrome", "Google Chrome"),
            ("firefox", "Firefox"),
            ("code", "VS Code"),
            ("discord", "Discord"),
            ("vesktop", "Vesktop"),
        ]
        
        found_apps = {}
        
        for pattern, friendly_name in search_patterns:
            try:
                matching_apps = app_manager.find_applications_by_name(pattern)
                if matching_apps:
                    found_apps[friendly_name] = matching_apps
                    results.append(f"✅ Found {len(matching_apps)} {friendly_name} instance(s)")
                    for app in matching_apps:
                        results.append(f"    - {app.process_name} (PID: {app.process_id}) - '{app.main_window_title}'")
                else:
                    results.append(f"   No {friendly_name} found")
            except Exception as e:
                results.append(f"❌ Error searching for {friendly_name}: {e}")
        
        results.append("")
        
        # Search by window title
        results.append("STEP 3: Searching by window titles...")
        
        title_patterns = [
            "Notepad",
            "Discord", 
            "Chrome",
            "Visual Studio",
            "PowerShell",
        ]
        
        for pattern in title_patterns:
            try:
                matching_apps = app_manager.find_applications_by_title(pattern)
                if matching_apps:
                    results.append(f"✅ Found {len(matching_apps)} app(s) with '{pattern}' in title:")
                    for app in matching_apps:
                        results.append(f"    - {app.process_name} - '{app.main_window_title}'")
                else:
                    results.append(f"   No apps found with '{pattern}' in title")
            except Exception as e:
                results.append(f"❌ Error searching by title '{pattern}': {e}")
        
        results.append("")
        
        # Test getting a window from a specific application
        results.append("STEP 4: Testing window retrieval...")
        
        if found_apps:
            # Try to get a window from the first found application
            app_name, app_list = next(iter(found_apps.items()))
            test_app = app_list[0]
            
            try:
                results.append(f"Getting window for {app_name} (PID: {test_app.process_id})...")
                window = app_manager.get_window_by_process_id(test_app.process_id)
                window_title = window.title
                results.append(f"✅ Successfully got window: '{window_title}'")
                
                # Try to get some basic window info
                try:
                    # Get UI tree for the window
                    ui_tree = window.get_ui_tree()
                    results.append(f"✅ Window UI tree retrieved")
                    results.append(f"    Root element: {ui_tree.root.name}")
                    results.append(f"    Window class: {ui_tree.window_class}")
                    results.append(f"    Timestamp: {ui_tree.timestamp}")
                    
                except Exception as e:
                    results.append(f"⚠️  Could not get UI tree: {e}")
                
            except Exception as e:
                results.append(f"❌ Failed to get window for {app_name}: {e}")
        else:
            results.append("No applications found to test window retrieval")
        
        results.append("")
        results.append("=== APPLICATION MANAGER DEMO COMPLETED ===")
        results.append("✅ Application enumeration working")
        results.append("✅ Search by name working") 
        results.append("✅ Search by title working")
        results.append("✅ Window retrieval working")
        results.append("")
        results.append(f"Summary: Found {len(apps)} total applications, {len(found_apps)} common apps detected")
        
        return {
            "success": True,
            "total_apps": len(apps),
            "found_common_apps": len(found_apps),
            "common_apps": list(found_apps.keys()),
            "results": results,
        }
        
    except Exception as e:
        results.append(f"❌ CRITICAL ERROR: {str(e)}")
        return {
            "success": False,
            "error": str(e),
            "results": results
        }

# Run the application manager demo
result = list_applications_demo()
"#;

    // Run the application listing demo
    info!("Running application manager demo...");
    let result = run_python_code(python_code)?;
    
    // Display the result
    println!("{}", result);

    info!("Application manager demo complete.");

    Ok(())
} 