use crate::soutln;

/// Splits a string combined of an IP address and port with a ":" delimiter into two parts.
///
/// # Arguments
/// * `address`: The combination of address and port joined by a ":", e.g. "127.0.0.1:5432"
///
/// # Example
/// ```
/// let address_port_1 = "127.0.0.1:5432".to_string();
/// assert_eq!(split_address(address_port_1), Some(("5432", "127.0.0.1")));
///
/// let address_port_2 = "fails.com".to_string();
/// assert_eq!(split_address(address_port_2), None);
/// ```
///
/// # Returns
/// If the string can be successfully split,
/// it will return a tuple containing the address and the port, if not `None`.
pub fn split_address(address: &str) -> Option<(&str, &str)> {
    static DELIMITER: &str = ":";

    let mut address_parts = address.rsplitn(2, DELIMITER);
    match (address_parts.next(), address_parts.next()) {
        (Some(first), Some(second)) => Some((second, first)),
        _ => None,
    }
}

/// Handles the output of the `split_address` function by replacing the port with a "-" if the string couldn't be split.
/// ###### TODO: maybe combine it with the `split_address` function.
///
/// # Arguments
/// * `address`: The address-port combination which should be split.
///
/// # Example
/// ```
/// let address_port_1 = "127.0.0.1:5432".to_string();
/// assert_eq!(get_address_parts(address_port_1), ("5432", "127.0.0.1"));
///
/// let address_port_2 = "fails.com".to_string();
/// assert_eq!(get_address_parts(address_port_1), ("-", "127.0.0.1"));
/// ```
///
/// # Returns
/// A tuple containing the address and port or just the address and a "-" if there wasn't a port.
pub fn get_address_parts(address: &str) -> (String, String) {
    split_address(address)
        .map(|(a, p)| (a.to_string(), p.to_string()))
        .unwrap_or((address.to_string(), "-".to_string()))
}

/// Wraps the input text in ANSI escape codes to print it in red.
fn red_text(text: &str) -> String {
    format!("\x1b[1;31m{}\x1b[0m", text)
}

/// Wraps the input text in ANSI escape codes to print it in cyan.
fn cyan_text(text: &str) -> String {
    format!("\x1B[36m{}\x1B[0m", text)
}

/// Wraps the input text in ANSI escape codes to print it in bold.
fn bold_text(text: &str) -> String {
    format!("\x1B[1m{}\x1B[0m", text)
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
    fn test_split_address_valid() {
        let addr = "127.0.0.1:5432";
        assert_eq!(split_address(addr), Some(("127.0.0.1", "5432")));

        let addr = "[::1]:8080";
        assert_eq!(split_address(addr), Some(("[::1]", "8080")));
    }

    #[test]
    fn test_split_address_invalid() {
        let addr = "localhost";
        assert_eq!(split_address(addr), None);
        let addr = "192.168.0.1";
        assert_eq!(split_address(addr), None);
    }

    #[test]
    fn test_get_address_parts_valid() {
        let addr = "192.168.0.1:80";
        let (address, port) = get_address_parts(addr);
        assert_eq!(address, "192.168.0.1");
        assert_eq!(port, "80");
    }

    #[test]
    fn test_get_address_parts_invalid() {
        let addr = "example.com";
        let (address, port) = get_address_parts(addr);
        assert_eq!(address, "example.com");
        assert_eq!(port, "-");
    }
}
