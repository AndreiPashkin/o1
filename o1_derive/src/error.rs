//! Error definitions specific to the o1_derive crate.

use proc_macro2::Span;
use thiserror::Error;

/// Crate-wise error.
#[derive(Error, Debug)]
pub enum O1DeriveError {
    /// Input parsing or attribute parsing error.
    #[error("{message}")]
    ParseError { message: String, span: Span },
    /// Handled but critical internal error.
    #[error("{message}")]
    InternalError { message: String, span: Span },
}

impl O1DeriveError {
    pub fn parse_error<M: Into<String>>(message: M, span: Span) -> Self {
        O1DeriveError::ParseError {
            message: message.into(),
            span,
        }
    }
    pub fn internal_error<M: Into<String>>(message: M, span: Span) -> Self {
        O1DeriveError::InternalError {
            message: message.into(),
            span,
        }
    }
    pub fn span(&self) -> Span {
        match self {
            O1DeriveError::ParseError { span, .. } => *span,
            O1DeriveError::InternalError { span, .. } => *span,
        }
    }
    pub fn message(&self) -> &str {
        match self {
            O1DeriveError::ParseError { message, .. } => message,
            O1DeriveError::InternalError { message, .. } => message,
        }
    }
}

impl From<syn::Error> for O1DeriveError {
    fn from(err: syn::Error) -> Self {
        O1DeriveError::ParseError {
            message: err.to_string(),
            span: err.span(),
        }
    }
}

impl From<O1DeriveError> for syn::Error {
    fn from(err: O1DeriveError) -> Self {
        syn::Error::new(err.span(), err.to_string())
    }
}
