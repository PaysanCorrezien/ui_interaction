extern crate uia_interaction;

mod python_bindings;
mod script_runner;

use anyhow::Result;
use log::{info, error};
use uia_interaction::core::{UIAutomation, UIQuery, UITree, AppendPosition};
use uia_interaction::UIAutomationFactory;

// Function to display UI tree in a formatted way
fn display_ui_tree(tree: &UITree, indent: usize) {
    let indent_str = "  ".repeat(indent);
    info!("{}Window: {} (Class: {})", indent_str, tree.window_title, tree.window_class);
    info!("{}Timestamp: {}", indent_str, tree.timestamp);
    display_tree_node(&tree.root, indent + 1);
}

fn display_tree_node(node: &uia_interaction::core::UITreeNode, indent: usize) {
    let indent_str = "  ".repeat(indent);
    info!("{}Element: {} (Type: {})", indent_str, node.name, node.control_type);
    info!("{}Enabled: {}, Visible: {}", indent_str, node.is_enabled, node.is_visible);
    
    if let Some(bounds) = &node.bounds {
        info!("{}Bounds: ({}, {}) - ({}, {})", 
            indent_str, 
            bounds.left, bounds.top, 
            bounds.right, bounds.bottom
        );
    }

    // Display properties
    if !node.properties.is_empty() {
        info!("{}Properties:", indent_str);
        for (key, value) in &node.properties {
            info!("{}  {}: {}", indent_str, key, value);
        }
    }

    // Display children
    if !node.children.is_empty() {
        info!("{}Children:", indent_str);
        for child in &node.children {
            display_tree_node(child, indent + 1);
        }
    }
}

