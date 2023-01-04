use assert_cmd::prelude::*;
use predicates::str::contains;
use std::process::Command;

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::path::PathBuf;

    #[test]
    fn cli_version() {
        Command::cargo_bin("leetup")
            .unwrap()
            .args(&["-V"])
            .assert()
            .stdout(contains(env!("CARGO_PKG_VERSION")));
    }

    #[test]
    fn user() {
        // TODO add test
        assert!(true);
    }

    fn _get_id(problem: &str) -> usize {
        println!("{}", problem);
        let start_index = problem.find(" [").unwrap();
        let end_index = problem.find(']').unwrap();
        let id = problem.get(start_index + 2..end_index).unwrap().trim();
        println!("{}", id);
        id.parse().unwrap()
    }

    fn _list_problems() {
        let bytes: Vec<u8> = Command::cargo_bin("leetup")
            .unwrap()
            .args(&["list", "-oi"])
            .assert()
            .get_output()
            .stdout
            .clone();
        let result: Vec<String> = String::from_utf8(bytes)
            .unwrap()
            .split("\n")
            .map(String::from)
            .collect();

        let n = result.len() - 1;

        // Test OrderBy works by check first and last id
        //
        // NOTE: For some reason, result.last() is empty!
        assert_eq!(1, _get_id(result.get(0).as_ref().unwrap()));
        assert_eq!(n, _get_id(result.get(n - 1).as_ref().unwrap()));
    }

    #[test]
    fn pick_problem() {
        let mut response_data_path: PathBuf = std::env::current_dir().unwrap();
        response_data_path.push("tests/data/pick_problem_response.json");
        println!("Path {:#?}", response_data_path.to_str());
        let json: serde_json::Value =
            serde_json::from_reader(File::open(response_data_path).unwrap()).unwrap();
        println!("{:#?}", json);
    }

    #[test]
    fn test_problem() {
        // TODO add test
        assert!(true);
    }

    #[test]
    fn submit_problem() {
        // TODO add test
        assert!(true);
    }
}
