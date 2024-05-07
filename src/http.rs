use anyhow::Context;
use log::{debug, error, info};
use std::fmt::Display;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;
#[derive(PartialEq, Eq, Debug)]
pub enum HttpMethod {
    None,
    Get,
    Post,
    Put,
    Delete,
}

#[derive(Debug, PartialEq, Eq)]
pub enum HttpParserHeaderStatus {
    Continue,
    Ended,
}

#[derive(Debug)]
#[allow(clippy::enum_variant_names)]
pub enum HttpParseError<'a> {
    HeaderMalformed(&'a str), //fkd up request (client is sending messed up things)(bail out from paring)
    HeaderIncomplete(&'a str), //could only happen if the request is partially written (client disconnected between writes) (bail out)
    HeaderNoKeyValuePair(&'a str), //straight up fkd (client is sending messed up things)(bail out)
    HeaderKeyUnknown(&'a str), //ignore for now
}
fn split_once_no_error<'a>(str: &'a str, delim: &str) -> &'a str {
    let ret = str.split_once(delim).context("split delimeter not found");
    match ret {
        Ok(line) => line.0,
        Err(_) => str,
    }
}
impl<'a> Display for HttpParseError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpParseError::HeaderMalformed(location) => {
                write!(
                    f,
                    "ttpMalformedHeader: {}",
                    split_once_no_error(location, "\r\n").escape_default()
                )?;
            }
            HttpParseError::HeaderIncomplete(location) => {
                write!(
                    f,
                    "HttpHeaderIncomplete: {}",
                    split_once_no_error(location, "\r\n").escape_default()
                )?;
            }
            HttpParseError::HeaderNoKeyValuePair(location) => {
                write!(
                    f,
                    "HttpHeaderNoKeyValuePair: \"{}\"",
                    split_once_no_error(location, "\r\n").escape_default()
                )?;
            }
            HttpParseError::HeaderKeyUnknown(location) => {
                write!(
                    f,
                    "HttpHeaderKeyUnknown: \"{}\"",
                    split_once_no_error(location, "\r\n").escape_default()
                )?;
            }
        };
        Ok(())
    }
}
impl<'a> std::error::Error for HttpParseError<'a> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }

    fn description(&self) -> &str {
        "description() is deprecated; use Display"
    }
    fn cause(&self) -> Option<&dyn std::error::Error> {
        self.source()
    }
}

pub struct HttpResponse {
    pub response: String,
}
impl HttpResponse {
    pub fn new() -> Self {
        Self {
            response: String::new(),
        }
    }
    pub fn push_header(&mut self, header: &str) {
        if !header.ends_with("\r\n") {
            let mut current_header = String::from(header);
            current_header.push_str("\r\n");
            self.response.push_str(&current_header);
        } else {
            self.response.push_str(header);
        }
    }
    pub fn header_ok(&mut self) {
        self.response.clear();
        self.push_header("HTTP/1.1 200 OK");
    }
}

#[derive(Debug)]
pub struct HttpRequest {
    pub method: HttpMethod,
    pub path: String,
    pub ver: String,
    pub host: String,
    pub user_agent: String,
    pub accepts: String,
}
impl HttpRequest {
    pub fn new() -> Self {
        Self {
            method: HttpMethod::None,
            path: String::new(),
            ver: String::new(),
            host: String::new(),
            user_agent: String::new(),
            accepts: String::new(),
        }
    }
    //GET /index.html HTTP/1.1
    //Host: localhost:4221
    //User-Agent: curl/7.64.1
    pub fn parse_request_line(&mut self, request: &str) -> anyhow::Result<()> {
        let (method, rest) = request.split_once(' ').context("malformed request line")?;
        self.method = match method {
            "GET" => HttpMethod::Get,
            "POST" => HttpMethod::Post,
            "PUT" => HttpMethod::Put,
            "DELETE" => HttpMethod::Delete,
            _ => HttpMethod::None,
        };
        if self.method == HttpMethod::None {
            anyhow::bail!("HttpMethod not found");
        }
        let (path, version) = rest.split_once(' ').context("malformed request line")?;
        self.path = String::from(path);
        self.ver = String::from(version);
        anyhow::Ok(())
    }

    pub fn parse_header<'a>(
        &mut self,
        request: &'a str,
    ) -> Result<HttpParserHeaderStatus, HttpParseError<'a>> {
        if request == "\r\n" {
            return Ok(HttpParserHeaderStatus::Ended);
        }
        if !request.ends_with("\r\n") {
            return Err(HttpParseError::HeaderIncomplete(request));
        }

        let (key, value) = request
            .split_once(':')
            .context("malformed request")
            .map_err(|_| return HttpParseError::HeaderNoKeyValuePair(request))?;

        match key {
            "Host" => self.host = String::from(value.trim()),
            "User-Agent" => self.user_agent = String::from(value.trim()),
            "Accept" => self.accepts = String::from(value.trim()),
            "" => return Err(HttpParseError::HeaderMalformed(request)),
            _ => return Err(HttpParseError::HeaderKeyUnknown(request)),
        }
        Ok(HttpParserHeaderStatus::Continue)
    }
}

pub struct HttpReader<'a, T>
where
    T: Read,
    &'a T: Read,
{
    stream: BufReader<&'a T>,
    buf: String,
}
impl<'a, T> HttpReader<'a, T>
where
    T: Read,
    &'a T: Read,
{
    pub fn new(stream: &'a T) -> Self {
        Self {
            stream: BufReader::new(stream),
            buf: String::new(),
        }
    }
    pub fn read_request(mut self) -> std::io::Result<HttpRequest> {
        let mut request = HttpRequest::new();
        self.stream.read_line(&mut self.buf)?;
        request.parse_request_line(&self.buf).map_err(|e| {
            debug!("Error while parsing requestline: {}:{:?}", self.buf, e);
            std::io::Error::new(std::io::ErrorKind::InvalidData, e)
        })?;
        loop {
            self.buf.clear();
            let _bytes_read = self.stream.read_line(&mut self.buf)?;
            match request.parse_header(&self.buf) {
                Ok(status) => {
                    match status {
                        HttpParserHeaderStatus::Ended => break,
                        HttpParserHeaderStatus::Continue => continue,
                    };
                }
                Err(error) => {
                    debug!("failed to parse http header: {:?}", error);
                    if let HttpParseError::HeaderKeyUnknown(_) = error {
                        continue;
                    }
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        error.to_string(), //converting it to string because of ownership idk
                    ));
                }
            }
        }
        Ok(request)
    }
}
