use std::cell::RefCell;
use std::rc::Rc;

use email_address::EmailAddress;
use lazy_static::lazy_static;
use regex::Regex;
use url::Url;

pub mod config;
pub mod generate;
mod html;
mod lexer;
pub mod markdown;
mod parser;
mod utils;

pub type SharedLine = Rc<RefCell<parser::Line>>;

// This regex is used to match a string with double quotes("") or single quotes('')
lazy_static! {
    static ref D_QUOTED_STRING_RE: Regex = Regex::new("^\"([^\"\\\\]|\\\\.)*\"$").unwrap();
    static ref S_QUOTED_STRING_RE: Regex = Regex::new("^\'([^\'\\\\]|\\\\.)*\'$").unwrap();
}

pub fn is_quoted_string(s: &str) -> bool {
    D_QUOTED_STRING_RE.is_match(s) || S_QUOTED_STRING_RE.is_match(s)
}

pub fn is_url(s: &str) -> bool {
    Url::try_from(s).is_ok()
}

pub fn is_email(s: &str) -> bool {
    EmailAddress::is_valid(s)
}
