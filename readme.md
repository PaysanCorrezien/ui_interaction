# UIA Automation Library

A basic helper library for automating applcations interactions.
This library provides a Rust/Python interface to the Windows UI Automation framework, allowing for programmatic interaction with UI elements. It includes functionality for text handling, element manipulation, and UI navigation and process querying.
It expose a hight level api, with a backend that is only implemented for windows as of now which is built on top of the `uia_interaction` crate.

## TODO

### High Priority

- [ ] Fix the window detection via a fallback to windows.rs
- [ ] Replace fixed timing delays with adaptive waiting mechanisms
  - Implement polling with configurable timeouts
  - Add exponential backoff for retry operations
  - Create a wait utility that checks for specific UI conditions

- [ ] Expand UI element interaction

  - Implement comprehensive focus management


# Project Structure
src/
├── lib.rs
├── core.rs               # Traits + shared structs
├── platform/
│   ├── mod.rs
│   ├── windows/*        # UIA impl
│   └── linux/*          # AT-SPI impl
├── python/
│   └── script_context.rs # pyo3 bindings

# UI Interaction Library

A Rust library for UI automation on Windows using Microsoft UI Automation.

## Recent API Changes

The API has been refactored to be clearer about what each method returns:

### New Methods (Recommended)
- `automation.active_window()` - Gets the currently active (foreground) top-level application window
- `automation.window_containing_focus()` - Gets the window that contains the currently focused element (might be a child window or dialog)
- `automation.focused_element()` - Gets the currently focused element (the element with keyboard focus)

### Deprecated Methods
- `automation.focused_window()` - **DEPRECATED**: Use `active_window()` instead

### Python API
```python
# Get the top-level application window
window = automation.active_window()

# Get the window containing the focused element
focus_window = automation.window_containing_focus()

# Get the currently focused element
element = automation.focused_element()

# Get the complete UI tree
tree = window.get_ui_tree()

# Find elements using queries
from uia_interaction import PyUIQuery
query = PyUIQuery.by_name("File")
elements = window.find_elements(query)
```

## Example Workflow

The updated example demonstrates a complete workflow:
1. Open the File menu
2. Click "New" to create a new document/tab
3. Insert placeholder text into the focused text area

Run the example with:
```bash
cargo run --example minimalpython
```

Make sure to have Notepad or another text editor open and focused before running the example.
