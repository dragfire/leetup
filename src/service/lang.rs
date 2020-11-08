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
    Cpp(LangInfo),
    Ruby(LangInfo),
    C(LangInfo),
    CSharp(LangInfo),
    Go(LangInfo),
    Php(LangInfo),
    Kotlin(LangInfo),
    Scala(LangInfo),
    Swift(LangInfo),
    Typescript(LangInfo),
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

        let javascript_lang = Lang::Javascript(LangInfo {
            name: "javascript".to_string(),
            extension: "js".to_string(),
            comment: c_comment.clone(),
        });
        let python_lang = Lang::Python3(LangInfo {
            name: "python3".to_string(),
            extension: "py".to_string(),
            comment: py_comment.clone(),
        });

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
            "js" => Ok(javascript_lang),
            "javascript" => Ok(javascript_lang),
            "python" => Ok(python_lang),
            "py" => Ok(python_lang),
            "python3" => Ok(python_lang),
            "cpp" => Ok(Lang::Cpp(LangInfo {
                name: "cpp".into(),
                extension: "cpp".into(),
                comment: c_comment,
            })),
            "mysql" => Ok(Lang::MySQL(LangInfo {
                name: "mysql".to_string(),
                extension: "sql".to_string(),
                comment: mysql_comment,
            })),
            "ruby" => Ok(Lang::Ruby(LangInfo {
                name: "ruby".to_string(),
                extension: "rb".to_string(),
                comment: py_comment.clone(),
            })),
            "rb" => Ok(Lang::Ruby(LangInfo {
                name: "ruby".to_string(),
                extension: "rb".to_string(),
                comment: py_comment.clone(),
            })),
            "c" => Ok(Lang::C(LangInfo {
                name: "c".into(),
                extension: "c".into(),
                comment: c_comment,
            })),
            "csharp" => Ok(Lang::CSharp(LangInfo {
                name: "csharp".into(),
                extension: "cs".into(),
                comment: c_comment,
            })),
            "cs" => Ok(Lang::CSharp(LangInfo {
                name: "csharp".into(),
                extension: "cs".into(),
                comment: c_comment,
            })),
            "golang" => Ok(Lang::Go(LangInfo {
                name: "golang".into(),
                extension: "go".into(),
                comment: c_comment,
            })),
            "go" => Ok(Lang::Go(LangInfo {
                name: "golang".into(),
                extension: "go".into(),
                comment: c_comment,
            })),
            "php" => Ok(Lang::Php(LangInfo {
                name: "php".into(),
                extension: "php".into(),
                comment: c_comment,
            })),
            "kotlin" => Ok(Lang::Kotlin(LangInfo {
                name: "kotlin".into(),
                extension: "kt".into(),
                comment: c_comment,
            })),
            "scala" => Ok(Lang::Scala(LangInfo {
                name: "scala".into(),
                extension: "scala".into(),
                comment: c_comment,
            })),
            "swift" => Ok(Lang::Swift(LangInfo {
                name: "swift".into(),
                extension: "swift".into(),
                comment: c_comment,
            })),
            "typescript" => Ok(Lang::Typescript(LangInfo {
                name: "typescript".into(),
                extension: "ts".into(),
                comment: c_comment,
            })),
            "ts" => Ok(Lang::Typescript(LangInfo {
                name: "typescript".into(),
                extension: "ts".into(),
                comment: c_comment,
            })),
            _ => Err(LeetUpError::Any(anyhow!("Language not supported!"))),
        }
    }
}

impl Lang {
    pub fn info(&self) -> LangInfo {
        match self.clone() {
            Lang::Rust(info) => info,
            Lang::Java(info) => info,
            Lang::Javascript(info) => info,
            Lang::Python3(info) => info,
            Lang::MySQL(info) => info,
            Lang::Cpp(info) => info,
            Lang::Ruby(info) => info,
            Lang::C(info) => info,
            Lang::CSharp(info) => info,
            Lang::Go(info) => info,
            Lang::Php(info) => info,
            Lang::Kotlin(info) => info,
            Lang::Scala(info) => info,
            Lang::Swift(info) => info,
            Lang::Typescript(info) => info,
        }
    }
}
