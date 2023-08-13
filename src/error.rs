//! Library specific errors.

use http::StatusCode;
use reqwest::Response;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::Value;
use std::fmt::{Debug, Display, Formatter};

pub(crate) type Result<T, E = Error> = core::result::Result<T, E>;

/// Crate specfic error types.
#[derive(Clone, Debug)]
pub enum Error {
    /// Error was caused by something library internal. This only happens if something was
    /// implemented incorrectly (which hopefully should never be the case) or if Crunchyroll
    /// surprisingly changed specific parts of their api which broke a part of this crate.
    Internal(ErrorContext<()>),

    /// Some sort of error occurred while requesting the Crunchyroll api.
    Request(ErrorContext<StatusCode>),
    /// While decoding the api response body something went wrong.
    Decode(ErrorContext<()>),

    /// Something went wrong while logging in.
    Authentication(ErrorContext<()>),

    /// Generally malformed or invalid user input.
    Input(ErrorContext<()>),

    /// When the request got blocked. Currently this only triggers when the cloudflare bot
    /// protection is detected.
    Block(ErrorContext<()>),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Internal(context) => write!(f, "{context}"),
            Error::Request(context) => write!(f, "{context}"),
            Error::Decode(context) => write!(f, "{context}"),
            Error::Authentication(context) => write!(f, "{context}"),
            Error::Input(context) => write!(f, "{context}"),
            Error::Block(context) => write!(f, "{context}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::Decode(ErrorContext::new(err.to_string()))
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        let mut context: ErrorContext<()> = ErrorContext::new(err.to_string());
        if let Some(url) = err.url() {
            context = context.with_url(url.clone());
        }

        if err.is_request()
            || err.is_redirect()
            || err.is_timeout()
            || err.is_connect()
            || err.is_body()
            || err.is_status()
        {
            let mut request_context = context.into_other_context();
            if let Some(status) = err.status() {
                request_context = request_context.with_extra(status)
            }
            Error::Request(request_context)
        } else if err.is_decode() {
            Error::Decode(context)
        } else if err.is_builder() {
            Error::Internal(context)
        } else {
            Error::Internal(ErrorContext::new(format!(
                "Could not determine request error type - {err}"
            )))
        }
    }
}

/// Information about a [`Error`].
#[derive(Clone, Debug)]
pub struct ErrorContext<T: Clone> {
    pub message: String,
    pub url: Option<String>,
    pub value: Option<String>,

    pub extra: Option<T>,
}

impl<T: Clone> Display for ErrorContext<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut res = self.message.clone();

        if let Some(url) = &self.url {
            res.push_str(&format!(" ({url})"));
        }
        if let Some(value) = &self.value {
            res.push_str(&format!(": {value}"));
        }

        write!(f, "{res}")
    }
}

impl<T: Clone> From<String> for ErrorContext<T> {
    fn from(string: String) -> Self {
        ErrorContext::new(string)
    }
}

impl<T: Clone> From<&str> for ErrorContext<T> {
    fn from(str: &str) -> Self {
        ErrorContext::new(str)
    }
}

impl<T: Clone> ErrorContext<T> {
    pub(crate) fn new<S: ToString>(message: S) -> Self {
        Self {
            message: message.to_string(),
            url: None,
            value: None,
            extra: None,
        }
    }

    pub(crate) fn with_url<S: AsRef<str>>(mut self, url: S) -> Self {
        self.url = Some(url.as_ref().to_string());

        self
    }

    pub(crate) fn with_value(mut self, value: &[u8]) -> Self {
        self.value = Some(format!(
            ": {}",
            std::str::from_utf8(value).unwrap_or("-- not displayable --")
        ));

        self
    }

    pub(crate) fn with_extra(mut self, extra: T) -> Self {
        self.extra = Some(extra);

        self
    }

    pub(crate) fn into_other_context<T1: Clone>(self) -> ErrorContext<T1> {
        ErrorContext {
            message: self.message,
            url: self.url,
            value: self.value,
            extra: None,
        }
    }
}

