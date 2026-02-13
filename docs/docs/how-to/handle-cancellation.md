---
sidebar_position: 6
---

# Handle cancellation

This guide shows practical patterns for responding to cancellation in your
Clawless commands. Each section addresses a specific scenario you might
encounter.

## Wait for cancellation

The simplest pattern is a command that blocks until the user presses Ctrl+C.
Use `cancelled().await` to wait asynchronously:

```rust
use clawless::prelude::*;

#[derive(Debug, Args)]
pub struct ServeArgs {}

/// Start a server and wait for shutdown
#[command]
pub async fn serve(_args: ServeArgs, context: Context) -> CommandResult {
    println!("server started, press Ctrl+C to stop");

    context.cancellation().cancelled().await;

    println!("shutting down");
    Ok(())
}
```

If the token is already cancelled when you call `cancelled().await`, the future
completes immediately.

## Check cancellation in a loop

For CPU-bound or iterative work, use `is_cancelled()` to poll the token at
each iteration:

```rust
use clawless::prelude::*;

#[derive(Debug, Args)]
pub struct MigrateArgs {}

/// Run database migrations
#[command]
pub async fn migrate(_args: MigrateArgs, context: Context) -> CommandResult {
    let cancellation = context.cancellation();
    let migrations = discover_migrations()?;

    for migration in &migrations {
        if cancellation.is_cancelled() {
            println!("cancelled, stopping after last completed migration");
            break;
        }

        run_migration(migration)?;
        println!("applied: {}", migration.name());
    }

    Ok(())
}
```

This pattern ensures each migration either completes fully or isn't started.

## Use `tokio::select!` with cancellation

To race cancellation against an async operation, use `tokio::select!`. This is
the most common pattern for commands that perform long-running async work:

```rust
use clawless::prelude::*;

#[derive(Debug, Args)]
pub struct FetchArgs {
    /// URL to fetch
    url: String,
}

/// Fetch a URL with cancellation support
#[command]
pub async fn fetch(args: FetchArgs, context: Context) -> CommandResult {
    let cancellation = context.cancellation();

    tokio::select! {
        result = do_fetch(&args.url) => {
            let body = result.context("fetch failed")?;
            println!("{body}");
        }
        _ = cancellation.cancelled() => {
            println!("fetch cancelled");
        }
    }

    Ok(())
}
```

The `tokio::select!` macro waits for whichever branch completes first. If the
user presses Ctrl+C before the fetch finishes, the cancellation branch runs and
the fetch future is dropped.

## Scope cancellation with child tokens

When your command spawns independent sub-tasks, create child tokens so each
sub-task can be cancelled independently without stopping the parent:

```rust
use clawless::prelude::*;

#[derive(Debug, Args)]
pub struct WatchArgs {}

/// Watch for changes and rebuild on each change
#[command]
pub async fn watch(_args: WatchArgs, context: Context) -> CommandResult {
    let cancellation = context.cancellation();

    loop {
        if cancellation.is_cancelled() {
            break;
        }

        let build_token = cancellation.child();

        tokio::select! {
            _result = build(&build_token) => {
                println!("build completed");
            }
            _ = wait_for_change() => {
                // New change detected, cancel the current build
                build_token.cancel();
                println!("change detected, restarting build");
            }
            _ = cancellation.cancelled() => {
                println!("shutting down");
            }
        }
    }

    Ok(())
}
```

In this example, each build gets its own child token. When a file change is
detected, the current build's token is cancelled without affecting the parent.
When the user presses Ctrl+C, the parent token is cancelled and the
cancellation cascades to the active build.

## See also

- **[Cancellation](../concepts/cancellation)** - How cancellation works and
  why Clawless uses cooperative shutdown
- **[Context](../concepts/context)** - The Context system that provides
  cancellation to commands
