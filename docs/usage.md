## Help
```markdown
❯ leetup --help

USAGE:
    leetup <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    help      Prints this message or the help of the given subcommand(s)
    list      List questions
    pick      Pick a problem
    submit    Submit a problem
    test      Submit a problem
    user      User auth
```

## List
```markdown
❯ leetup list --help

List questions

USAGE:
    leetup list [FLAGS] [OPTIONS] [keyword]

FLAGS:
    -h, --help       Prints help information
    -s, --stat       Show statistic counter of the output list
    -V, --version    Prints version information

OPTIONS:
    -o, --order <order>    Order by ProblemId, Question Title, or Difficulty
    -q, --query <query>    Query by conditions
    -t, --tag <tag>        Filter by given tag

ARGS:
    <keyword>
```

## Pick
```markdown
❯ leetup pick --help

Pick a problem

USAGE:
    leetup pick [FLAGS] [OPTIONS] [id]

FLAGS:
    -d               Include problem definition in generated source file
    -g               Generate code if true
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -l, --lang <lang>    Language used to generate problem's source [default: rust]

ARGS:
    <id>    Show/Pick a problem using ID
```

## Submit
```markdown
❯ leetup submit --help

Submit a problem

USAGE:
    leetup submit <filename>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

ARGS:
    <filename>    Code filename
```

## Test
```markdown
❯ leetup test --help

Test a problem

USAGE:
    leetup test <filename> -t <test-data>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -t <test-data>        Custom test cases

ARGS:
    <filename>    Code filename
```

## User
```markdown
❯ leetup user --help

User auth

USAGE:
    leetup user [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -c, --cookie <cookie>    Login using cookie
    -g, --github <github>    Login using github
    -l, --logout <logout>    Logout user
```
