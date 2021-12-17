# `vpixsh`

`vpixsh` is a shell featuring:

- [`sh` compatible
  grammar](https://pubs.opengroup.org/onlinepubs/9699919799/utilities/V3_chap02.html#tag_18_10)
  (unlike `fish`)
  - No Here-Documents, they make no sense on the commandline
  - [An
    "operator"](https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/V1_chap03.html#tag_03_260)
    is either a [control
    operator](https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/V1_chap03.html#tag_03_113)
    or a [redirection
    operator](https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/V1_chap03.html#tag_03_318).
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

- `bash` compatible command line parser
- Use `vpixsh` as my default shell
