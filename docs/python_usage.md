# UI Automation Python Bindings - Usage Guide

This guide shows how to use the Python bindings for the UI Automation Rust library.

## Building the Library

This is a Rust library with Python bindings. To use it:

1. **Install Rust** (if not already installed):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Build the library**:
   ```bash
   cargo build --release
   ```

3. **Use from Python** (the compiled module will be available):
   ```python
   import uia_interaction
   ```

## Basic Examples

### Application Discovery

```python
import uia_interaction

# Create application manager
app_manager = uia_interaction.PyApplicationManager()

# Find all applications
apps = app_manager.get_all_applications()
print(f"Found {len(apps)} applications")

# Find specific applications
notepad_apps = app_manager.find_applications_by_name("notepad.exe")
for app in notepad_apps:
    print(f"Notepad: {app.main_window_title} (PID: {app.process_id})")
```

### Window Interaction

```python
import uia_interaction

# Create automation instance
automation = uia_interaction.PyAutomation()

# Get the active window
window = automation.active_window()
print(f"Active window: {window.title}")

# Activate a specific window
window.activate()
```

### Element Interaction

```python
import uia_interaction

automation = uia_interaction.PyAutomation()
window = automation.active_window()

# Find elements by type
buttons = window.find_elements(uia_interaction.PyUIQuery.by_type("Button"))
text_fields = window.find_elements(uia_interaction.PyUIQuery.by_type("Edit"))

# Interact with elements
if buttons:
    print(f"Found button: {buttons[0].name}")
    buttons[0].click()

if text_fields:
    text_fields[0].set_text("Hello, World!")
```

## Available Classes

- **PyAutomation**: Main UI Automation interface for Python

This class provides the primary entry point for UI automation tasks
- **PyUIElement**: Represents a UI element in an application

This class provides methods to interact with UI elements such as buttons,
text fields, menus, and other controls in desktop applications
- **PyWindow**: Represents a window in a desktop application

This class provides methods to interact with windows, find elements
within them, and control window state
- **PyUITree**: Represents a complete UI tree structure of a window

This class provides a snapshot of the entire UI hierarchy of a window,
useful for debugging, analysis, and understanding the structure of an application
- **PyUITreeNode**: Represents a node in the UI tree hierarchy

Each node represents a UI element and contains information about
its properties, state, and children
- **PyUIQuery**: Query builder for finding UI elements

This class provides methods to create queries for finding specific UI elements
based on various criteria such as name, type, properties, or combinations thereof
- **PyApplicationInfo**: Information about a running application

This class contains details about a running application process,
including its window information and process details
- **PyApplicationManager**: Manager for discovering and interacting with running applications

This class provides methods to find running applications, get their windows,
and filter them by various criteria such as process name or window title

For detailed API documentation, see [python_api.md](python_api.md)
