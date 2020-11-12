#[derive(Copy, Clone)]
pub enum Pattern {
    LeetUpInfo,
    CustomCode,
    Code,
    InjectCodePosition(InjectPosition),
}

#[derive(Copy, Clone)]
pub enum InjectPosition {
    BeforeCode, // Helpful for imports
    BeforeCodeExclude,
    AfterCode,
    BeforeFunctionDefinition,
}

impl From<Pattern> for String {
    fn from(p: Pattern) -> Self {
        match p {
            Pattern::LeetUpInfo => "@leetup=info".into(),
            Pattern::CustomCode => "@leetup=custom".into(),
            Pattern::Code => "@leetup=code".into(),
            Pattern::InjectCodePosition(pos) => match pos {
                InjectPosition::BeforeCode => "@leetup=inject:before_code".into(),
                InjectPosition::BeforeCodeExclude => "@leetup=inject:before_code_ex".into(),
                InjectPosition::AfterCode => "@leetup=inject:after_code".into(),
                InjectPosition::BeforeFunctionDefinition => {
                    "@leetup=inject:before_function_definition".into()
                }
            },
        }
    }
}

impl<'a> From<&'a Pattern> for String {
    fn from(p: &Pattern) -> Self {
        String::from(*p)
    }
}

impl ToString for Pattern {
    fn to_string(&self) -> String {
        String::from(*self)
    }
}

/// Parse code to submit only the relevant chunk of code.
///
/// Ignore generated code definition and custom injected code for
/// testing purposes.
pub fn parse_code(code: &str) -> Option<String> {
    let code_pattern: String = Pattern::Code.into();
    let len = code_pattern.len();

    let start_index = match code.find(&code_pattern) {
        Some(index) => index + len,
        None => 0,
    };

    let code = code.get(start_index..)?;

    let end_index = match code.find(&code_pattern) {
        Some(index) => {
            let code = &code[..index];
            let index = code.rfind("\n").unwrap();
            index + 1
        }
        None => code.len(),
    };
    let code = code.get(..end_index)?;

    Some(code.into())
}

#[test]
fn test_parse_with_comments() {
    let code = r#"
// @leetup=custom
// Given an array of integers `nums` and an integer `target`, return *indices of
// the two numbers such that they add up to `target`*.
//
// You may assume that each input would have *exactly* one solution, and you
// may not use the *same* element twice.
//
// You can return the answer in any order.
//
//
// Example 1:
//
// Input: nums = [2,7,11,15], target = 9
// Output: [0,1]
// Output: Because nums[0] + nums[1] == 9, we return [0, 1].
//
// Example 2:
//
// Input: nums = [3,2,4], target = 6
// Output: [1,2]
//
// Example 3:
//
// Input: nums = [3,3], target = 6
// Output: [0,1]
//
//
// Constraints:
//
// * `2 <= nums.length <= 105`
// * `-109 <= nums[i] <= 109`
// * `-109 <= target <= 109`
// * Only one valid answer exists.
// @leetup=custom

// @leetup=code
use std::collections::HashMap;

impl Solution {
    pub fn two_sum(nums: Vec<i32>, target: i32) -> Vec<i32> {
        let mut index_map = HashMap::new();

        for (i, num) in nums.iter().enumerate() {
            let y = target - num;
            if let Some(&idx) = index_map.get(&y) {
                return vec![idx as i32, i as i32];
            }

            index_map.insert(num, i);
        }

        vec![]
    }
}
// @leetup=code
"#;

    let expected_code = r#"
use std::collections::HashMap;

impl Solution {
    pub fn two_sum(nums: Vec<i32>, target: i32) -> Vec<i32> {
        let mut index_map = HashMap::new();

        for (i, num) in nums.iter().enumerate() {
            let y = target - num;
            if let Some(&idx) = index_map.get(&y) {
                return vec![idx as i32, i as i32];
            }

            index_map.insert(num, i);
        }

        vec![]
    }
}
"#;

    let actual_code = parse_code(code);
    assert_eq!(actual_code, Some(expected_code.into()));
}

#[test]
fn test_parse_just_code() {
    let code = r#"
use std::collections::HashMap;

impl Solution {
    pub fn two_sum(nums: Vec<i32>, target: i32) -> Vec<i32> {
        let mut index_map = HashMap::new();

        for (i, num) in nums.iter().enumerate() {
            let y = target - num;
            if let Some(&idx) = index_map.get(&y) {
                return vec![idx as i32, i as i32];
            }

            index_map.insert(num, i);
        }

        vec![]
    }
}
"#;

    let expected_code = r#"
use std::collections::HashMap;

impl Solution {
    pub fn two_sum(nums: Vec<i32>, target: i32) -> Vec<i32> {
        let mut index_map = HashMap::new();

        for (i, num) in nums.iter().enumerate() {
            let y = target - num;
            if let Some(&idx) = index_map.get(&y) {
                return vec![idx as i32, i as i32];
            }

            index_map.insert(num, i);
        }

        vec![]
    }
}
"#;

    let actual_code = parse_code(code);
    assert_eq!(actual_code, Some(expected_code.into()));
}
