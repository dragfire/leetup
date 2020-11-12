use crate::{service::Problem, template::Pattern, LeetUpError, Result};
use log::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::str::FromStr;

impl FromStr for Problem {
    type Err = LeetUpError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        info!("LeetupInfo: {}", s);
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
    let mut typed_code = String::new();
    let mut file = File::open(filename)?;
    file.read_to_string(&mut typed_code)?;
    let pattern_leetup_info: String = Pattern::LeetUpInfo.into();
    let info_index = typed_code
        .find(&pattern_leetup_info)
        .map(|i| i + pattern_leetup_info.len())
        .expect("LeetUpInfo is required.");
    let line = &typed_code[info_index..].trim();
    let end_index = line.find("\n").expect("LeetupInfo needs a new line");
    let line = &line[..end_index].trim();
    let mut problem = Problem::from_str(line)?;
    problem.typed_code = Some(typed_code);
    debug!("{:#?}", problem);

    Ok(problem)
}