pub(crate) fn is_request_error(value: Value) -> Result<()> {
    #[derive(Debug, Deserialize)]
    struct CodeFieldContext {
        code: String,
        field: String,
    }

    #[derive(Debug, Deserialize)]
    struct MessageType {
        message: String,
        #[serde(rename = "type")]
        error_type: String,
    }
    #[derive(Debug, Deserialize)]
    struct CodeContextError {
        code: String,
        context: Vec<CodeFieldContext>,
        #[serde(alias = "error")]
        message: Option<String>,
    }
    #[derive(Debug, Deserialize)]
    struct ConstraintsErrorContext {
        code: String,
        violated_constraints: Vec<(String, String)>,
    }
    #[derive(Debug, Deserialize)]
    struct ConstraintsError {
        code: String,
        context: Vec<ConstraintsErrorContext>,
    }

    if let Ok(err) = serde_json::from_value::<MessageType>(value.clone()) {
        return Err(Error::Request(
            format!("{} - {}", err.error_type, err.message).into(),
        ));
    } else if let Ok(err) = serde_json::from_value::<CodeContextError>(value.clone()) {
        let mut details: Vec<String> = vec![];

        for item in err.context.iter() {
            details.push(format!("{}: {}", item.field, item.code))
        }

        return if let Some(message) = err.message {
            Err(Error::Request(
                format!("{} ({}) - {}", message, err.code, details.join(", ")).into(),
            ))
        } else {
            Err(Error::Request(
                format!("({}) - {}", err.code, details.join(", ")).into(),
            ))
        };
    } else if let Ok(err) = serde_json::from_value::<ConstraintsError>(value) {
        let details = err
            .context
            .iter()
            .map(|e| {
                format!(
                    "{}: ({})",
                    e.code,
                    e.violated_constraints
                        .iter()
                        .map(|(key, value)| format!("{key}: {value}"))
                        .collect::<Vec<String>>()
                        .join(", ")
                )
            })
            .collect::<Vec<String>>();

        return Err(Error::Request(
            format!("{}: {}", err.code, details.join(", ")).into(),
        ));
    }
    Ok(())
}

pub(crate) async fn check_request<T: DeserializeOwned>(url: String, resp: Response) -> Result<T> {
    let content_length = resp.content_length().unwrap_or(0);
    let status = resp.status();
    let _raw = match resp.status().as_u16() {
        403 => {
            let raw = resp.bytes().await?;
            if raw.starts_with(b"<!DOCTYPE html>")
                && raw
                    .windows(31)
                    .any(|w| w == b"<title>Just a moment...</title>")
            {
                return Err(Error::Block(
                    ErrorContext::new("Triggered Cloudflare bot protection").with_url(url),
                ));
            }
            raw
        }
        404 => {
            return Err(Error::Request(
                ErrorContext::new("The requested resource is not present (404)")
                    .with_url(url)
                    .with_extra(status),
            ))
        }
        429 => {
            let retry_secs =
                if let Some(retry_after) = resp.headers().get(http::header::RETRY_AFTER) {
                    retry_after.to_str().map_or(None, |retry_after_secs| {
                        retry_after_secs.parse::<u32>().ok()
                    })
                } else {
                    None
                };

            return Err(Error::Request(
                ErrorContext::new(format!(
                    "Rate limit detected. {}",
                    retry_secs.map_or("Try again later".to_string(), |secs| format!(
                        "Try again in {secs} seconds"
                    ))
                ))
                .with_url(url)
                .with_extra(status),
            ));
        }
        _ => resp.bytes().await?,
    };
    let mut raw: &[u8] = _raw.as_ref();

    // to ensure compatibility with `T`, convert a empty response to {}
    if raw.is_empty() && (content_length == 0) {
        raw = "{}".as_bytes();
    }

    let value: Value = serde_json::from_slice(raw).map_err(|e| {
        Error::Decode(
            ErrorContext::new(format!("{} at {}:{}", e, e.line(), e.column()))
                .with_url(&url)
                .with_value(raw),
        )
    })?;
    is_request_error(value.clone()).map_err(|e| {
        if let Error::Request(context) = e {
            Error::Request(context.with_url(&url).with_extra(status))
        } else {
            e
        }
    })?;
    serde_json::from_value::<T>(value).map_err(|e| {
        Error::Decode(
            ErrorContext::new(format!("{} at {}:{}", e, e.line(), e.column()))
                .with_url(&url)
                .with_value(raw),
        )
    })
}
