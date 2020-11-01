use crate::service::Problem;
use std::path::Path;

pub fn extract_problem<P: AsRef<Path>>(filename: P) -> Problem {
    Problem {
        slug: "two-sum".to_string(),
        id: 1,
        lang: "rust".to_string(),
        link: "https://leetcode.com/problems/two-sum/submissions/".to_string(),
    }
}

pub fn get_code(problem: &Problem) -> String {
    r#"
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
    "#
    .to_string()
}
