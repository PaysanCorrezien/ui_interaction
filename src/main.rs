mod core;
mod python_bindings;
mod script_runner;

use anyhow::Result;

fn main() -> Result<()> {
    // Initialize Python
    pyo3::prepare_freethreaded_python();

    // Run all scripts in the scripts directory
    let scripts_dir = std::env::current_dir()?.join("scripts");
    script_runner::run_all_scripts(&scripts_dir)
} 