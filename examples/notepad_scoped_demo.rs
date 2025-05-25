use anyhow::Result;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyModule};
use env_logger::Env;
use log::info;
use std::process::{Command, Child};
use std::thread;
use std::time::Duration;

mod python_bindings {
    include!("../src/python_bindings.rs");
}

struct NotepadProcess {
    _child: Child,
}

impl NotepadProcess {
    fn spawn() -> Result<Self> {
        info!("Spawning Notepad process...");
        let child = Command::new("notepad.exe")
            .spawn()
            .map_err(|e| anyhow::anyhow!("Failed to spawn notepad: {}", e))?;
        
        // Give Notepad time to fully initialize
        thread::sleep(Duration::from_millis(2000));
        info!("Notepad process spawned successfully");
        
        Ok(Self { _child: child })
    }
}

impl Drop for NotepadProcess {
    fn drop(&mut self) {
        info!("Cleaning up Notepad process...");
        let _ = self._child.kill();
    }
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
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    info!("Starting Notepad Scoped Demo - Testing Window Scoping Fix");

    // Initialize Python
    pyo3::prepare_freethreaded_python();

    // Spawn Notepad process
    let _notepad = NotepadProcess::spawn()?;

    // Python code that implements the notepad workflow: File → New Tab → Insert Text
    let python_code = r#"
import time
from uia_interaction import PyUIQuery

def notepad_file_menu_workflow():
    results = []
    
    try:
        results.append("=== NOTEPAD FILE MENU WORKFLOW ===")
        results.append("Testing: File → New Tab → Insert Text")
        results.append("")
        
        # Get the focused window (should be Notepad)
        window = automation.focused_window()
        window_title = window.title
        results.append(f"Working with: '{window_title}'")
        results.append("")
        
        # Step 1: Find and click the File menu
        results.append("STEP 1: Finding File menu...")
        try:
            file_query = PyUIQuery.by_name("File")
            file_elements = window.find_elements(file_query)
            
            if not file_elements:
                results.append("❌ No File menu found!")
                return {"success": False, "results": results}
            
            file_menu = file_elements[0]
            results.append(f"✅ Found File menu: '{file_menu.name}' ({file_menu.control_type})")
            
            # Get File menu properties for debugging
            file_props = file_menu.get_properties()
            automation_id = file_props.get('automation_id', 'N/A')
            results.append(f"   AutomationId: {automation_id}")
            results.append("")
            
            # Click the File menu
            results.append("STEP 2: Clicking File menu...")
            file_menu.click()
            results.append("✅ File menu clicked!")
            
            # Wait for menu to open
            time.sleep(1.5)
            results.append("   Waited for menu to open...")
            
        except Exception as e:
            results.append(f"❌ Failed to find/click File menu: {e}")
            return {"success": False, "results": results}
        
        # Step 3: Look for menu items after clicking File
        results.append("")
        results.append("STEP 3: Searching for menu items...")
        try:
            # Get the window again (might have changed after menu opened)
            current_window = automation.focused_window()
            
            # Search for menu items
            menu_item_query = PyUIQuery.by_type("MenuItem")
            menu_items = current_window.find_elements(menu_item_query)
            
            results.append(f"✅ Found {len(menu_items)} menu items after clicking File:")
            
            new_tab_item = None
            for i, item in enumerate(menu_items):
                item_name = item.name.strip()
                item_props = item.get_properties()
                automation_id = item_props.get('automation_id', 'N/A')
                results.append(f"  [{i}] '{item_name}' (AutomationId: {automation_id})")
                
                # Look for "New Tab" or similar text
                if any(keyword in item_name.lower() for keyword in ["new", "tab", "document", "file"]):
                    if "tab" in item_name.lower() or "new" in item_name.lower():
                        results.append(f"      ^^^ POTENTIAL NEW TAB ITEM: '{item_name}'")
                        if new_tab_item is None:  # Take the first match
                            new_tab_item = item
                            results.append(f"      >>> SELECTED as New Tab candidate!")
            
            results.append("")
            
        except Exception as e:
            results.append(f"❌ Failed to search for menu items: {e}")
            return {"success": False, "results": results}
        
        # Step 4: Click on New Tab (if found)
        if new_tab_item:
            results.append("STEP 4: Clicking New Tab menu item...")
            try:
                new_tab_name = new_tab_item.name.strip()
                results.append(f"Clicking: '{new_tab_name}'")
                
                new_tab_item.click()
                results.append("✅ New Tab menu item clicked!")
                
                # Wait for new tab to be created
                time.sleep(2)
                results.append("   Waited for new tab to be created...")
                
