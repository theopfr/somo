
use reqwest::{self};
use serde_json::{Value};
use std::{error::Error, env};
use crate::string_utils;


/// returns an abuse-score (0-100) for a given IP address using the AbuseIPDB /check endpoint
pub fn get_ip_audit(remote_ip: &String, verbose: bool) -> Result<Option<i64>, Box<dyn Error>> {

    let abuseipdb_api_key: String = match env::var("ABUSEIPDB_API_KEY") {
        Ok(val) => val,
        Err(_e) => {
            if verbose {
                string_utils::pretty_print_warning(
                    "Couldn't find AbuseIPDB API key. If you want to use this feature make sure to put the API key into the environment variable `ABUSEIPDB_API_KEY`.*"
                );
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
        if verbose {
            string_utils::pretty_print_error(
                &format!("AbuseIPDB Request failed with status code: {}", response.status())
            );
        }
        return Ok(None);
    }
}


/// checks if a given IP address is either "unspecified" or localhost
pub fn check_for_known_ip(remote_ip: &str) -> u8 {
    if remote_ip == "0.0.0.0" || remote_ip == "[::]" {
        return 1u8;
    }
    else if remote_ip == "127.0.0.1" || remote_ip == "[::1]" {
        return 2u8;
    }
    return 0u8;
}

