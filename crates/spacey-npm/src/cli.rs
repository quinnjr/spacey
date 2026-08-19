//! CLI argument parsing for snpm.
//!
//! Provides NPM-compatible command line interface.

use clap::{Args, Parser, Subcommand};

/// spacey-npm (snpm) - A fast, async, multithreaded NPM-compatible package manager
#[derive(Parser, Debug)]
#[command(name = "snpm")]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Run in quiet mode (minimal output)
    #[arg(short, long, global = true)]
    pub quiet: bool,

    /// Set the registry URL
    #[arg(long, global = true, env = "NPM_REGISTRY")]
    pub registry: Option<String>,

    /// Number of concurrent downloads (default: CPU count * 2)
    #[arg(long, global = true)]
    pub concurrency: Option<usize>,

    /// Skip SSL certificate verification
    #[arg(long, global = true)]
    pub insecure: bool,

    /// Use offline mode (only use cached packages)
    #[arg(long, global = true)]
    pub offline: bool,

    /// Prefer offline mode (use cached packages when available)
    #[arg(long, global = true)]
    pub prefer_offline: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Install all dependencies from package.json
    #[command(alias = "i")]
    Install(InstallArgs),

    /// Add a package to dependencies
    Add(AddArgs),

    /// Remove a package from dependencies
    #[command(alias = "rm", alias = "uninstall", alias = "un")]
    Remove(RemoveArgs),

    /// Update packages to their latest versions
    #[command(alias = "up", alias = "upgrade")]
    Update(UpdateArgs),

    /// Create a new package.json file
    Init(InitArgs),

    /// Run a script defined in package.json
    #[command(alias = "run-script")]
    Run(RunArgs),

    /// Run the test script
    #[command(alias = "t")]
    Test(TestArgs),

    /// Publish a package to the registry
    Publish(PublishArgs),

    /// Create a tarball from a package
    Pack(PackArgs),

    /// Symlink a package folder
    Link(LinkArgs),

    /// Remove a linked package
    Unlink(UnlinkArgs),

    /// List installed packages
    #[command(alias = "ls", alias = "la", alias = "ll")]
    List(ListArgs),

    /// Check for outdated packages
    Outdated(OutdatedArgs),

    /// Search for packages
    #[command(alias = "s", alias = "find")]
    Search(SearchArgs),

    /// View package information
    #[command(alias = "view", alias = "show", alias = "v")]
    Info(InfoArgs),

    /// Manage the package cache
    Cache(CacheArgs),

    /// Manage npm configuration
    #[command(alias = "c")]
    Config(ConfigArgs),

    /// Run a security audit
    Audit(AuditArgs),

    /// Execute a shell command in the context of the project
    #[command(alias = "x")]
    Exec(ExecArgs),

    /// Bump the package version
    Version(VersionArgs),

    /// Install dependencies in CI mode (clean install)
    #[command(alias = "clean-install")]
    Ci(CiArgs),

    /// Manage the package store (pnpm-style)
    Store(StoreArgs),
}

#[derive(Args, Debug, Default, Clone)]
pub struct InstallArgs {
    /// Package names to install (if empty, installs all from package.json)
    #[arg(value_name = "PACKAGE")]
    pub packages: Vec<String>,

    /// Save as production dependency (default)
    #[arg(short = 'P', long)]
    pub save_prod: bool,

    /// Save as development dependency
    #[arg(short = 'D', long)]
    pub save_dev: bool,

    /// Save as optional dependency
    #[arg(short = 'O', long)]
    pub save_optional: bool,

    /// Save as peer dependency
    #[arg(long)]
    pub save_peer: bool,

    /// Install exact version (don't use semver range)
    #[arg(short = 'E', long)]
    pub save_exact: bool,

    /// Don't save to package.json
    #[arg(long)]
    pub no_save: bool,

    /// Install globally
    #[arg(short, long)]
    pub global: bool,

    /// Force reinstallation even if package exists
    #[arg(short, long)]
    pub force: bool,

    /// Skip running lifecycle scripts
    #[arg(long)]
    pub ignore_scripts: bool,

    /// Only install production dependencies
    #[arg(long)]
    pub production: bool,

    /// Also install devDependencies
    #[arg(long)]
    pub include_dev: bool,

    /// Create a package-lock.json file
    #[arg(long, default_value = "true")]
    pub package_lock: bool,

    /// Don't read or generate a package-lock.json
    #[arg(long)]
    pub no_package_lock: bool,

    /// Install packages without checking peer dependencies
    #[arg(long)]
    pub legacy_peer_deps: bool,

