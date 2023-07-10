use gtk::glib;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    HttpError(reqwest::Error),
    GlibError(glib::error::Error),
    XmlError(quick_xml::Error),
    XmlAttrError(quick_xml::events::attributes::AttrError),
    FromUtf8Error(std::string::FromUtf8Error),
    NoConversionError,
    HeaderToStrError(reqwest::header::ToStrError),
    MimeParseError(mime::FromStrError),
    NoContentTypeError,
    UnsupportedCharsetError(String),
    UnsupportedContentTypeError(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::IoError(err) => write!(f, "io error: {}", err),
            Error::HttpError(err) => write!(f, "http error: {}", err),
            Error::GlibError(err) => write!(f, "glib error: {}", err),
            Error::XmlError(err) => write!(f, "xml error: {}", err),
            Error::XmlAttrError(err) => write!(f, "xml attribute error: {}", err),
            Error::FromUtf8Error(err) => write!(f, "from utf8 error: {}", err),
            Error::NoConversionError => write!(f, "no conversion error"),
            Error::HeaderToStrError(err) => {
                write!(f, "failed to convert header value to string: {}", err)
            }
            Error::MimeParseError(err) => write!(f, "failed to parse mime type: {}", err),
            Error::NoContentTypeError => write!(f, "no Content-Type header provided by server"),
            Error::UnsupportedCharsetError(charset) => {
                write!(f, "expected UTF-8, received charset: {}", charset)
            }
            Error::UnsupportedContentTypeError(content_type) => {
                write!(f, "unsupported Content-Type: {}", content_type)
            }
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::IoError(err)
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Error {
        Error::HttpError(err)
    }
}

impl From<glib::Error> for Error {
    fn from(err: glib::Error) -> Error {
        Error::GlibError(err)
    }
}

impl From<quick_xml::Error> for Error {
    fn from(err: quick_xml::Error) -> Error {
        Error::XmlError(err)
    }
}

impl From<quick_xml::events::attributes::AttrError> for Error {
    fn from(err: quick_xml::events::attributes::AttrError) -> Error {
        Error::XmlAttrError(err)
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(err: std::string::FromUtf8Error) -> Error {
        Error::FromUtf8Error(err)
    }
}

impl From<reqwest::header::ToStrError> for Error {
    fn from(err: reqwest::header::ToStrError) -> Error {
        Error::HeaderToStrError(err)
    }
}

impl From<mime::FromStrError> for Error {
    fn from(err: mime::FromStrError) -> Error {
        Error::MimeParseError(err)
    }
}
