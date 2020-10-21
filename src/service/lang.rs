use crate::LeetUpError;
use anyhow::anyhow;
use std::str::FromStr;

/// Store Lang attributes.
#[derive(Debug)]
pub struct LangInfo {
    pub name: String,
    pub extension: String,
    pub comment_style: CommentStyle,
}

/// Comment styles for different languages.
#[derive(Debug)]
pub enum CommentStyle {
    C(String),
    Lisp(String),
}

/// Represent different languages supported by a Service provider.
#[derive(Debug)]
pub enum Lang {
    Rust(LangInfo),
    Java(LangInfo),
}

impl FromStr for Lang {
    type Err = LeetUpError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let c_comment = CommentStyle::C("//".to_string());

        match s {
            "rust" => Ok(Lang::Rust(LangInfo {
                name: "rust".to_string(),
                extension: "rs".to_string(),
                comment_style: c_comment,
            })),
            "java" => Ok(Lang::Java(LangInfo {
                name: "java".to_string(),
                extension: "java".to_string(),
                comment_style: c_comment,
            })),
            _ => Err(LeetUpError::Any(anyhow!("Language not supported!"))),
        }
    }
}
