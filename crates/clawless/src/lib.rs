pub use clawless_derive::{app, command};

// Re-export the inventory crate for use with the `clawless-derive` crate
#[doc(hidden)]
pub use inventory;

/// Run an async function in the Clawless runtime
pub fn run_async<F>(future: F)
where
    F: std::future::Future<Output = ()>,
{
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(future);
}

#[macro_export]
macro_rules! clawless {
    () => {
        $crate::run_async(async {
            let app = commands::clawless_init();
            commands::clawless_exec(app.get_matches()).await;
        });
    };
}
