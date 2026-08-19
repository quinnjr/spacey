//! Add command implementation.

use crate::cli::{AddArgs, Cli, InstallArgs};
use crate::error::Result;

/// Run the add command (alias for install with packages).
pub async fn run(args: &AddArgs, cli: &Cli) -> Result<()> {
    let install_args = InstallArgs {
        packages: args.packages.clone(),
        save_dev: args.dev,
        save_optional: args.optional,
        save_peer: args.peer,
        save_exact: args.exact,
        global: args.global,
        ..Default::default()
    };

    super::install::run(&install_args, cli).await
}
