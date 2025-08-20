use crate::utils::{pretty_print_error, pretty_print_info, pretty_print_warning};
use etcetera::{choose_base_strategy, BaseStrategy};
use std::{
    fs::{self, File},
    io::{self, BufRead, BufReader, Write},
    path::PathBuf,
};

const DEFAULT_CONFIG_CONTENT: &str = r#"# somo configuration file
# Each line is either a flag or a comment.
# Flags listed here are automatically added when running somo.
# Lines starting with '#' are ignored.

# View compact version of the table
# --compact

# Sort by a specific field (proto, local_port, remote_address, remote_port, program, pid, state)
# --sort=pid

# Only include TCP connections
# --tcp

# Only include established connections
# --established

# Only show established connections
# --established
"#;

/// Gets the somo config path inside the current OS’s default configuration directory
///
/// # Arguments
/// None
///
/// # Returns
/// The path to the '/somo/config' plaintext config file.
pub fn get_config_path() -> PathBuf {
    match choose_base_strategy() {
        Ok(strategy) => return strategy.config_dir().join("somo/config"),
        Err(err) => {
            pretty_print_error(&format!(
                "Could not determine default configuration path: {}",
                err
            ));
            std::process::exit(1);
        }
    }
}

/// Generates the somo config file.
pub fn generate_config_file() {
    let config_path = get_config_path();

    if config_path.is_file() {
        pretty_print_warning(&format!(
            "Config file already exists at {}. Overwrite? (y/N)",
            config_path.to_string_lossy()
        ));

        let _ = io::stdout().flush();
        let mut decision = String::new();
        let _ = io::stdin().read_line(&mut decision);

        if !decision.trim().eq_ignore_ascii_case("y") {
            return;
        }
    }

    if let Some(parent) = config_path.parent() {
        if let Err(err) = fs::create_dir_all(parent) {
            pretty_print_error(&format!("Could not create config directory: {}", err));
            std::process::exit(1);
        }
    }

    let file = File::create(&config_path);
    let mut file = match file {
        Ok(f) => f,
        Err(err) => {
            pretty_print_error(&format!("Could not create config file: {}", err));
            std::process::exit(1);
        }
    };

    if let Err(err) = file.write_all(DEFAULT_CONFIG_CONTENT.as_bytes()) {
        pretty_print_error(&format!("Failed to write to config file: {}", err));
        std::process::exit(1);
    }

    pretty_print_info(&format!(
        "Config file generated at {}.",
        config_path.display()
    ));
}

/// Reads the config file contents.
///
/// # Arguments
/// None
///
/// # Returns
/// A list of all flags specified in the config file (ignoring empty and comment lines).
pub fn read_config_file() -> Vec<String> {
    let mut argv: Vec<String> = vec![];

    let config_path = get_config_path();
    if !config_path.is_file() {
        return argv;
    }

    if let Ok(config_file) = File::open(config_path) {
        let reader = BufReader::new(config_file);
        for line in reader.lines().map_while(Result::ok) {
            let cur_line = line.trim();
            if cur_line.is_empty() || cur_line.starts_with('#') {
                continue;
            }
            argv.push(cur_line.to_string());
        }
    }

    argv
}

/// Merges the CLI argmuments and config file arguments together into one argv.
///
/// # Arguments
/// * `cli_args`: List of CLI arguments (first argument in the binary name, in this case `somo`)
/// * `config_args`: List of arguments specified in the config file
///
/// # Returns
/// A list of all arguments by combining the config with the CLI arguments (CLI arguments supersede config arguments).
#[inline]
pub fn merge_cli_config_args(cli_args: &Vec<String>, config_args: &Vec<String>) -> Vec<String> {
    if config_args.len() == 0 || cli_args.iter().any(|arg| arg == "--no-config") {
        return cli_args.to_vec();
    }

    // Merge config and CLI args, put CLI args at the end to supersede config args
    let mut merged_args = config_args.clone();
    merged_args.insert(0, cli_args[0].clone());
    merged_args.extend_from_slice(&cli_args[1..]);

    merged_args.to_vec()
}

#[cfg(test)]
mod tests {
    use crate::config::merge_cli_config_args;

    #[test]
    fn test_merge_cli_config_args() {
        let config_args = vec![
            "--compact".to_owned(),
            "--sort=pid".to_owned(),
            "-t".to_owned(),
        ];
        let cli_args = vec![
            "somo".to_owned(),
            "-l".to_owned(),
            "--sort=local_port".to_owned(),
        ];

        let merged_args = merge_cli_config_args(&cli_args, &config_args);
        assert_eq!(
            merged_args,
            vec![
                "somo",
                "--compact",
                "--sort=pid",
                "-t",
                "-l",
                "--sort=local_port"
            ]
        );
    }

    #[test]
    fn test_merge_cli_config_args_no_config() {
        let config_args = vec![
            "--compact".to_owned(),
            "--sort=pid".to_owned(),
            "-t".to_owned(),
        ];
        let cli_args = vec![
            "somo".to_owned(),
            "-l".to_owned(),
            "--sort=local_port".to_owned(),
            "--no-config".to_owned(),
        ];

        let merged_args = merge_cli_config_args(&cli_args, &config_args);
        assert_eq!(
            merged_args,
            vec!["somo", "-l", "--sort=local_port", "--no-config"]
        );
    }
}
