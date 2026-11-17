---
sidebar_position: 3
---

# Output

Output is the mechanism Clawless uses for framework-controlled command output.
Rather than printing directly to stdout, commands route all user-facing text
through an [`Output`][output-type] instance, which automatically provides
`--quiet`, `--verbose`, and `--json` support.

## What is output?

Output is an abstraction between your command and the terminal. Instead of
calling `println!` directly, you call methods on the [`Output`][output-type]
type that the framework provides through [`Context`][context]. The framework
then decides where and whether to write each message based on the flags the
user passed.

This matters for commands that produce structured data, need verbosity control,
or must work in scripting pipelines. Framework-controlled output lets your
command focus on what to say while the framework handles how to say it.

## How it works

Access the [`Output`][output-type] instance through context:

```rust
use clawless::prelude::*;

/// Count words in a sentence
#[command]
pub async fn count(args: CountArgs, context: Context) -> CommandResult {
    let output = context.output();

    output.detail(format!("input: {}", args.sentence));
    output.message("counting words");
    output.artifact(&WordCount { words: 42 });

    Ok(())
}
```

The framework adds `--quiet`, `--verbose`, and `--json` flags to every command
automatically. Your command doesn't need to declare or parse them.

## The three methods

### `message`

Writes an informational message. This is the replacement for `println!` in most
cases:

```rust
output.message("deploying to production");
output.message(format!("found {} items", count));
```

Messages from `message` are suppressed when the user passes `--quiet`.

### `detail`

Writes a message that only appears with `--verbose`. Use this for additional
detail that is helpful when debugging but noisy in normal usage:

```rust
output.detail(format!("loading config from {}", path.display()));
output.detail("retrying connection");
```

### `artifact`

Writes the primary data a command produces. In text mode, the value is
formatted via [`Display`][display]. In JSON mode, it is serialized via
[`Serialize`][serialize]:

```rust
use serde::Serialize;
use std::fmt;

#[derive(Serialize)]
struct WordCount {
    words: usize,
}

impl fmt::Display for WordCount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.words)
    }
}

output.artifact(&WordCount { words: 42 });
```

Artifacts are always written, regardless of verbosity. This ensures that machine
consumers always get the data they need.

### Styled Artifacts

An artifact can offer a styled form next to its plain form, for example a
diagnostic with colors. The presenter chooses the form, because only the
presenter knows where the text goes. The artifact implements
[`TextForm`][text-form] in place of [`Display`][display] and writes the form
that the [`Styling`][styling] asks for. A type with `TextForm`, `Serialize`,
and `Debug` is an [`Artifact`][artifact-trait]:

```rust
use std::fmt;

use clawless::event::{Styling, TextForm};
use serde::Serialize;

#[derive(Debug, Serialize)]
struct Verdict {
    passed: bool,
}

impl TextForm for Verdict {
    fn fmt_text(&self, f: &mut fmt::Formatter<'_>, styling: Styling) -> fmt::Result {
        let word = match self.passed {
            true => "passed",
            false => "failed",
        };

        match styling {
            Styling::Styled => write!(f, "\x1b[1m{word}\x1b[0m"),
            Styling::Plain => f.write_str(word),
        }
    }
}

artifact!(Verdict { passed: true });
```

Such a type does not implement `Display`, because every type with `Display`
already has a `TextForm` with one form. A value that contains another value
calls `fmt_text` on it with the same styling.

That same rule limits the wrappers of a styled type:

- `Box<dyn Artifact>` is an artifact and writes the styled form of its value,
  so a command can pass on an artifact that it received as a box.
- `Box<Verdict>`, `&'static Verdict`, and `Arc<Verdict>` are not artifacts.
  Clawless cannot forward `TextForm` through them, because each of them has
  `Display` when its value has it. Your crate can implement `TextForm` for
  `Box<Verdict>` or `&Verdict` itself and forward to the value. An `Arc` needs a
  newtype of your crate around it.
- `Arc<dyn Artifact>` is not an artifact either. Keep the artifact in a `Box`
  if a command passes it on.
- A styled value cannot be an argument of `format!`, because it has no
  `Display`. Pass the value on whole, or call `to_text` with a styling.

## Behavior matrix

The interaction of method, verbosity, and mode:

| Method     | Default    | `--quiet`  | `--verbose` | `--json`         |
| ---------- | ---------- | ---------- | ----------- | ---------------- |
| `message`  | stdout     | suppressed | stdout      | stderr           |
| `detail`   | suppressed | suppressed | stdout      | suppressed       |
| `artifact` | stdout     | stdout     | stdout      | stdout (as JSON) |

In JSON mode, `message` and `detail` output is redirected to stderr so that
stdout contains only machine-readable JSON from `artifact`. This follows the
same convention as `gh`, `kubectl`, and `jq`.

## Output macros

The `message!`, `detail!`, and `artifact!` macros provide shorthand for the
corresponding method calls. Each macro resolves `context` at the call site, so
you do not need to extract the [`Output`][output-type] instance yourself.

Here is the count example rewritten with macros:

```rust
use clawless::prelude::*;

/// Count words in a sentence
#[command]
pub async fn count(args: CountArgs, context: Context) -> CommandResult {
    detail!("input: {}", args.sentence);
    message!("counting words");
    artifact!(WordCount { words: 42 });

    Ok(())
}
```

The macros expand to `context.output().message(...)`, `context.output().detail(...)`,
and `context.output().artifact(...)` respectively. They resolve `context` from
the enclosing function scope — every `#[command]` function receives a `context`
parameter, so no extra plumbing is needed.

## When to use each method

**`message`** for informational messages that describe what the command is doing.
Status updates, progress notes, and confirmations belong here.

**`detail`** for information that helps with debugging or understanding internals.
Raw inputs, resolved paths, retry attempts, and timing information belong here.

**`artifact`** for the primary output of the command. If your command produces
data that another program might consume, use `artifact`. The value must implement
both [`Display`][display] and [`Serialize`][serialize], or implement
[`TextForm`][text-form] in place of `Display` to offer a styled form.

Most commands only need `message`. Commands that produce structured data use
`artifact`. Commands with complex internals add `detail` for observability.

## What's next

- **[Rendering](./rendering)** - How output events reach the terminal through
  presenters and projections
- **[Context](./context)** - The Context system that provides output to
  commands
- **[Cancellation](./cancellation)** - Cooperative shutdown through
  cancellation tokens

[output-type]: https://docs.rs/clawless/latest/clawless/output/struct.Output.html
[artifact-trait]: https://docs.rs/clawless-core/latest/clawless_core/event/trait.Artifact.html
[context]: ./context
[display]: https://doc.rust-lang.org/std/fmt/trait.Display.html
[serialize]: https://docs.rs/serde/latest/serde/trait.Serialize.html
[styling]: https://docs.rs/clawless-core/latest/clawless_core/event/enum.Styling.html
[text-form]: https://docs.rs/clawless-core/latest/clawless_core/event/trait.TextForm.html
