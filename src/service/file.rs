use crate::{service::Problem, LeetUpError, Result};
use log::*;
use std::collections::HashMap;
use std::io::{self, BufRead, Read};
use std::path::Path;
use std::str::FromStr;

const LEETUP_MARKER: &'static str = "@leetup";

impl FromStr for Problem {
    type Err = LeetUpError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let map: HashMap<_, _> = s
            .split(' ')
            .map(|e| {
                let split = e.split('=').collect::<Vec<_>>();
                (split[0], split[1])
            })
            .collect();
        let id: usize = map.get("id").unwrap().parse().unwrap();
        let slug = map.get("slug").unwrap().to_string();
        let lang = map.get("lang").unwrap().to_string();
        let link = format!("https://leetcode.com/problems/{}/submissions/", slug);
        Ok(Self {
            id,
            slug,
            lang,
            link,
            typed_code: None,
        })
    }
}

pub fn extract_problem<P: AsRef<Path>>(filename: P) -> Result<Problem> {
    debug!("Filename: {:#?}", filename.as_ref());
    let reader = io::BufReader::new(std::fs::File::open(filename)?);
    let mut lines = reader.lines();
    let line = lines.next().ok_or(LeetUpError::OptNone)??;
    debug!("Line: {}", line);
    let line = line
        .get(line.find(LEETUP_MARKER).unwrap() + LEETUP_MARKER.len()..)
        .unwrap()
        .trim();
    let mut problem = Problem::from_str(line)?;
    let typed_code = lines
        .filter_map(|x| x.ok())
        .collect::<Vec<String>>()
        .join("\n")
        .to_string();
    problem.typed_code = Some(typed_code);
    debug!("{:#?}", problem);
    Ok(problem)
}
