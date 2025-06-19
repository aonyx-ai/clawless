#![cfg_attr(not(doctest),doc = include_str!("../README.md"))]
#![warn(missing_docs)]

pub use clawless_derive::{app, command};

// Re-export the inventory crate for use with the `clawless-derive` crate
#[doc(hidden)]
pub use inventory;

/// Run an async function in an async runtime.
///
/// This function starts an asynchronous runtime and blocks until the passed
/// future is resolved. It is used internally by the `clawless!` macro to hide
/// the use of the `tokio` runtime as an implementation detail.
#[doc(hidden)]
pub fn run_async<F>(future: F)
where
    F: std::future::Future<Output = ()>,
{
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(future);
}

/// Initialize and run a Clawless application
///
/// This macro initializes a Clawless application and runs it in an asynchronous
/// runtime. It requires that the `app!` macro has been called in a submodule
/// called `commands` to initialize the application and that at least one
/// `#[command]` has been registered.
///
/// # Examples
///
/// ```rust,ignore
/// use clawless::clawless;
///
/// mod commands;
///
/// fn main() {
///     clawless!()
/// }
/// ```
#[macro_export]
#[allow(clippy::crate_in_macro_def)] // The use of `crate` is intentional
#[allow(clippy::needless_doctest_main)]
macro_rules! clawless {
    () => {
        $crate::run_async(async {
            let app = crate::commands::clawless_init();
            crate::commands::clawless_exec(app.get_matches()).await;
        });
    };
}
