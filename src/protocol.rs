use std::str;
use crate::Result;
use crate::ServerError;

#[derive(Debug, Clone)]
pub enum StatusCode {
    Input,
    SensitiveInput,
    Success,
    RedirectTemporary, //TODO
    RedirectPermenent, //TODO
    TemporaryFailure, // Unimplemented
    ServerUnavailible, //TODO
    CGIError,
    ProxyError, //TODO
    SlowDown, // Unimplemented
    PermanentFailure, // Unimplemented
    NotFound,
    Gone, // Unimplemented (maybe in future)
    ProxyRequestRefused, //TODO
    BadRequest,
    CertificateRequired, //TODO
    CertificateUnauthorized, //TODO
    CertificateInvalid
}

impl StatusCode {
    pub fn to_u32(&self) -> u32 {
        match self {
            Self::Input => 10,
            Self::SensitiveInput => 11,
            Self::Success => 20,
            Self::RedirectTemporary => 30,
            Self::RedirectPermenent => 31,
            Self::TemporaryFailure => 40,
            Self::ServerUnavailible => 41,
            Self::CGIError => 42,
            Self::ProxyError => 43,
            Self::SlowDown => 44,
            Self::PermanentFailure => 50,
            Self::NotFound => 51,
            Self::Gone => 52,
            Self::ProxyRequestRefused => 53,
            Self::BadRequest => 59,
            Self::CertificateRequired => 60,
            Self::CertificateUnauthorized => 61,
            Self::CertificateInvalid => 62
        }
    }
}

#[derive(Debug, Clone)]
pub struct Response {
    pub status_code: StatusCode,
    pub meta: String,
    pub body: Vec<u8>
}

impl Response {
    pub fn new(status_code: StatusCode, meta: String, body: Vec<u8>) -> Response {
        Response {
            status_code,
            meta,
            body
        }
    }

    pub fn build(&self) -> Vec<u8> {
        let mut header = self.status_code.to_u32().to_string();
        header.push_str(&self.meta);
        header.push_str("\r\n");

        [ header.as_bytes(), &self.body ].concat()
    }
}

#[derive(Debug, Clone)]
pub struct Request {
    pub domain: String,
    pub path: String,
    pub parameters: Vec<(String, String)>
}

pub fn parse_request(bytes: &[u8]) -> Result<Request> {
    let request_string = match str::from_utf8(bytes) {
        Ok(val) => val,
        Err(_) => {
            return Err(ServerError::from_str(
                "Error: Request not in utf-8 form",
                StatusCode::BadRequest
            ));
        }
    };

    // Seperate request into url and parameters
    let url;
    let mut params: Vec<(String, String)> = Vec::new();
    let mut parts: Vec<&str> = request_string.splitn(2, "?").collect();
    if parts.len() == 1 {
        url = parts.pop().unwrap().to_string();
    }
    else {
        let param_val_pairs: Vec<&str> = parts.pop().unwrap().split("&").collect();
        for param_and_val in param_val_pairs {
            let mut param_and_val: Vec<&str> = param_and_val.split("+").collect();
            if param_and_val.len() != 2 {
                return Err(ServerError::new(
                    format!(
                        "Error: Invalid pairing of parameters and values in request. Request: {}",
                        &request_string
                    ),
                    StatusCode::BadRequest
                ));
            }

            let val = param_and_val.pop().unwrap();
            let param = param_and_val.pop().unwrap();
            params.push((param.to_string(), val.to_string()));
        }

        url = parts.pop().unwrap().to_string();
    }

    // Get domain and path from url
    let url = url.replace("gemini://", "").replace("\r\n", "");
    let mut domain_and_path: Vec<&str> = url.splitn(2, "/").collect();
    if domain_and_path.len() != 2 {
        return Err(ServerError::new(
            format!(
                "Error: Invalid pairing of parameters and values in request. Request: {}",
                &request_string
            ),
            StatusCode::BadRequest
        ));
    }
    let path = domain_and_path.pop().unwrap().to_string();
    let domain = domain_and_path.pop().unwrap().to_string();

    Ok(
        Request {
            domain: domain,
            path: path,
            parameters: params
        }
    )
}