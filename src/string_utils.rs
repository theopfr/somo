
/// splits remote address and port at ":"
pub fn split_address(address: &str) -> Option<(&str, &str)> {
    static DELIMITER: &str = ":";

    let mut address_parts = address.rsplitn(2, DELIMITER);
    match (address_parts.next(), address_parts.next()) {
        (Some(first), Some(second)) => Some((second, first)),
        _ => None,
    }
}


/// gets remote address and port from a composite string: "<remote-address>:<port>" if possible
pub fn get_address_parts(address: &str) -> (String, String) {
    let remote_address: String;
    let remote_port: String;
    if let Some((part1, part2)) = split_address(&address) {
        remote_address = String::from(part1);
        remote_port = String::from(part2);
    } else {
        remote_address = String::from(address);
        remote_port = "-".to_string();
    }

    return (remote_address, remote_port);
}


/// converts utf-8 to char
pub fn str_from_bytes(char_bytes: &[u8]) -> String {
    let s = std::str::from_utf8(&char_bytes).expect("Invalid UTF-8 sequence");
    return s.chars().next().expect("Empty string").to_string();
}


/// creates a row which consists of empty characters to fill out the terminal width 
/// with respect to how much space each column should receive based on the content length
pub fn fill_terminal_width(terminal_width: u16, max_column_spaces: [u16; 7]) -> String {
    let total_column_spaces: u16 = max_column_spaces.iter().sum();

    let calculate_column_width = |column_space: u16| ((column_space as f64 / total_column_spaces as f64) * (terminal_width as f64));
    let empty_character: String = str_from_bytes(&[0xE2, 0xA0, 0x80]);

    let mut row: String = format!("");
    for &max_column_space in &max_column_spaces {
        row.push_str(&format!("| {} ", empty_character.repeat(calculate_column_width(max_column_space) as usize)));
    }
    row.push_str("|\n");

    return row;
}
