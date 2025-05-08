
# UIA Automation Library

This library provides a Rust interface to the Windows UI Automation framework, allowing for programmatic interaction with UI elements. It includes functionality for text handling, element manipulation, and UI navigation.

## TODO

### High Priority

- [ ] Fix the window detection via a fallback to windows.rs
- [ ] Replace fixed timing delays with adaptive waiting mechanisms
  - Implement polling with configurable timeouts
  - Add exponential backoff for retry operations
  - Create a wait utility that checks for specific UI conditions

### Medium Priority

- [ ] Enhance text handling capabilities
  - Develop more robust multiline text input methods
  - Improve text verification with normalization options
  - Add support for rich text and formatted content

- [ ] Expand UI element interaction

  - Implement comprehensive focus management

- [ ] Optimize performance
  - Add caching mechanisms for frequently accessed elements

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
