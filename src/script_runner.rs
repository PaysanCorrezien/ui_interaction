use std::path::Path;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyModule};
use anyhow::Result;
use crate::python_bindings::PyAutomation;

/// Executes a Python script with the automation object injected into its globals
pub fn run_script(path: &Path) -> Result<()> {
    let code = std::fs::read_to_string(path)?;
    
    Python::with_gil(|py| {
        let globals = PyDict::new_bound(py);

        // Create and inject the automation object
        let automation = Py::new(py, PyAutomation::new()?)?;
        globals.set_item("automation", automation)?;

        // Load the script as a module
        let module = PyModule::from_code_bound(py, &code, path.to_str().unwrap(), "user_script")?;

        // Validate that the required run() function exists
        if !module.hasattr("run")? {
            anyhow::bail!("Script '{}' missing required function `run()`", path.display());
        }

        // Execute the script
        module.getattr("run")?.call0()?;
        Ok(())
    })
}

/// Runs all Python scripts in a directory
pub fn run_all_scripts(dir: &Path) -> Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        if path.extension().and_then(|e| e.to_str()) == Some("py") {
            println!("▶ Running {}", path.display());
            run_script(&path)?;
        }
    }
    Ok(())
} 