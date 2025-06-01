use termimad::crossterm::style::{Color::*, Attribute::*};
use termimad::*;


/// Splits a string combined of an IP address and port with a ":" delimiter into two parts.
/// 
/// # Arguments
/// * `address`: The combination of address and port joined by a ":", e.g "127.0.0.1:5432"
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
/// # Retunrs
/// If the string can be successfully split it will return a tuple containing the address and the port, if not `None`.
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
    let remote_address: String;
    let remote_port: String;
    if let Some((part1, part2)) = split_address(address) {
        remote_address = String::from(part1);
        remote_port = String::from(part2);
    } else {
        remote_address = String::from(address);
        remote_port = "-".to_string();
    }

    return (remote_address, remote_port);
}


/// Converts an array of bytes in to string character.
/// 
/// # Arguments
/// * `char_bytes`: The character encoding as an UTF-8 byte sequence.
/// 
/// # Returns
/// The string decoded from the UTF-8 byte sequence.
pub fn str_from_bytes(char_bytes: &[u8]) -> String {
    let str = std::str::from_utf8(char_bytes).expect("Invalid UTF-8 sequence");
    return str.chars().next().expect("Empty string").to_string();
}


/// Prints out Markdown formatted text using a custom appearence / termimad "skin".
/// 
/// # Appearence
/// * **bold** text -> bold and white
/// * *italic* text -> not italic and gray
/// * ~~strikeout~~ text -> not striked out and green
/// 
/// # Arguments
/// * `text`: The text to print to the console.
/// 
/// # Returns
/// None
pub fn pretty_print_info(text: &str) {
    let mut skin = MadSkin::default();
    skin.bold.set_fg(White);
    skin.italic = CompoundStyle::new(Some(gray(11)), None, Encircled.into());
    skin.strikeout = CompoundStyle::new(Some(DarkGreen), None, Encircled.into());

    let markdown: String = format!("~~Info~~: *{}*", text);
    print!("{}", skin.term_text(&markdown));
}

/// Prints out Markdown formatted text using a custom appearence / termimad "skin".
/// 
/// # Appearence
/// * **bold** text -> bold and white
/// * *italic* text -> not italic and gray
/// * ~~strikeout~~ text -> not striked out and red
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
    print!("{}", skin.term_text(&markdown));
}
