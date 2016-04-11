use std::io::Read;

use hyper::Client;
use hyper::header::Connection;
use rustc_serialize::json;

use {Result, Error};

pub fn download<S: AsRef<str>>(url: S) -> Result<Vec<u8>> {
    match Client::new().get(url.as_ref()).header(Connection::close()).send() {
        Ok(mut response) => {
            let mut body: Vec<u8> = vec![];

            if let Err(e) = response.read_to_end(&mut body) {
                return Err(Error::IoError(e.to_string()));
            }

            if response.status.is_success() {
                Ok(body)
            } else {
                match String::from_utf8(body) {
                    Ok(encoded_error) => {
                        match json::decode(&encoded_error[..]) {
                            Ok(api_error) => Err(Error::ApiCallFailed(api_error)),
                            Err(e)        => Err(Error::ParsingFailed(e.to_string())),
                        }
                    },

                    Err(e) => return Err(Error::ParsingFailed(e.to_string())),
                }
            }
        },

        Err(ref e) => Err(Error::DownloadFailed(e.to_string())),
    }
}
