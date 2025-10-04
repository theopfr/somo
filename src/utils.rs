use crate::{schemas::AddressType, sout, soutln};
use std::env;
use std::io::{IsTerminal, Write};
use std::os::fd::AsRawFd;
use std::process::{Command, Stdio};

/// Wraps the input text in ANSI escape codes to print it in red.
fn red_text(text: &str) -> String {
    format!("\x1b[1;31m{text}\x1b[0m")
}

/// Wraps the input text in ANSI escape codes to print it in cyan.
fn cyan_text(text: &str) -> String {
    format!("\x1B[36m{text}\x1B[0m")
}

/// Wraps the input text in ANSI escape codes to print it in yellow.
fn yellow_text(text: &str) -> String {
    format!("\x1b[1;33m{text}\x1b[0m")
}

/// Wraps the input text in ANSI escape codes to print it in bold.
fn bold_text(text: &str) -> String {
    format!("\x1B[1m{text}\x1B[0m")
}

/// Marks localhost and unspecified IP addresses (i.e., 0.0.0.0) using Markdown formatting
///
/// * `address_type` == Localhost -> *italic* + "localhost"
/// * `address_type` == Unspecified -> *italic*
/// * `address_type` == Extern -> not formatted
///
/// # Arguments
/// * `remote_address`: The remote address.
/// * `address_type`: The address type as an AddressType enum.
///
/// # Example
/// ```
/// let address = "127.0.0.1".to_string();
/// let address_type = AddressType::Localhost;
/// let formatted = format_known_address(&address, &address_type);
/// assert_eq!(formatted, "*127.0.0.1 localhost*");
/// ```
///
/// # Returns
/// A Markdown formatted string based on the address-type.
pub fn format_known_address(remote_address: &str, address_type: &AddressType) -> String {
    match address_type {
        AddressType::Unspecified => {
            format!("*{remote_address}*")
        }
        AddressType::Localhost => {
            format!("*{remote_address} localhost*")
        }
        AddressType::Extern => remote_address.to_string(),
    }
}

/// Creates a formatted text starting with a cyan "Info:" prefix.
///
/// # Arguments
/// * `text`: The text to print to the console.
///
/// # Returns
/// None
pub fn render_info_line(text: &str) -> String {
    bold_text(&format!("{} {}", cyan_text("Info:"), text))
}

/// Prints out text starting with a cyan "Info:" prefix.
///
/// # Arguments
/// * `text`: The text to print to the console.
///
/// # Returns
/// None
pub fn pretty_print_info(text: &str) {
    soutln!("{}", render_info_line(text));
}

/// Prints out a formatted text starting with a red "Error:" prefix.
///
/// # Arguments
/// * `text`: The text to print to the console.
///
/// # Returns
/// None
pub fn pretty_print_error(text: &str) {
    soutln!("{}", bold_text(&format!("{} {}", red_text("Error:"), text)));
}

/// Prints out a formatted text starting with a yellow "Warning:" prefix.
///
/// # Arguments
/// * `text`: The text to print to the console.
///
/// # Returns
/// None
pub fn pretty_print_warning(text: &str) {
    soutln!(
        "{}",
        bold_text(&format!("{} {}", yellow_text("Warning:"), text))
    );
}

/// Prints a syntax error message with an error preamble, the error line, and
/// a caret pointing to the error column.
///
/// # Arguments
/// * `preamble`: The error message or description to print.
/// * `text`: The full source text containing the error.
/// * `line`: The line number (starting at 1) where the error occurred.
/// * `column`: The column number (starting at 1) where the error occurred.
///
/// # Returns
/// None
///
/// # Example
/// ```
/// let code = "let x = 5;\nlet y = 1o;";
/// pretty_print_syntax_error("Unexpected token", code, 2, 9);
/// ```
pub fn pretty_print_syntax_error(preamble: &str, text: &str, line: usize, column: usize) {
    let erronous_line: &str = text.lines().nth(line - 1).unwrap_or(text);
    let line_pointer = "└─>";

    pretty_print_error(preamble);
    soutln!("  {}", red_text("│"));
    soutln!("  {}  {}", red_text(line_pointer), erronous_line);
    soutln!(
        "  {}  {}",
        " ".repeat(line_pointer.chars().count() + column - 1),
        red_text("^")
    );
}

/// Returns true if stdout is a TTY.
pub fn is_stdout_tty() -> bool {
    std::io::stdout().is_terminal()
}

/// Returns the terminal height in rows if stdout is a TTY.
///
/// Note:
/// - We could use a crate like `terminal_size` to get the terminal dimensions, but we
///   keep this lightweight direct `ioctl(TIOCGWINSZ)` call to avoid an extra dependency
///   and because we already use libc here for this specific purpose.
pub fn terminal_rows() -> Option<usize> {
    if !is_stdout_tty() {
        return None;
    }
    unsafe {
        let fd = std::io::stdout().as_raw_fd();
        let mut ws: libc::winsize = std::mem::zeroed();
        if libc::ioctl(fd, libc::TIOCGWINSZ, &mut ws) == 0 && ws.ws_row > 0 {
            return Some(ws.ws_row as usize);
        }
    }
    None
}

/// Write the given text to a pager as defined in an env. variable (falls back to `less -R`).
///
/// # Arguments
/// * `text`: The text write to the pager.
///
/// # Returns
/// An empty `Result` to signal if writing to the pager was successfullq or not.
fn write_to_pager(text: &str) -> std::io::Result<()> {
    // Prefer `SOMO_PAGER` over `PAGER` to allow somo-specific configuration.
    let pager_env = env::var("SOMO_PAGER").or_else(|_| env::var("PAGER")).ok();
    let mut parts: Vec<String> = match pager_env {
        Some(p) if !p.trim().is_empty() => {
            // Split on whitespace for simple commands like "less -R"
            p.split_whitespace().map(|s| s.to_string()).collect()
        }
        _ => vec!["less".into(), "-R".into()],
    };

    let program = parts.remove(0);

    let mut child = Command::new(program)
        .args(parts)
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;

    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(text.as_bytes());
    }
    let _ = child.wait();
    Ok(())
}

/// If the text exceeds the terminal height, open a pager, otherwise print to console.
///
/// # Arguments
/// * `text`: The text to print to console or write to pager.
/// * `no_paging`: If `true`, never page.
///
/// # Returns
/// None
pub fn page_or_print(text: &str, no_paging: bool) {
    let should_page = !no_paging
        && terminal_rows()
            .map(|rows| text.lines().count() >= rows)
            .unwrap_or(false);

    if should_page && write_to_pager(text).is_ok() {
        return;
    }

    soutln!("{}", text);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_known_address_localhost() {
        let addr = "127.0.0.1".to_string();
        let result = format_known_address(&addr, &AddressType::Localhost);
        assert_eq!(result, "*127.0.0.1 localhost*");
    }

    #[test]
    fn test_format_known_address_unspecified() {
        let addr = "0.0.0.0".to_string();
        let result = format_known_address(&addr, &AddressType::Unspecified);
        assert_eq!(result, "*0.0.0.0*");
    }

    #[test]
    fn test_format_known_address_extern() {
        let addr = "123.123.123".to_string();
        let result = format_known_address(&addr, &AddressType::Extern);
        assert_eq!(result, "123.123.123");
    }
}
