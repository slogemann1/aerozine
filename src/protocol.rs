use std::str;
use openssl::x509::X509;
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
    ProxyRequestRefused,
    BadRequest,
    CertificateRequired,
    CertificateUnauthorized, //TODO
    CertificateInvalid //TODO
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

    pub fn from_i32(code: i32) -> Option<Self> {
        match code {
            10 => Some(Self::Input),
            11 => Some(Self::SensitiveInput),
            20 => Some(Self::Success),
            30 => Some(Self::RedirectTemporary),
            31 => Some(Self::RedirectPermenent),
            40 => Some(Self::TemporaryFailure),
            41 => Some(Self::ServerUnavailible),
            42 => Some(Self::CGIError),
            43 => Some(Self::ProxyError),
            44 => Some(Self::SlowDown),
            50 => Some(Self::PermanentFailure),
            51 => Some(Self::NotFound),
            52 => Some(Self::Gone),
            53 => Some(Self::ProxyRequestRefused),
            59 => Some(Self::BadRequest),
            60 => Some(Self::CertificateRequired),
            61 => Some(Self::CertificateUnauthorized),
            62 => Some(Self::CertificateInvalid),
            _ => None
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
        header.push(' ');
        header.push_str(&self.meta);
        header.push_str("\r\n");

        [ header.as_bytes(), &self.body ].concat()
    }
}

pub struct Request<'a> {
    pub domain: String,
    pub path: String,
    pub query: Option<String>,
    pub certificate: Option<&'a X509>
}

pub fn parse_request(bytes: &[u8]) -> Result<Request> {
    let request_string = match str::from_utf8(bytes) {
        Ok(val) => val,
        Err(_) => {
            return Err(ServerError::from_str(
                "Error: Request not in utf-8 encoding",
                StatusCode::BadRequest
            ));
        }
    };

    // Check request header for gemini://
    let request;
    let len = request_string.len();
    if request_string.starts_with("gemini://") && len > 9 {
        if request_string.ends_with("\r\n") { // Check for trailing \r\n
            request = request_string[9..(len - 2)].to_string();
        }
        else {
            return Err(ServerError::new(
                String::from("Error: Invalid request, \\r\\n was not present"),
                StatusCode::BadRequest
            ));
        }

    }
    else if request_string.starts_with("http://") || request_string.starts_with("https://") // Check for attempted proxy request
    || request_string.starts_with("gopher://") {
        return Err(ServerError::new(
            String::from("Error: This server does not handle proxy requests"),
            StatusCode::ProxyRequestRefused
        ));
    }
    else { // Otherwise bad request
        return Err(ServerError::new(
            String::from("Error: Bad header"),
            StatusCode::BadRequest
        ));
    }

    // Seperate request into url and parameters
    let mut url;
    let mut query: Option<String> = None;
    let mut parts: Vec<&str> = request.splitn(2, "?").collect();
    if parts.len() == 1 {
        url = parts.pop().unwrap().to_string();
    }
    else {
        query = Some(parts.pop().unwrap().to_string());
        url = parts.pop().unwrap().to_string();
    }

    // Escape ' and " for command line
    if let Some(val) = query {
        query = Some(val.replace("'", "%27").replace("\"", "%22"));
    }

    // Pad url with extra slash
    if !url.ends_with('/') {
        url.push('/');
    }

    // Get domain and path from url
    let mut domain_and_path: Vec<&str> = url.splitn(2, "/").collect();
    if domain_and_path.len() != 2 {
        return Err(ServerError::new(
            format!(
                "Error: Invalid request. Request: {}",
                &request_string
            ),
            StatusCode::BadRequest
        ));
    }

    let path = domain_and_path.pop().unwrap().to_string();
    let mut domain = domain_and_path.pop().unwrap().to_string();

    // Handle requests with explicit port
    if domain.ends_with(":1965") {
        domain = domain[0..(domain.len() - 5)].to_string();
    }

    Ok(
        Request {
            domain,
            path,
            query,
            certificate: None
        }
    )
}