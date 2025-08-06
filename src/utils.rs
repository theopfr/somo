use crate::{schemas::AddressType, soutln};

/// Wraps the input text in ANSI escape codes to print it in red.
fn red_text(text: &str) -> String {
    format!("\x1b[1;31m{text}\x1b[0m")
}

/// Wraps the input text in ANSI escape codes to print it in cyan.
fn cyan_text(text: &str) -> String {
    format!("\x1B[36m{text}\x1B[0m")
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

/// Prints out formatted text starting with a cyan "Info:" prefix.
///
/// # Arguments
/// * `text`: The text to print to the console.
///
/// # Returns
/// None
pub fn pretty_print_info(text: &str) {
    soutln!(
        "{}",
        bold_text(&format!("{} {}", cyan_text("Info:"), bold_text(text)))
    );
}

/// Prints out formatted text starting with a red "Error:" prefix.
///
/// # Arguments
/// * `text`: The text to print to the console.
///
/// # Returns
/// None
pub fn pretty_print_error(text: &str) {
    soutln!(
        "{}",
        bold_text(&format!("{} {}", red_text("Error:"), bold_text(text)))
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
