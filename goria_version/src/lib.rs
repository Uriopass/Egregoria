#[allow(clippy::manual_unwrap_or)]
pub const VERSION: &str = {
    match option_env!("VERGEN_GIT_SHA") {
        Some(x) => x,
        None => "undefined_version",
    }
};
