use crate::soutln;

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
