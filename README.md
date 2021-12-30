# `vpixsh`

`vpixsh` is a shell featuring:

- [`sh` compatible
  grammar](https://pubs.opengroup.org/onlinepubs/9699919799/utilities/V3_chap02.html#tag_18_10)
  (unlike `fish`)
  - (But no Here-Documents, they make no sense on the commandline)
- Syntax highlighting at the prompt
- No scripting support except what's required below, `vpixsh` should be a
  commandline first experience
- Support [the `export X=y`
  construct](https://www.gnu.org/software/bash/manual/html_node/Bourne-Shell-Builtins.html#index-export)
  because I like it
- Support [the `$()` command substitution
  syntax](https://www.gnu.org/software/bash/manual/html_node/Command-Substitution.html#Command-Substitution)
  because I like it
- The [`venv`](https://docs.python.org/3/library/venv.html) `bash` scripts
  should work:
  - [`. ./env/bin/activate`](https://github.com/pypa/virtualenv/blob/main/src/virtualenv/activation/bash/activate.sh)
    (not entirely sure about this link)
  - Prompt should now show the `(env)` prefix
  - Some `pip install` invocation
  - `deactivate`
- Consider whether we should be able to use `bash` command completions. This
  probably scales better than trying to roll our own, no matter how automated we
  can make that.
- Configuration in `~/.vpixsh/config.yaml`
  - Default environment variables
  - `$PATH` value
- `git` friendly prompt. Maybe as a separate binary?
- Command line history stored in `~/.vpixsh/history`. No suffix to this file,
  this enables us to switch formats if needed. Start out with whatever format
  `fish` is using, they probably thought this through already.
- `fzf` based fuzzy history search

Cred to
[https://random.org](https://random.org/strings/?num=10&len=4&loweralpha=on&unique=on&format=html&rnd=new)
for the name.

## Development

Do `cargo test` to run the test suite.

## TODO

### Before Johan can use it as his default shell

- Print `^^^` markers pointing out any parse errors
- Command line editing
- Show most recent exit code in the prompt
- Typing just a directory name should `cd` into that directory
- History collection
- Suggestions from history
- Arrow up to go back in history
- Smart completion
- Syntax highlighted command line
- Informative VCS prompt
- Pipes
- Handle ctrl-c as expected

### Before others can use it

- Support for `exit` command
- Print useful error diagnostics on command line parse errors
- Print a report-errors-here message on crashes and on startup
- Job control, backgrounding things with ctrl-z or `&`, `fg`, `bg`, `jobs`

### Misc

- Complete `tokenizer.rs` with support for all kinds of quoting
- Fully `bash` compatible command line parser
- Print hints on `cd` so we know where we're going on `cd ../..`
- Smart history search using `fzf` (or whatever)
- `shellcheck` command lines and show as-you-type hints
- Handle multiline input at the prompt; `for` loops, function declarations...

### DONE

- Support for ctrl-d to exit
- Main loop:
  - Print prompt
  - Read command line
  - Parse the command line we just read
  - Execute the command line
- Basic prompt with path and a `$`
- `cd` support
  - Prompt should change after `cd`
  - Spawned processes should get new CWDs after `cd`
  - `cd` should handle relative directories
  - `cd ..` should go one notch up, not add `..` to the current path
  - `cd -` should go back to the previous directory