fn demonstrate_ui_automation(automation: &dyn UIAutomation) -> Result<()> {
    info!("Waiting for 3 seconds to switch to your target window...");
    std::thread::sleep(std::time::Duration::from_secs(3));

    // Get the focused window
    let window = automation.get_focused_window()
        .map_err(|e| anyhow::anyhow!("Failed to get focused window: {}", e))?;
    
    info!("\nFocused Window Information:");
    info!("Title: {}", window.get_title()
        .map_err(|e| anyhow::anyhow!("Failed to get window title: {}", e))?);
    info!("Class: {}", window.get_class_name()
        .map_err(|e| anyhow::anyhow!("Failed to get window class: {}", e))?);
    info!("Process: {} (PID: {})", 
        window.get_process_name()
            .map_err(|e| anyhow::anyhow!("Failed to get process name: {}", e))?,
        window.get_process_id()
            .map_err(|e| anyhow::anyhow!("Failed to get process ID: {}", e))?);
    info!("Thread ID: {}", window.get_thread_id()
        .map_err(|e| anyhow::anyhow!("Failed to get thread ID: {}", e))?);
    info!("Visible: {}", window.is_visible()
        .map_err(|e| anyhow::anyhow!("Failed to get window visibility: {}", e))?);
    info!("Minimized: {}", window.is_minimized()
        .map_err(|e| anyhow::anyhow!("Failed to get window minimized state: {}", e))?);
    info!("Maximized: {}", window.is_maximized()
        .map_err(|e| anyhow::anyhow!("Failed to get window maximized state: {}", e))?);

    // Get and display the UI tree
    info!("\nUI Tree Structure:");
    let tree = window.get_ui_tree()
        .map_err(|e| anyhow::anyhow!("Failed to get UI tree: {}", e))?;
    display_ui_tree(&tree, 0);

    // Demonstrate different types of queries
    info!("\nDemonstrating UI Queries:");

    // Example 1: Find elements by name
    let name_query = UIQuery::ByName("Button".to_string());
    let elements = window.find_elements(&name_query)
        .map_err(|e| anyhow::anyhow!("Failed to find elements by name: {}", e))?;
    info!("\nElements found by name 'Button':");
    for element in elements {
        info!("- {} ({})", 
            element.get_name()
                .map_err(|e| anyhow::anyhow!("Failed to get element name: {}", e))?,
            element.get_type()
                .map_err(|e| anyhow::anyhow!("Failed to get element type: {}", e))?);
    }

    // Example 2: Find elements by type
    let type_query = UIQuery::ByType("Edit".to_string());
    let elements = window.find_elements(&type_query)
        .map_err(|e| anyhow::anyhow!("Failed to find elements by type: {}", e))?;
    info!("\nElements found by type 'Edit':");
    for element in elements {
        info!("- {} ({})", 
            element.get_name()
                .map_err(|e| anyhow::anyhow!("Failed to get element name: {}", e))?,
            element.get_type()
                .map_err(|e| anyhow::anyhow!("Failed to get element type: {}", e))?);
    }

    // Example 3: Complex query (AND condition)
    let complex_query = UIQuery::And(vec![
        UIQuery::ByType("Edit".to_string()),
        UIQuery::ByProperty("enabled".to_string(), "true".to_string()),
    ]);
    let elements = window.find_elements(&complex_query)
        .map_err(|e| anyhow::anyhow!("Failed to find elements with complex query: {}", e))?;
    info!("\nEnabled Edit elements found:");
    for element in elements {
        info!("- {} ({})", 
            element.get_name()
                .map_err(|e| anyhow::anyhow!("Failed to get element name: {}", e))?,
            element.get_type()
                .map_err(|e| anyhow::anyhow!("Failed to get element type: {}", e))?);
    }

    // Get the focused element and demonstrate text manipulation
    info!("\nDemonstrating Text Manipulation:");
    let focused_element = automation.get_focused_element()
        .map_err(|e| anyhow::anyhow!("Failed to get focused element: {}", e))?;
    
    info!("Focused element: {} ({})", 
        focused_element.get_name()
            .map_err(|e| anyhow::anyhow!("Failed to get element name: {}", e))?,
        focused_element.get_type()
            .map_err(|e| anyhow::anyhow!("Failed to get element type: {}", e))?);
    
    // Get current text
    let current_text = focused_element.get_text()
        .map_err(|e| anyhow::anyhow!("Failed to get element text: {}", e))?;
    info!("Current text: {}", current_text);
    
    // Set new text
    let new_text = "This is a test\nWith multiple lines\nTo verify multiline handling";
    info!("Setting text: {}", new_text);
    focused_element.set_text(new_text)
        .map_err(|e| anyhow::anyhow!("Failed to set element text: {}", e))?;
    info!("Text set successfully");
    
    // Append text at end of text (default)
    let append_text = " - Appended text";
    info!("Appending text at end of text: {}", append_text);
    focused_element.append_text(append_text, AppendPosition::EndOfText)
        .map_err(|e| anyhow::anyhow!("Failed to append text at end of text: {}", e))?;

    // Append text at end of current line
    let append_text_line = " [EOL]";
    info!("Appending text at end of line: {}", append_text_line);
    focused_element.append_text(append_text_line, AppendPosition::EndOfLine)
        .map_err(|e| anyhow::anyhow!("Failed to append text at end of line: {}", e))?;

    // Append text at current cursor position
    let append_text_cursor = " [CURSOR]";
    info!("Appending text at current cursor: {}", append_text_cursor);
    focused_element.append_text(append_text_cursor, AppendPosition::CurrentCursor)
        .map_err(|e| anyhow::anyhow!("Failed to append text at cursor: {}", e))?;
    
    // Get final text
    let final_text = focused_element.get_text()
        .map_err(|e| anyhow::anyhow!("Failed to get final text: {}", e))?;
    info!("Final text content: {}", final_text);

    Ok(())
}

fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();

    // Initialize Python
    pyo3::prepare_freethreaded_python();

    // Get the UI automation instance
    let automation = UIAutomationFactory::new()
        .map_err(|e| anyhow::anyhow!("Failed to create UI automation instance: {}", e))?;

    // Demonstrate UI automation capabilities
    if let Err(e) = demonstrate_ui_automation(automation.as_ref()) {
        error!("Error demonstrating UI automation: {}", e);
    }

    // Run all scripts in the scripts directory
    let scripts_dir = std::env::current_dir()?.join("scripts");
    script_runner::run_all_scripts(&scripts_dir)
} 