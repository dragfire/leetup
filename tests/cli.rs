use std::process::Command;

use assert_cmd::prelude::*;
use predicates::str::contains;

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Read;

    use super::*;

    #[test]
    fn cli_version() {
        Command::cargo_bin("leetup")
            .unwrap()
            .args(["-V"])
            .assert()
            .stdout(contains(env!("CARGO_PKG_VERSION")));
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
            .args(["list", "-oi"])
            .assert()
            .get_output()
            .stdout
            .clone();
        let result: Vec<String> = String::from_utf8(bytes)
            .unwrap()
            .split('\n')
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
    fn pick_problem_lang_rust() {
        let bytes: Vec<u8> = Command::cargo_bin("leetup")
            .unwrap()
            .args(["pick", "1"])
            .assert()
            .get_output()
            .stdout
            .clone();
        let stripped_output = strip_ansi_escapes::strip(bytes).unwrap();
        let generated_path = String::from_utf8(stripped_output)
            .unwrap()
            .replace("Generated: ", "");
        let result = generated_path.trim_end();

        let mut generated_file = File::open(result).unwrap();
        let mut buffer = String::new();
        generated_file.read_to_string(&mut buffer).unwrap();
        assert!(buffer.contains("// @leetup=custom\n// @leetup=info id=1 lang=rust slug=two-sum"));
        assert!(buffer.contains("// @leetup=code\n"));
    }
}
