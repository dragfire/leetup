// TODO Add more Languages

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
    Javascript(LangInfo),
    Python3(LangInfo),
    MySQL(LangInfo),
}

impl FromStr for Lang {
    type Err = LeetUpError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let c_comment = CommentStyle::C("//".to_string());
        let py_comment = CommentStyle::C("#".to_string());
        let mysql_comment = CommentStyle::C("--".to_string());

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
            "js" => Ok(Lang::Java(LangInfo {
                name: "javascript".to_string(),
                extension: "js".to_string(),
                comment_style: c_comment,
            })),
            "python" => Ok(Lang::Java(LangInfo {
                name: "python3".to_string(),
                extension: "py".to_string(),
                comment_style: py_comment,
            })),
            "mysql" => Ok(Lang::Java(LangInfo {
                name: "mysql".to_string(),
                extension: "sql".to_string(),
                comment_style: mysql_comment,
            })),
            _ => Err(LeetUpError::Any(anyhow!("Language not supported!"))),
        }
    }
}
