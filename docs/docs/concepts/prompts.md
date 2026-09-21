---
sidebar_position: 7
---

# Prompts

A prompt asks the user a question and returns the answer to your command.

## Asking a question

[`Context`][context] provides the [`Prompt`][prompt] interface:

```rust
use clawless::prelude::*;
use clawless::prompt::Confirm;

/// Publish the release
#[command]
pub async fn publish(args: PublishArgs, context: Context) -> CommandResult {
    let question = Confirm::new("Publish the release?").with_default(Confirmation::No);

    match context.prompt().confirm(question).await? {
        Confirmation::Yes => message!("Published."),
        Confirmation::No => message!("Nothing was published."),
    }

    Ok(())
}
```

```console
$ mycli publish
Publish the release? [y/N] y
Published.
```

## Kinds of prompt

| Method    | Question               | Returns                          |
| --------- | ---------------------- | -------------------------------- |
| `confirm` | Yes or no              | [`Confirmation`][confirmation]   |
| `text`    | One line of text       | `String`                         |
| `select`  | One of several options | The option that the user selects |

The user answers each prompt with one line and the Enter key. After a line that
is not a valid answer, the terminal shows a hint and asks again.

### `confirm`

`Confirm::with_default` sets the answer that the Enter key alone gives. Without
a default, the user must type `y` or `n`.

### `text`

```rust
let title = context.prompt().text("Title of the change").await?;
```

The answer is the line as the user typed it. Clawless does not trim or validate
it, and it can be empty. To reject an answer, send a `message!` and ask again.

### `select`

```rust
let kind = context
    .prompt()
    .select("Kind of change", [Kind::Added, Kind::Changed, Kind::Fixed])
    .await?;
```

```console
Kind of change:
  1) Added
  2) Changed
  3) Fixed
Enter a number [1-3]: 3
```

`select` shows each option through its `Display` implementation. The user types
the number of an option, and `select` returns that option.

## Without a user

A user is present when the standard input and the standard error are both
terminals. Without a user, a prompt returns `PromptUserError::AbsentUser`
immediately. It does not apply a default.

```rust
use clawless::prompt::PromptUserError;

let answer = match context.prompt().confirm("Publish the release?").await {
    Ok(answer) => answer,
    Err(PromptUserError::AbsentUser { .. }) => {
        message!("Nobody can confirm the release. Run this command in a terminal.");
        return Ok(());
    }
    Err(error) => return Err(error).context("ask for the confirmation"),
};
```

[`context.interactivity()`][interactivity] returns `Interactive` or
`NonInteractive` without a prompt.

An application that uses the [`application` macro][macros] cannot prompt yet.
Its context is always `NonInteractive`.

Clawless has no flag that answers prompts. To skip a prompt, add an
[argument][arguments] to your command and do not ask when it is set.

## Output and input

A question appears after all output that your command sent before it. Output
that your command sends while a question is open appears after the answer.

The question is written to the standard error in every output mode. Under
`--json`, the standard output contains only JSON. `--json` and `--quiet` do not
suppress a question.

Clawless reads the standard input only while a question is open. At all other
times, a program that your command starts can read the terminal.

## Errors

A prompt returns a [`PromptUserError`][prompt-user-error]:

| Variant               | Cause                                                                                                           |
| --------------------- | --------------------------------------------------------------------------------------------------------------- |
| `AbsentUser`          | No user is present                                                                                              |
| `CancelledPrompt`     | The [cancellation][cancellation] token was cancelled, which Ctrl+C does                                         |
| `UnansweredPrompt`    | The user ended the input with Ctrl+D (source `ClosedInput`), or the terminal failed (source `UnusableTerminal`) |
| `UndeliverablePrompt` | Nothing receives the output of the command                                                                      |
| `MissingOptions`      | `select` received no options                                                                                    |
| `UnknownOption`       | The answer to `select` names an option that does not exist                                                      |

## Testing

[`ScriptedUser`][scripted-user] answers prompts in a test. `attend` reads the
output of the command and answers each prompt with the next `ScriptedAnswer`:

```rust
use clawless::event::event_channel;
use clawless::prelude::*;
use clawless::prompt::{ScriptedAnswer, ScriptedUser};

#[tokio::test]
async fn confirm_with_a_user_who_declines_returns_no() {
    let (sender, receiver) = event_channel();
    let user = ScriptedUser::new([ScriptedAnswer::Confirm(Confirmation::No)]);
    tokio::spawn(user.attend(receiver));

    let context = Context::builder()
        .output(Output::new(sender))
        .interactivity(Interactivity::Interactive)
        .build()
        .expect("should build the context");

    let answer = context
        .prompt()
        .confirm("Publish the release?")
        .await
        .expect("should answer");

    assert_eq!(answer, Confirmation::No);
}
```

The context must be `Interactive`. Otherwise the prompt returns `AbsentUser`. A
prompt without a matching answer returns `UnansweredPrompt` with the source
`UnscriptedPrompt`.

The [`prompt` example][example] has a command and a test for each kind of
prompt.

## What's next

- **[Context](./context)** - The other features of the context
- **[Cancellation](./cancellation)** - The token that cancels a prompt
- **[Output](./output)** - Messages, details, and artifacts

[arguments]: ./arguments
[cancellation]: ./cancellation
[confirmation]: https://docs.rs/clawless/latest/clawless/prompt/enum.Confirmation.html
[context]: ./context
[example]: https://github.com/aonyx-ai/clawless/tree/main/examples/prompt
[interactivity]: https://docs.rs/clawless/latest/clawless/context/enum.Interactivity.html
[macros]: ./macros
[prompt]: https://docs.rs/clawless/latest/clawless/prompt/struct.Prompt.html
[prompt-user-error]: https://docs.rs/clawless/latest/clawless/prompt/enum.PromptUserError.html
[scripted-user]: https://docs.rs/clawless/latest/clawless/prompt/struct.ScriptedUser.html
