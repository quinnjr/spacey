//! Test command implementation.

use crate::cli::{Cli, RunArgs, TestArgs};
use crate::error::Result;

/// Run the test command.
pub async fn run(args: &TestArgs, cli: &Cli) -> Result<()> {
    let run_args = RunArgs {
        script: "test".to_string(),
        args: args.args.clone(),
        parallel: false,
        workspaces: false,
        include_workspace_root: false,
    };

    super::run::run(&run_args, cli).await
}
