use std::{error::Error, fs::File, io::Read};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub css_href: String,
    pub custom_html_head: String,
    pub add_class_on_article: String,
    // convert newline to <br>
    pub enable_newline_to_br: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            css_href: String::from(""),
            custom_html_head: String::from(""),
            add_class_on_article: String::from(""),
            enable_newline_to_br: true,
        }
    }
}

pub fn read_config(path: &str) -> Result<Config, Box<dyn Error>> {
    let mut buf = String::new();
    File::open(path)?.read_to_string(&mut buf)?;
    let cfg: Config = serde_json::from_str(&buf)?;
    Ok(cfg)
}