    /// Strictly enforce peer dependencies
    #[arg(long)]
    pub strict_peer_deps: bool,

    /// Don't execute any lifecycle scripts
    #[arg(long)]
    pub no_scripts: bool,

    /// Use symlinked node_modules (pnpm-style)
    #[arg(long)]
    pub symlink: bool,

    /// Use hard-linked node_modules (better compatibility than symlink)
    #[arg(long)]
    pub hardlink: bool,

    /// Auto-install missing peer dependencies
    #[arg(long, default_value = "true")]
    pub auto_install_peers: bool,

    /// Use snpm.toml lockfile format instead of package-lock.json
    #[arg(long)]
    pub toml: bool,

    /// Fail if lockfile is out of date (CI mode)
    #[arg(long)]
    pub frozen_lockfile: bool,
}

#[derive(Args, Debug, Clone)]
pub struct AddArgs {
    /// Package names to add
    #[arg(required = true)]
    pub packages: Vec<String>,

    /// Save as development dependency
    #[arg(short = 'D', long)]
    pub dev: bool,

    /// Save as optional dependency
    #[arg(short = 'O', long)]
    pub optional: bool,

    /// Save as peer dependency
    #[arg(long)]
    pub peer: bool,

    /// Install exact version
    #[arg(short = 'E', long)]
    pub exact: bool,

    /// Install globally
    #[arg(short, long)]
    pub global: bool,
}

#[derive(Args, Debug, Clone)]
pub struct RemoveArgs {
    /// Package names to remove
    #[arg(required = true)]
    pub packages: Vec<String>,

    /// Remove globally
    #[arg(short, long)]
    pub global: bool,

    /// Don't update package.json
    #[arg(long)]
    pub no_save: bool,
}

#[derive(Args, Debug, Clone)]
pub struct UpdateArgs {
    /// Package names to update (if empty, updates all)
    pub packages: Vec<String>,

    /// Update globally installed packages
    #[arg(short, long)]
    pub global: bool,

    /// Update to latest version (ignore semver)
    #[arg(long)]
    pub latest: bool,
}

#[derive(Args, Debug, Clone)]
pub struct InitArgs {
    /// Create a default package.json without prompts
    #[arg(short, long)]
    pub yes: bool,

    /// Create a scoped package
    #[arg(long)]
    pub scope: Option<String>,

    /// Force overwrite existing package.json
    #[arg(short, long)]
    pub force: bool,
}

#[derive(Args, Debug, Clone)]
pub struct RunArgs {
    /// Script name to run
    pub script: String,

    /// Arguments to pass to the script
    #[arg(trailing_var_arg = true)]
    pub args: Vec<String>,

    /// Run scripts in parallel for workspaces
    #[arg(long)]
    pub parallel: bool,

    /// Run for all workspace packages
    #[arg(long)]
    pub workspaces: bool,

    /// Include the root package when running workspaces
    #[arg(long)]
    pub include_workspace_root: bool,
}

#[derive(Args, Debug, Clone)]
pub struct TestArgs {
    /// Arguments to pass to the test command
    #[arg(trailing_var_arg = true)]
    pub args: Vec<String>,
}

#[derive(Args, Debug, Clone)]
pub struct PublishArgs {
    /// Registry to publish to
    #[arg(long)]
    pub registry: Option<String>,

    /// Tag to publish under
    #[arg(long, default_value = "latest")]
    pub tag: String,

    /// Perform a dry run
    #[arg(long)]
    pub dry_run: bool,

    /// Make package public
    #[arg(long)]
    pub access: Option<String>,
}

#[derive(Args, Debug, Clone)]
pub struct PackArgs {
    /// Directory to create the tarball in
    #[arg(long)]
    pub pack_destination: Option<String>,

    /// Perform a dry run
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Args, Debug, Clone)]
pub struct LinkArgs {
    /// Package to link (if empty, links current package)
    pub package: Option<String>,
}

#[derive(Args, Debug, Clone)]
pub struct UnlinkArgs {
    /// Package to unlink (if empty, unlinks current package)
    pub package: Option<String>,

    /// Unlink globally
    #[arg(short, long)]
    pub global: bool,
}

#[derive(Args, Debug, Clone)]
pub struct ListArgs {
    /// Only show top-level packages
    #[arg(long)]
    pub depth: Option<usize>,

    /// Output as JSON
    #[arg(long)]
    pub json: bool,

    /// List globally installed packages
    #[arg(short, long)]
    pub global: bool,

