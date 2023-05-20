
use std::error::Error;
use reqwest;
use serde_json::{json, Value};


/// this doesnt work yet
pub fn get_ip_audit() -> Result<(), Box<dyn Error>> {
    let url = "https://api.abuseipdb.com/api/v2/check";

    let client = reqwest::blocking::Client::new();

    let ips = vec!["127.0.0.1", "192.168.0.1", "10.0.0.1"];
    let params = json!({
        "ip": ips,
        "maxAgeInDays": 30,
        "verbose": true,
    });

    let response = client.get(url)
        .header("Key", "...")
        .header("Accept", "application/json")
        .query(&params)
        .send()?;


    // check if the request was successful
    if response.status().is_success() {
        let json_response: Value = response.json()?;
        println!("hdfhdfh");
        for ip_result in json_response["data"].as_array().unwrap() {
            let ip = &ip_result["ipAddress"];
            let abuse_confidence_score = &ip_result["abuseConfidenceScore"];

            println!("IP: {}, Score: {}", ip, abuse_confidence_score);
        }
    } else {
        println!("Request failed with status code: {}", response.status());
    }

    Ok(())
}


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


pub fn check_if_malicious(remote_address: &str) -> (String, i16) {
    /* check if an IP corresponds to a DNS server */
    let mut marked_remote_address: String = remote_address.to_owned();
    let mut checked_ip_status: i16 = 0;

    // 0: nothing checked
    // 1: succesfully checked

    if remote_address == "185.230.162.220" {
        marked_remote_address = format!("{} ~~malicious~~", remote_address);
        checked_ip_status = 1;
    }
    
    return (marked_remote_address, checked_ip_status);
}

