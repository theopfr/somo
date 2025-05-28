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

    (remote_address, remote_port)
}


/// Converts an array of bytes in to string character.
/// 
/// # Arguments
/// * `char_bytes`: The character encoding as an UTF-8 byte sequence.
/// 
/// # Returns
/// The string decoded from the UTF-8 byte sequence.
pub fn str_from_bytes(char_bytes: &[u8]) -> String {
    let s = std::str::from_utf8(char_bytes).expect("Invalid UTF-8 sequence");
    return s.chars().next().expect("Empty string").to_string();
}


/// creates a row which consists of empty characters to fill out the terminal width 
/// with respect to how much space each column should receive based on the content length
/// 


/// Creates a Markdown table row with just empty characters with the width of the terminal window.
/// 
/// # Argument
/// * `terminal_width`: The current width of the terminal.
/// * `max_column_spaces`: An array in which the values represent the max-width of each of the 7 Markdown table rows.
/// 
/// # Returns
/// A Markdown table row string in which each column is filled with as much empty characters needed to fit in content and as well fill out the terminal width.
pub fn fill_terminal_width(terminal_width: u16, max_column_spaces: [u16; 7]) -> String {
    let total_column_spaces: u16 = max_column_spaces.iter().sum();

    let calculate_column_width = |column_space: u16| ((column_space as f64 / total_column_spaces as f64) * (terminal_width as f64));
    let empty_character: String = str_from_bytes(&[0xE2, 0xA0, 0x80]);

    let mut row: String = String::new();
    for &max_column_space in &max_column_spaces {
        row.push_str(&format!("| {} ", empty_character.repeat(calculate_column_width(max_column_space) as usize)));
    }
    row.push_str("|\n");

    row
}


/// prints text with a green "Info" prefix
/// a new style is given to common Markdown syntax:
/// **bold** text -> bold and white
/// *italic* text -> gray and not italic
/// ~~strikeout~~ text -> green and not striked out
/// 
/// 

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