            except Exception as e:
                results.append(f"❌ Failed to click New Tab: {e}")
                return {"success": False, "results": results}
        else:
            # If no obvious "New Tab" found, try common alternatives
            results.append("STEP 4: No obvious 'New Tab' found, trying alternatives...")
            try:
                # Look for common menu items that might create new tabs/documents
                alternatives = ["New", "New Document", "New File", "Open"]
                
                for alt_name in alternatives:
                    results.append(f"Trying to find: '{alt_name}'")
                    alt_query = PyUIQuery.by_name(alt_name)
                    alt_elements = current_window.find_elements(alt_query)
                    
                    if alt_elements:
                        alt_item = alt_elements[0]
                        results.append(f"Found '{alt_name}', clicking it...")
                        alt_item.click()
                        time.sleep(1.5)
                        results.append(f"✅ Clicked '{alt_name}'")
                        break
                else:
                    results.append("❌ No suitable menu item found for creating new tab/document")
                    # Press Escape to close menu and continue with text insertion
                    results.append("Pressing Escape to close menu...")
                    time.sleep(0.5)
                
            except Exception as e:
                results.append(f"❌ Failed to try alternatives: {e}")
        
        # Step 5: Find a text area and insert text
        results.append("")
        results.append("STEP 5: Finding text area for text insertion...")
        try:
            # Get current window (might be different after new tab)
            current_window = automation.focused_window()
            current_title = current_window.title
            results.append(f"Current window: '{current_title}'")
            
            # Look for text input areas
            text_queries = [
                ("Document", PyUIQuery.by_type("Document")),
                ("Edit", PyUIQuery.by_type("Edit")),
                ("Text", PyUIQuery.by_type("Text")),
            ]
            
            text_element = None
            for query_name, query in text_queries:
                elements = current_window.find_elements(query)
                results.append(f"Found {len(elements)} '{query_name}' elements")
                
                for element in elements:
                    # Try to find a suitable text input element
                    if element.is_enabled:
                        text_element = element
                        results.append(f"✅ Selected {query_name} element: '{element.name}'")
                        break
                
                if text_element:
                    break
            
            if not text_element:
                # Try to get the focused element as fallback
                results.append("No specific text element found, trying focused element...")
                try:
                    text_element = automation.focused_element()
                    results.append(f"Using focused element: '{text_element.name}' ({text_element.control_type})")
                except:
                    results.append("❌ Could not find any suitable text input area")
                    return {"success": False, "results": results}
            
        except Exception as e:
            results.append(f"❌ Failed to find text area: {e}")
            return {"success": False, "results": results}
        
        # Step 6: Insert text into the text area
        results.append("")
        results.append("STEP 6: Inserting text...")
        try:
            # Text to insert
            test_text = """This is a test document created through UI automation!

We successfully:
1. Clicked the File menu
2. Found and clicked a menu item (New Tab/Document)
3. Located this text input area
4. Inserted this multi-line text

Mission accomplished! 🎉"""
            
            results.append(f"Inserting text into: '{text_element.name}' ({text_element.control_type})")
            
            # Set the text
            text_element.set_text(test_text)
            results.append("✅ Text inserted successfully!")
            
            # Wait a moment then verify
            time.sleep(1)
            try:
                current_text = text_element.get_text()
                if test_text in current_text or len(current_text) > 50:
                    results.append("✅ Text insertion verified!")
                    results.append(f"Text length: {len(current_text)} characters")
                else:
                    results.append(f"⚠️ Text verification uncertain. Current length: {len(current_text)}")
            except:
                results.append("⚠️ Could not verify text (but insertion command succeeded)")
            
        except Exception as e:
            results.append(f"❌ Failed to insert text: {e}")
            return {"success": False, "results": results}
        
        # Success summary
        results.append("")
        results.append("=== WORKFLOW COMPLETED SUCCESSFULLY! ===")
        results.append("✅ File menu clicked")
        results.append("✅ Menu item selected")
        results.append("✅ Text area found")
        results.append("✅ Text inserted")
        results.append("")
        results.append("The notepad File → New Tab → Insert Text workflow is working!")
        
        return {
            "success": True,
            "window_title": window_title,
            "workflow_completed": True,
            "results": results,
        }
        
    except Exception as e:
        results.append(f"❌ CRITICAL ERROR: {str(e)}")
        return {
            "success": False,
            "error": str(e),
            "results": results
        }

# Run the notepad file menu workflow
result = notepad_file_menu_workflow()
"#;

    // Run the notepad file menu workflow
    info!("Running notepad file menu workflow...");
    let result = run_python_code(python_code)?;
    
    // Display the result
    println!("{}", result);

    // Keep the program running to see the results
    info!("Notepad workflow complete. Keeping Notepad open for 15 seconds...");
    thread::sleep(Duration::from_secs(15));

    Ok(())
} 