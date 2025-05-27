# UI Automation Library

A powerful Rust library for desktop application UI automation with Python bindings. This library provides comprehensive tools for discovering applications, controlling windows, and interacting with UI elements.

## 🚀 Features

- **Cross-platform Support**: Full Windows support, Linux planned
- **Application Discovery**: Find and manage running applications by name, title, or process ID
- **Window Control**: Activate, focus, and manage window states
- **Element Interaction**: Click, type, and manipulate UI elements (buttons, text fields, etc.)
- **UI Tree Analysis**: Capture and analyze complete UI hierarchies
- **Python Bindings**: Full Python API with comprehensive documentation
- **Query System**: Flexible element finding with complex queries

## 📋 Quick Start

### Rust Usage

```rust
use uia_interaction::factory::UIAutomationFactory;
use uia_interaction::core::UIQuery;

// Create automation instance
let automation = UIAutomationFactory::new()?;

// Get the active window
let window = automation.get_active_window()?;
println!("Active window: {}", window.get_title()?);

// Find and click a button
let buttons = window.find_elements(&UIQuery::ByType("Button".to_string()))?;
if let Some(button) = buttons.first() {
    button.click()?;
}

// Find a text field and set content
let text_fields = window.find_elements(&UIQuery::ByType("Edit".to_string()))?;
if let Some(text_field) = text_fields.first() {
    text_field.set_text("Hello, World!")?;
}
```

### Python Usage

```python
import uia_interaction

# Create automation instances
automation = uia_interaction.PyAutomation()
app_manager = uia_interaction.PyApplicationManager()

# Find and interact with applications
notepad_apps = app_manager.find_applications_by_name("notepad.exe")
if notepad_apps:
    window = app_manager.get_window_by_process_id(notepad_apps[0].process_id)
    window.activate()
    
    # Find text area and set content
    text_areas = window.find_elements(uia_interaction.PyUIQuery.by_type("Edit"))
    if text_areas:
        text_areas[0].set_text("Hello from Python!")
```

## 🛠️ Building

### Prerequisites

- **Rust**: Install from [rustup.rs](https://rustup.rs/)
- **Python** (for Python bindings): Python 3.7+

### Build the Library

```bash
# Clone the repository
git clone https://github.com/paysancorrezien/ui_interaction.git
cd ui_interaction

# Build the Rust library
cargo build --release

# Run tests
cargo test

# Build with Python bindings
cargo build --release --features python
```

### Running Examples

```bash
# Run basic automation example
cargo run --example main

# Run application discovery example
cargo run --example list_applications_demo

# Run Vesktop automation example
cargo run --example vesktop_demo
```

## 📚 Documentation

### Rust Documentation

Generate and view the Rust documentation:

```bash
cargo doc --open
```

### Python Documentation

We provide comprehensive Python documentation:

- **[Python API Reference](docs/python_api.md)** - Complete API documentation
- **[Python Usage Guide](docs/python_usage.md)** - Getting started and examples

To regenerate Python documentation:

```bash
python scripts/generate_python_docs.py
```

## 🏗️ Architecture

### Core Components

- **`core`** - Core traits and abstractions for UI automation
- **`factory`** - Platform-specific factory functions
- **`platform`** - Platform-specific implementations
  - `windows` - Windows UI Automation implementation
  - `linux` - Linux AT-SPI implementation (planned)
- **`python_bindings`** - PyO3-based Python bindings

### Key Traits

- **`UIAutomation`** - Main automation interface
- **`Window`** - Window interaction and management
- **`UIElement`** - UI element interaction
- **`ApplicationManager`** - Application discovery and management

## 🔧 Application Management

The library provides powerful application discovery and management:

```rust
use uia_interaction::factory::ApplicationManagerFactory;

let app_manager = ApplicationManagerFactory::new()?;

// Find all running applications
let apps = app_manager.get_all_applications()?;

// Find specific applications
let chrome_apps = app_manager.find_applications_by_name("chrome")?;
let discord_apps = app_manager.find_applications_by_title("Discord")?;

// Get window from application
let window = app_manager.get_window_by_process_id(chrome_apps[0].process_id)?;
window.activate()?;
```

## 🎯 Element Finding

Advanced element finding with flexible queries:

```rust
use uia_interaction::core::UIQuery;

// Simple queries
let buttons = window.find_elements(&UIQuery::ByType("Button".to_string()))?;
let save_btn = window.find_elements(&UIQuery::ByName("Save".to_string()))?;

// Complex queries
let enabled_buttons = window.find_elements(&UIQuery::And(vec![
    UIQuery::ByType("Button".to_string()),
    UIQuery::ByProperty("IsEnabled".to_string(), "True".to_string())
]))?;

// Multiple criteria
let text_inputs = window.find_elements(&UIQuery::Or(vec![
    UIQuery::ByType("Edit".to_string()),
    UIQuery::ByType("Document".to_string())
]))?;
```

## 🌐 Platform Support

### Windows
- ✅ Full support via Windows UI Automation API
- ✅ All features available
- ✅ Tested on Windows 10/11

### Linux
- 🔄 Planned support via AT-SPI
- 🔄 Implementation in progress

### macOS
- ❌ Not yet supported
- 🔮 May be added in future versions


## 📝 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🔗 Related Projects

- [Windows UI Automation](https://docs.microsoft.com/en-us/windows/win32/winauto/entry-uiauto-win32) - The underlying Windows API
- [Windows UI Automation rust base library](https://github.com/leexgone/uiautomation-rs) - The base library for this project
- [PyO3](https://pyo3.rs/) - Python bindings for Rust
- [AT-SPI](https://www.freedesktop.org/wiki/Accessibility/AT-SPI2/) - Linux accessibility API

## 📊 Examples

The `examples/` directory contains practical examples:

- **`main.rs`** - Basic window and element interaction
- **`list_applications_demo.rs`** - Application discovery and enumeration
- **`vesktop_demo.rs`** - Real-world Vesktop/Discord automation

## 📈 Roadmap

- [ ] Python package
- [ ] Linux AT-SPI implementation ( or the new accessibility API for linux)
- [ ] Enhanced element selection strategies
- [ ] Browser automation integration
