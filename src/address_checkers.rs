
use reqwest::{self};
use serde_json::{Value};
use std::{error::Error, env};
use crate::string_utils;


/// Requests an abuse score from the AbuseIPDB.com /check endpoint given an IP address.
/// The function expects that the environment variable `ABUSEIPDB_API_KEY` is set with an AbuseIPDB.com API key.
/// 
/// # Arguments
/// * `remote_address`: The address to be checked.
/// * `verbose`: Print information about the API request if set to `true`.
/// 
/// # Returns
/// If the request is successful the abuse sore is returned, if not `Some(None)` is returned.
pub fn check_address_for_abuse(remote_address: &String, verbose: bool) -> Result<Option<i64>, Box<dyn Error>> {
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
        ("ipAddress", remote_address),
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

        Ok(abuse_confidence_score)
    }
    else {
        if verbose {
            string_utils::pretty_print_error(
                &format!("AbuseIPDB Request failed with status code: {}", response.status())
            );
        }
        Ok(None)
    }
}


/// Represents the type of an IP address.
///
/// # Variants
/// * `Localhost`: Represents the localhost/127.0.0.1 address.
/// * `Unspecified`: Represents an unspecified or wildcard address.
/// * `Extern`: Represents an external address.
#[derive(Debug)]
pub enum IPType {
    Localhost,
    Unspecified,
    Extern
}


/// Checks if a given IP address is either "unspecified", localhost or an extern address.
/// 
/// * `0.0.0.0` or `[::]` -> unspecified
/// * `127.0.0.1` or `[::1]` -> localhost
/// * else -> extern address
/// 
/// # Arguments
/// * `remote_address`: The address to be checked.
/// 
/// # Returns
/// The address-type as an IPType enum.
pub fn check_address_type(remote_address: &str) -> IPType {
    if remote_address == "127.0.0.1" || remote_address == "[::1]" {
        return IPType::Localhost;
    }
    else if remote_address == "0.0.0.0" || remote_address == "[::]" {
        return IPType::Unspecified;
    }
    IPType::Extern
}