    /// Show extended information
    #[arg(short, long)]
    pub long: bool,

    /// Show parseable output
    #[arg(long)]
    pub parseable: bool,

    /// List production dependencies only
    #[arg(long)]
    pub prod: bool,

    /// List development dependencies only
    #[arg(long)]
    pub dev: bool,

    /// Show only packages that are dependencies of the specified package
    #[arg(long)]
    pub only: Option<String>,

    /// List all packages (including nested dependencies)
    #[arg(short, long)]
    pub all: bool,
}

#[derive(Args, Debug, Clone)]
pub struct OutdatedArgs {
    /// Only check specified packages
    pub packages: Vec<String>,

    /// Output as JSON
    #[arg(long)]
    pub json: bool,

    /// Check globally installed packages
    #[arg(short, long)]
    pub global: bool,

    /// Show extended information
    #[arg(short, long)]
    pub long: bool,
}

#[derive(Args, Debug, Clone)]
pub struct SearchArgs {
    /// Search terms
    #[arg(required = true)]
    pub terms: Vec<String>,

    /// Output as JSON
    #[arg(long)]
    pub json: bool,

    /// Limit number of results
    #[arg(long, default_value = "20")]
    pub limit: usize,

    /// Show extended information
    #[arg(short, long)]
    pub long: bool,
}

#[derive(Args, Debug, Clone)]
pub struct InfoArgs {
    /// Package name
    pub package: String,

    /// Specific field to show
    pub field: Option<String>,

    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

#[derive(Args, Debug, Clone)]
pub struct CacheArgs {
    #[command(subcommand)]
    pub action: CacheAction,
}

#[derive(Subcommand, Debug, Clone)]
pub enum CacheAction {
    /// Add a package to the cache
    Add { packages: Vec<String> },
    /// Clean the cache
    Clean,
    /// List cached packages
    #[command(alias = "ls")]
    List,
    /// Verify cache integrity
    Verify,
    /// Show cache directory path
    Path,
}

#[derive(Args, Debug, Clone)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub action: ConfigAction,
}

#[derive(Subcommand, Debug, Clone)]
pub enum ConfigAction {
    /// Set a config value
    Set { key: String, value: String },
    /// Get a config value
    Get { key: String },
    /// Delete a config value
    Delete { key: String },
    /// List all config values
    List,
    /// Edit the config file
    Edit,
}

#[derive(Args, Debug, Clone)]
pub struct AuditArgs {
    /// Output as JSON
    #[arg(long)]
    pub json: bool,

    /// Only audit production dependencies
    #[arg(long)]
    pub production: bool,

    /// Attempt to fix vulnerabilities
    #[arg(long)]
    pub fix: bool,

    /// Force fixes even if breaking changes are required
    #[arg(long)]
    pub force: bool,

    /// Perform a dry run of fixes
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Args, Debug, Clone)]
pub struct ExecArgs {
    /// Command to execute
    pub command: String,

    /// Arguments to pass to the command
    #[arg(trailing_var_arg = true)]
    pub args: Vec<String>,

    /// Package to use for the command
    #[arg(short, long)]
    pub package: Option<String>,
}

#[derive(Args, Debug, Clone)]
pub struct VersionArgs {
    /// Version bump type or explicit version
    pub version: Option<String>,

    /// Don't commit the version change
    #[arg(long)]
    pub no_git_tag_version: bool,

    /// Allow version command on dirty working directory
    #[arg(long)]
    pub allow_same_version: bool,

    /// Message for version commit
    #[arg(short, long)]
    pub message: Option<String>,
}

#[derive(Args, Debug, Clone)]
pub struct CiArgs {
    /// Skip running lifecycle scripts
    #[arg(long)]
    pub ignore_scripts: bool,

    /// Only install production dependencies
    #[arg(long)]
    pub production: bool,
}

#[derive(Args, Debug, Clone)]
pub struct StoreArgs {
    #[command(subcommand)]
    pub action: StoreAction,
}

#[derive(Subcommand, Debug, Clone)]
pub enum StoreAction {
    /// Show store path
    Path,
    /// Show store status and statistics
    Status,
    /// Prune unused packages from the store
    Prune {
        /// Perform a dry run
        #[arg(long)]
        dry_run: bool,
    },
    /// Add packages to the store
    Add {
        /// Package specifiers to add
        packages: Vec<String>,
    },
    /// List packages in the store
    #[command(alias = "ls")]
    List {
        /// Filter by package name
        #[arg(long)]
        filter: Option<String>,
    },
}
