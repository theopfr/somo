
pub fn check_if_known(remote_ip: &str) -> String {
    /* check if an IP corresponds to a DNS server */

    if remote_ip == "0.0.0.0" || remote_ip == "[::]" {
        return format!("*{}*", remote_ip.to_string());
    }
    else if remote_ip == "127.0.0.1" || remote_ip == "[::1]" {
        return format!("{} *localhost*", remote_ip.to_string());
    }
    return remote_ip.to_string();
}


pub fn check_if_malicious(remote_ip: &str) -> (bool, i16) {
    /* check if an IP corresponds to a DNS server */
    let mut malicious: bool = false;
    let mut checked_ip_status: i16 = 0;

    if remote_ip == "185.230.162.220" {
        malicious = true;
        checked_ip_status = 1;
    }
    
    return (malicious, checked_ip_status);
}

