use crate::config::configuration_builder::ConfigurationBuilder;
use crate::constants::simulation::CONFIG_DIR;

fn list_config_files(config_dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let Ok(entries) = std::fs::read_dir(config_dir) else {
        return vec![];
    };
    let mut files: Vec<std::path::PathBuf> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("json"))
        .collect();
    files.sort();
    files
}

/// Select config source interactive of load.
///
/// # Errors
///
/// - If there is an error reading user input, an error is returned.
///
/// # Examples
///
/// ```
/// use your_crate::config::config_source::select_config_source;
/// let result = select_config_source();
/// assert!(result.is_ok());
/// ```
pub fn select_config_source() -> anyhow::Result<ConfigurationBuilder> {
    println!("Select configuration source:");
    println!("  [1] Interactive input (CLI)");
    println!("  [2] Load from config file");

    loop {
        let mut input = String::new();
        print!("Enter choice (default: 1): ");
        std::io::Write::flush(&mut std::io::stdout())?;
        if std::io::BufRead::read_line(&mut std::io::stdin().lock(), &mut input)? == 0 {
            return Err(anyhow::anyhow!("Input stream closed unexpectedly"));
        }
        match input.trim() {
            "" | "1" => return crate::tui::run_tui(),
            "2" => {
                let config_dir = std::path::Path::new(CONFIG_DIR);
                let files = list_config_files(config_dir);
                if files.is_empty() {
                    println!(
                        "No config files found in '{}'. Falling back to interactive input.",
                        config_dir.display()
                    );
                    return crate::tui::run_tui();
                }
                println!("Available config files:");
                for (i, path) in files.iter().enumerate() {
                    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    println!("  [{}] {}", i + 1, name);
                }
                loop {
                    let mut sel = String::new();
                    print!("Enter number (1-{}): ", files.len());
                    std::io::Write::flush(&mut std::io::stdout())?;
                    if std::io::BufRead::read_line(&mut std::io::stdin().lock(), &mut sel)? == 0 {
                        return Err(anyhow::anyhow!("Input stream closed unexpectedly"));
                    }
                    if let Ok(n) = sel.trim().parse::<usize>()
                        && n >= 1
                        && let Some(path) = files.get(n - 1)
                    {
                        println!("Loading config from '{}'...", path.display());
                        return ConfigurationBuilder::from_json(path);
                    }
                    println!(
                        "Invalid selection. Please enter a number between 1 and {}.",
                        files.len()
                    );
                }
            }
            _ => println!("Invalid choice. Please enter 1 or 2."),
        }
    }
}
