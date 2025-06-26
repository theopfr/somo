use crate::{sout, soutln};
use termimad::crossterm::style::{Attribute::*, Color::*};
use termimad::*;

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

/// Prints out Markdown formatted text using a custom appearance / termimad "skin".
///
/// # Appearance
/// * **bold** text -> bold and white
/// * *italic* text -> not italic and gray
/// * ~~strikeout~~ text -> not struck out and green
///
/// # Arguments
/// * `text`: The text to print to the console.
///
/// # Returns
/// None
pub fn pretty_print_info(text: &str) {
    let mut skin: MadSkin = MadSkin::default();
    skin.bold.set_fg(White);
    skin.italic = CompoundStyle::new(Some(gray(11)), None, Encircled.into());
    skin.strikeout = CompoundStyle::new(Some(Cyan), None, Encircled.into());

    let markdown: String = format!("~~Info~~: *{}*", text);
    sout!("{}", skin.term_text(&markdown));
}

/// Prints out Markdown formatted text using a custom appearance / termimad "skin".
///
/// # Appearance
/// * **bold** text -> bold and white
/// * *italic* text -> not italic and gray
/// * ~~strikeout~~ text -> not struck out and red
///
/// # Arguments
/// * `text`: The text to print to the console.
///
/// # Returns
/// None
pub fn pretty_print_error(text: &str) {
    let mut skin = MadSkin::default();
    skin.bold.set_fg(White);
    skin.italic = CompoundStyle::new(Some(gray(11)), None, Encircled.into());
    skin.strikeout = CompoundStyle::new(Some(Red), None, Encircled.into());

    let markdown: String = format!("~~Error~~: *{}*", text);
    sout!("{}", skin.term_text(&markdown));
}

// template_segment is unfortunately NOT exposed in handlebars api :)
pub fn pretty_print_syntax_error(preamble: &str, text: &str, column: usize) {
    let mut skin = MadSkin::default();
    skin.bold.set_fg(White);
    skin.italic = CompoundStyle::new(Some(gray(11)), None, Encircled.into());
    skin.strikeout = CompoundStyle::new(Some(Red), None, Encircled.into());

    let preamble_markdown: String = format!("~~Error~~: *{}*", preamble);

    let indicator: String = format!("{}{}", "─".repeat(column - 1), "┘");
    let mut syntax_skinned: String = skin.term_text("*├────> *").to_string();
    syntax_skinned.pop(); // remove trailing newline from termimad
    syntax_skinned.push_str(text);

    let indicator_markdown: String = format!("*└──────{}*", indicator);
    sout!("{}", skin.term_text(&preamble_markdown));
    soutln!("{}", syntax_skinned);
    sout!("{}", skin.term_text(&indicator_markdown));
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
