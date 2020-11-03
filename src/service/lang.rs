// TODO Add more Languages

use crate::LeetUpError;
use anyhow::anyhow;
use std::str::FromStr;

/// Store Lang attributes.
#[derive(Debug, Clone)]
pub struct LangInfo {
    pub name: String,
    pub extension: String,
    pub comment: Comment,
}

#[derive(Debug, Clone)]
pub enum CommentStyle {
    Single(String),
    Multiline {
        start: String,
        between: String,
        end: String,
    },
}

/// Comment for different languages.
#[derive(Debug, Clone)]
pub enum Comment {
    C(CommentStyle, Option<CommentStyle>),
    Python3(CommentStyle, Option<CommentStyle>),
    MySQL(CommentStyle, Option<CommentStyle>),
}

/// Represent different languages supported by a Service provider.
#[derive(Debug, Clone)]
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
        let c_comment = Comment::C(
            CommentStyle::Single("//".into()),
            Some(CommentStyle::Multiline {
                start: "/*".into(),
                between: "*".into(),
                end: "*/".into(),
            }),
        );
        let py_comment = Comment::Python3(CommentStyle::Single("#".into()), None);
        let mysql_comment = Comment::MySQL(CommentStyle::Single("--".into()), None);

        match s {
            "rust" => Ok(Lang::Rust(LangInfo {
                name: "rust".to_string(),
                extension: "rs".to_string(),
                comment: c_comment,
            })),
            "java" => Ok(Lang::Java(LangInfo {
                name: "java".to_string(),
                extension: "java".to_string(),
                comment: c_comment,
            })),
            "js" => Ok(Lang::Java(LangInfo {
                name: "javascript".to_string(),
                extension: "js".to_string(),
                comment: c_comment,
            })),
            "python" => Ok(Lang::Java(LangInfo {
                name: "python3".to_string(),
                extension: "py".to_string(),
                comment: py_comment,
            })),
            "mysql" => Ok(Lang::Java(LangInfo {
                name: "mysql".to_string(),
                extension: "sql".to_string(),
                comment: mysql_comment,
            })),
            _ => Err(LeetUpError::Any(anyhow!("Language not supported!"))),
        }
    }
}
