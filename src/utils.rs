use std::io;
use std::io::Write;
use std::str::FromStr;

pub fn get_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::Write::flush(&mut io::stdout()).expect("Failed to flush stdout");
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    input.trim().to_string()
}

pub fn get_parsed_input<T: FromStr>(prompt: &str) -> T {
    loop {
        let input = get_input(prompt);
        match input.parse() {
            Ok(value) => return value,
            Err(_) => println!("Invalid input. Please try again."),
        }
    }
}

pub fn get_parsed_input_with_default<T: FromStr + Clone>(prompt: &str, default: T) -> T {
    loop {
        let input = get_input(prompt);
        if input.is_empty() {
            return default.clone();
        }
        match input.parse() {
            Ok(value) => return value,
            Err(_) => println!("Invalid input. Please try again."),
        }
    }
}

/// Write a potential profile (depth,potential pairs) to a CSV file.
///
/// The output file will be created or overwritten. The first line is the header
/// `depth,potential`, followed by one row per pair.
///
/// # Arguments
///
/// * `path` - filesystem path for the CSV output
/// * `profile` - slice of `(depth, potential, electron_density, ionized_donor_concentration)` tuples
pub fn write_potential_profile_csv(
    path: &str,
    profile: &[(f64, f64, f64, f64)],
) -> std::io::Result<()> {
    let mut file = std::fs::File::create(path)?;
    writeln!(
        file,
        "depth,potential,electron_density,ionized_donor_concentration"
    )?;
    for &(depth, pot, electron_density, ionized_donor_concentration) in profile {
        writeln!(
            file,
            "{},{},{},{}",
            depth, pot, electron_density, ionized_donor_concentration
        )?;
    }
    Ok(())
}

/// Anti traversal filename
///
/// # Arguments
///
/// - `filename` (`&str`) - filename to validate. Must not contain path separators (`/` or `\`) or `..` segments.
///
/// # Returns
///
/// - `Option<String>` - `Some(filename)` if valid, `None` if invalid (contains path separators or `..`).
///
/// # Examples
///
/// ```
/// use crate::...;
///
/// let filename = match anti_traversal_filename(&filename) {
///     Some(name) => name,
///     None => {
///         anyhow::bail!("Invalid filename: must not contain path separators or '..'.");
///     }
/// };
/// ```
pub fn anti_traversal_filename(filename: &str) -> Option<String> {
    // Disallow path separators and parent directory references
    if filename.contains('/') || filename.contains('\\') || filename.contains("..") {
        None
    } else {
        Some(filename.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn test_write_potential_profile_csv() {
        let tmp_dir = std::env::temp_dir();
        let mut file_path = PathBuf::from(&tmp_dir);
        file_path.push("test_profile.csv");
        let profile = vec![(0.0, 1.0, 2.0, 3.0), (2.5, -0.5, 4.0, 5.0)];
        let path_str = file_path.to_str().unwrap();

        // ensure previous file is removed
        let _ = fs::remove_file(path_str);

        write_potential_profile_csv(path_str, &profile).expect("failed to write csv");

        let contents = fs::read_to_string(path_str).expect("failed to read csv");
        assert!(
            contents.starts_with("depth,potential,electron_density,ionized_donor_concentration")
        );
        assert!(contents.contains("0,1,2,3"));
        // f64 formatting drops trailing .0; expect minimal representation
        assert!(contents.contains("2.5,-0.5,4,5"));

        // cleanup
        let _ = fs::remove_file(path_str);
    }

    #[test]
    fn test_anti_traversal_filename() {
        assert!(anti_traversal_filename("test.csv").is_some());
        assert!(anti_traversal_filename("test/path.csv").is_none());
        assert!(anti_traversal_filename("test\\path.csv").is_none());
        assert!(anti_traversal_filename("test/../path.csv").is_none());
        assert!(anti_traversal_filename("test\\..\\path.csv").is_none());
    }
}
