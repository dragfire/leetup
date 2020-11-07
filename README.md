<h1 align="center">

![Rust](https://github.com/dragfire/leetup/workflows/Rust/badge.svg) [![Build Status](https://travis-ci.org/dragfire/leetup.svg?branch=master)](https://travis-ci.org/dragfire/leetup) [![crates](https://img.shields.io/crates/v/leetup.svg)](https://crates.io/crates/leetup) ![Downloads](https://img.shields.io/crates/d/leetup)

</h1>

<h4 align="center">Solve Leetcode problems</h4>

![](assets/leetup.gif)

## Install
- MacOs/Linux:
```sh
curl -LSfs https://japaric.github.io/trust/install.sh | sh -s -- --git dragfire/leetup
```
- Cargo:
```sh
cargo install leetup
```
- Windows:  
Download from ![releases](https://github.com/dragfire/leetup/releases). Extract the zipped x86_64 windows target file.
> Note: You will need to add `leetup.exe` to PATH to access from Command Prompt.

## Quick Start:
- Login using Github: `leetup user -g`
- Login using Cookie: `leetup user -c`
- Pick a problem: `leetup pick -l python 1`
- Test a problem: `leetup test two-sum.py -t "[1,2]\n3"`
- Submit a problem: `leetup submit two-sum.py`
- List/Show problems: `leetup list`
    - Search by keyword: `leetup list <keyword>`
    - Query easy: `leetup list -q e`
    - Order by Id, Title, Difficulty: `leetup list -qE -oIdT`
![More Commands](https://github.com/dragfire/leetup/docs/usage.md)

## Inject code fragments:
You can inject pieces of code that you frequently use in certain positions of the generated code file. Example: Standard library imports for each language can be put into a config. `Leetup` will pick it up and insert into the generated file.  

### Config:
Create `~/.leetup/config.json` and customize according to your preference:
```json
{
    "inject_code": {
        "rust": {
            "before_code": ["use std::rc::Rc;", "use std::collections::{HashMap, VecDeque};", "use std::cell::RefCell;"],
            "before_code_exclude": ["// Test comment", "// Test code"],
            "after_code": "\nstruct Solution; \n\nfn main() {\n    let solution = Solution::$func();\n\n}\n",
            "before_function_definition": null
        },
        "java": {
            "before_code": "import java.util.*;",
            "before_code_exclude": ["// Test comment", "// Test code"],
            "after_code": null,
            "before_function_definition": null
        },
        "python3": {
            "before_code": "import math",
            "before_code_exclude": ["# Test comment", "# Test code"],
            "after_code": ["if __name__ = \"__main__\":", "    solution = Solution()"],
            "before_function_definition": null
        }
    }
}
```
Generated code looks something like this in Rust:
```rust
// @leetup=custom
// @leetup=info id=1 lang=rust slug=two-sum

/*
* [SNIP]
*/
// @leetup=custom

// @leetup=inject:before_code_ex
// Test comment
// Test code
// @leetup=inject:before_code_ex

// @leetup=code

// @leetup=inject:before_code
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;
// @leetup=inject:before_code

impl Solution {
    pub fn two_sum(nums: Vec<i32>, target: i32) -> Vec<i32> {}
}
// @leetup=code

// @leetup=inject:after_code
// This is helpful when you want to run this program locally
// and avoid writing this boilerplate code for each problem.
struct Solution;

fn main() {
    let solution = Solution::two_sum();
}

// @leetup=inject:after_code
```

During testing and submitting to Leetcode, only the chunk of code between `@leetup=code` will be submitted:
```rust
// @leetup=inject:before_code
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;
// @leetup=inject:before_code

impl Solution {
    pub fn two_sum(nums: Vec<i32>, target: i32) -> Vec<i32> {
    }
}
```
Others are ignored!

### Credit:
This project is inspired by: https://github.com/leetcode-tools/leetcode-cli
