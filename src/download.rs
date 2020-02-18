use std::io::Read;

use reqwest;
use serde_json;

use crate::{Result, Error};

pub fn download<S: AsRef<str>>(url: S) -> Result<Vec<u8>> {
    let (body, is_success) = {
        match reqwest::blocking::get(url.as_ref()) {
            Ok(mut response) => {
                let mut body: Vec<u8> = vec![];

                if let Err(e) = response.read_to_end(&mut body) {
                    return Err(Error::IoError(e.to_string()));
                }

                (body, response.status().is_success())
            },

            Err(e) => return Err(Error::DownloadFailed(e.to_string())),
        }
    };

    if is_success {
        Ok(body)
    } else {
        match String::from_utf8(body) {
            Ok(encoded_data) => {
                match serde_json::from_str(&encoded_data[..]) {
                    Ok(api_error) => Err(Error::ApiCallFailed(api_error)),
                    Err(e) => Err(Error::ParsingFailed(e.to_string())),
                }
            },

            Err(e) => Err(Error::ParsingFailed(e.to_string())),
        }
    }
}
