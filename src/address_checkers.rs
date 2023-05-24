
use std::{error::Error, env};
use reqwest::{self, Response};
use serde_json::{json, Value};




/// this doesnt work yet
pub fn get_ip_audit(remote_ip: &String, verbose: bool) -> Result<Option<i64>, Box<dyn Error>> {

    let abuseipdb_api_key: String = match env::var("ABUSEIPDB_API_KEY") {
        Ok(val) => val,
        Err(_e) => {
            if verbose {
                println!("Couldn't find AbuseIPDB API key. If you want to use this feature make sure to put the API key into the environment variable 'ABUSEIPDB_API_KEY'.");
            }
            return Ok(None);
        },
    };

    let client = reqwest::blocking::Client::new();
    let url = "https://api.abuseipdb.com/api/v2/check";
    let params = [
        ("ipAddress", remote_ip),
        ("maxAgeInDays", &("40".to_string())),
    ];
    let response: reqwest::blocking::Response = client
        .get(url)
        .header("Key", abuseipdb_api_key)
        .header("Accept", "application/json")
        .query(&params)
        .send()?;

    // check if the request was successful
    if response.status().is_success() {
        let json_response: Value = response.json()?;
        let abuse_confidence_score: Option<i64> = json_response["data"]["abuseConfidenceScore"].as_i64();

        return Ok(abuse_confidence_score);
    }
    else {
        println!("AbuseIPDB Request failed with status code: {}", response.status());
        return Ok(None);
    }
}


pub fn check_if_known(remote_ip: &str) -> String {
    /* check if an IP corresponds to a DNS server */

    if remote_ip == "0.0.0.0" || remote_ip == "[::]" {
        return format!("*{}*", remote_ip.to_string());
    }
    else if remote_ip == "127.0.0.1" || remote_ip == "[::1]" {
        return format!("*{} localhost*", remote_ip.to_string());
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

