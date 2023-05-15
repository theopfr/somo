

pub fn split_address(address: &str) -> Option<(&str, &str)> {
    static DELIMITER: &str = ":";

    let mut address_parts = address.rsplitn(2, DELIMITER);
    match (address_parts.next(), address_parts.next()) {
        (Some(first), Some(second)) => Some((second, first)),
        _ => None,
    }
}


pub fn get_address_parts(address: &str) -> (String, String) {
    // split remote address and port
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


pub fn str_from_bytes(char_bytes: &[u8]) -> String {
    let s = std::str::from_utf8(&char_bytes).expect("Invalid UTF-8 sequence");
    return s.chars().next().expect("Empty string").to_string();
}