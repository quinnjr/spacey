<!-- converted from Cursor rules -->

## Cursor rule: `.cursor/rules/async-parallel.mdc`

# Async and Parallel Optimization

Always optimize for asynchronous and parallel execution using Tokio and Rayon.

## Async-First Design

Prefer async operations for:
- File I/O (`tokio::fs` instead of `std::fs`)
- Network operations
- Any operation that may block
- Inter-task communication

```rust
// Good: Async file reading
use tokio::fs;
let content = fs::read_to_string(path).await?;

// Avoid: Blocking file reading in async context
use std::fs;
let content = fs::read_to_string(path)?; // Blocks the executor!
```

## Tokio Best Practices

### Runtime Configuration
```rust
#[tokio::main]
async fn main() {
    // Use multi-threaded runtime by default
}

// For CPU-bound work, use spawn_blocking
let result = tokio::task::spawn_blocking(|| {
    expensive_computation()
}).await?;
```

### Concurrency Patterns
```rust
// Good: Parallel async operations
let (result1, result2) = tokio::join!(
    async_operation1(),
    async_operation2(),
);

// Good: Concurrent stream processing
use futures::stream::{self, StreamExt};
stream::iter(items)
    .buffer_unordered(10)
    .collect::<Vec<_>>()
    .await;
```

### Avoid Blocking in Async
```rust
// Bad: Blocking in async context
async fn bad_example() {
    std::thread::sleep(Duration::from_secs(1)); // Blocks executor!
}

// Good: Use async sleep
async fn good_example() {
    tokio::time::sleep(Duration::from_secs(1)).await;
}
```

## Rayon for CPU-Bound Work

Use Rayon for data-parallel operations:

```rust
use rayon::prelude::*;

// Good: Parallel iteration
let results: Vec<_> = items.par_iter()
    .map(|item| expensive_transform(item))
    .collect();

// Good: Parallel sorting
let mut data = vec![...];
data.par_sort();

// Good: Parallel reduction
let sum: i64 = numbers.par_iter().sum();
```

### Rayon Thresholds
Only parallelize when beneficial:

```rust
const PARALLEL_THRESHOLD: usize = 1000;

if items.len() >= PARALLEL_THRESHOLD {
    items.par_iter().for_each(|item| process(item));
} else {
    items.iter().for_each(|item| process(item));
}
```

## Thread Safety Requirements

### Use Arc for Shared Ownership
```rust
// Good: Arc for cross-thread sharing
use std::sync::Arc;
let shared_data = Arc::new(data);

// Avoid: Rc in multi-threaded context
use std::rc::Rc; // Not Send!
```

### Prefer Lock-Free When Possible
```rust
// Good: Atomic operations
use std::sync::atomic::{AtomicUsize, Ordering};
counter.fetch_add(1, Ordering::Relaxed);

// Good: Lock-free queues
use crossbeam::queue::SegQueue;
let queue = SegQueue::new();

// Good: Concurrent maps
use dashmap::DashMap;
let map = DashMap::new();
```

### When Locks Are Needed
```rust
// Good: parking_lot for faster mutexes
use parking_lot::{Mutex, RwLock};

// Prefer RwLock for read-heavy workloads
let data = RwLock::new(value);

// Keep critical sections small
{
    let guard = data.write();
    // Minimal work here
}
```

## Combining Async and Parallel

Bridge between Tokio and Rayon carefully:

```rust
// Good: Use spawn_blocking for rayon work
async fn parallel_process(items: Vec<Item>) -> Vec<Result> {
    tokio::task::spawn_blocking(move || {
        items.par_iter()
            .map(|item| process(item))
            .collect()
    }).await.unwrap()
}

// Good: Async with parallel file processing
async fn process_files(paths: Vec<PathBuf>) -> Vec<Content> {
    let contents = futures::future::join_all(
        paths.iter().map(|p| tokio::fs::read_to_string(p))
    ).await;

    tokio::task::spawn_blocking(move || {
        contents.into_par_iter()
            .filter_map(|c| c.ok())
            .map(|c| parse_content(&c))
            .collect()
    }).await.unwrap()
}
```

## Required Dependencies

Ensure these are in Cargo.toml:

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
rayon = "1"
futures = "0.3"
parking_lot = "0.12"
crossbeam = "0.8"
```

## Performance Considerations

1. **Measure before optimizing** - Not all operations benefit from parallelism
2. **Consider overhead** - Thread spawning has costs; batch small operations
3. **Avoid contention** - Design to minimize shared mutable state
4. **Use appropriate granularity** - Too fine-grained parallelism adds overhead
5. **Profile with real workloads** - Synthetic benchmarks can be misleading

## When NOT to Parallelize

- Operations with heavy synchronization requirements
- I/O-bound work (use async instead)
- Small data sets (< 1000 items typically)
- Operations with dependencies between iterations
- When memory bandwidth is the bottleneck


## Cursor rule: `.cursor/rules/commit-on-complete.mdc`

_Commit after completing each task_

Applies to: `*`

# Commit After Task Completion

After completing any task, create a git commit with the changes.

## Workflow

1. Complete the requested task
2. Stage all relevant changed files
3. Create a commit with a conventional commit message
4. Inform the user the commit was made

## Commit Guidelines

- Follow git-flow conventional commit format (see git-flow.mdc)
- Use appropriate type: `feat`, `fix`, `docs`, `refactor`, `test`, `chore`, etc.
- Include scope when applicable (e.g., `lexer`, `parser`, `repl`)
- Write clear, concise commit messages

## Example Workflow

```bash
# After implementing a feature
git add -A
git commit -m "feat(repl): add tab completion for keywords"

# After fixing a bug
git add -A
git commit -m "fix(parser): handle trailing commas in arrays"

# After refactoring
git add -A
git commit -m "refactor(vm): extract opcode dispatch logic"
```

## When NOT to Commit

- If the task is incomplete or broken
- If there are failing tests that should pass
- If the user explicitly asks not to commit
- If only exploring/investigating (no actual changes)

## Multiple Related Changes

If a task involves multiple logical changes, prefer atomic commits:
- One commit per logical unit of work
- Each commit should leave the codebase in a working state


## Cursor rule: `.cursor/rules/documentation.mdc`

_Documentation structure and location conventions_

Applies to: `*`

# Documentation Conventions

All project documentation must be placed in the `docs/` folder at the project root.

## Documentation Location

- **All markdown documentation** goes in `docs/`
- **Do not create** documentation files in the project root (except `README.md` and `LICENSE`)
- **Do not create** documentation alongside source code in `src/` or `crates/`

## Folder Structure

```
docs/
├── architecture/       # System design and architecture docs
├── api/               # API documentation and references
├── guides/            # How-to guides and tutorials
├── specs/             # Technical specifications
└── contributing/      # Contribution guidelines
```

## Exceptions

The following files may exist at the project root:
- `README.md` - Project overview and quick start
- `LICENSE` - License file
- `CHANGELOG.md` - Version history
- `TODO.md` - Development roadmap

## When Creating Documentation

1. Place new documentation in the appropriate `docs/` subdirectory
2. Use descriptive filenames in kebab-case (e.g., `bytecode-format.md`)
3. Link from `README.md` to detailed docs when appropriate
4. Keep `README.md` concise - detailed information belongs in `docs/`

## Documentation Types

| Type | Location | Purpose |
|------|----------|---------|
| Architecture | `docs/architecture/` | System design, component diagrams |
| API Reference | `docs/api/` | Public API documentation |
| Guides | `docs/guides/` | Tutorials, how-to guides |
| Specifications | `docs/specs/` | ECMAScript compliance, bytecode format |
| Contributing | `docs/contributing/` | Development setup, code style |


## Cursor rule: `.cursor/rules/git-flow.mdc`

_Git-flow commit conventions for the Spacey project_

Applies to: `*`

# Git-Flow Commit Conventions

This project follows git-flow branching and conventional commit message standards.

## Commit Message Format

All commits must follow the conventional commits specification:

```
<type>(<scope>): <subject>

[optional body]

[optional footer(s)]
```

### Types

| Type | Description |
|------|-------------|
| `feat` | A new feature |
| `fix` | A bug fix |
| `docs` | Documentation only changes |
| `style` | Code style changes (formatting, missing semicolons, etc.) |
| `refactor` | Code change that neither fixes a bug nor adds a feature |
| `perf` | Performance improvements |
| `test` | Adding or updating tests |
| `build` | Changes to build system or dependencies |
| `ci` | Changes to CI configuration |
| `chore` | Other changes that don't modify src or test files |
| `revert` | Reverts a previous commit |

### Scopes

Use the crate or module name as scope when applicable:

- `lexer` - Lexer/tokenizer changes
- `parser` - Parser changes
- `ast` - AST structure changes
- `compiler` - Bytecode compiler changes
- `vm` - Virtual machine changes
- `runtime` - Runtime/value system changes
- `builtins` - Built-in objects/functions
- `gc` - Garbage collector changes
- `repl` - REPL interface changes
- `cli` - CLI argument handling
- `node` - Node.js bindings (spacey-node)

### Subject Line Rules

- Use imperative mood ("add" not "added" or "adds")
- Don't capitalize the first letter
- No period at the end
- Maximum 50 characters

### Examples

```
feat(lexer): add support for template literals
fix(parser): handle trailing commas in array literals
docs(readme): update build instructions
refactor(vm): extract opcode dispatch into separate module
test(parser): add tests for arrow function parsing
perf(gc): implement incremental marking
chore: update dependencies
```

## Branch Naming

Follow git-flow branch naming conventions:

| Branch Type | Pattern | Example |
|-------------|---------|---------|
| Feature | `feature/<description>` | `feature/template-literals` |
| Bugfix | `bugfix/<description>` | `bugfix/array-bounds-check` |
| Hotfix | `hotfix/<description>` | `hotfix/critical-gc-crash` |
| Release | `release/<version>` | `release/0.2.0` |
| Main | `main` | Production-ready code |
| Develop | `develop` | Integration branch |

## Breaking Changes

For breaking changes, add `!` after the type/scope and include a `BREAKING CHANGE:` footer:

```
feat(runtime)!: change Value enum representation

BREAKING CHANGE: Value::Number now uses f64 instead of tagged union.
All code using pattern matching on Value must be updated.
```


## Cursor rule: `.cursor/rules/no-summaries.mdc`

_Prevent automatic generation of summary documents_

Applies to: `*`

# No Summary Documents

Do not automatically generate summary or recap documents.

## Prohibited Actions

- Do not create markdown files summarizing work performed
- Do not create "summary.md", "recap.md", "notes.md", or similar files
- Do not create documentation files unless explicitly requested
- Do not generate changelogs or release notes unless asked

## When Documentation IS Appropriate

Only create documentation when the user explicitly:
- Asks for documentation to be written
- Requests a specific document by name
- Asks you to "document" a feature or API

## Instead of Summaries

- Provide brief inline responses about what was done
- Use code comments for implementation notes
- Update existing documentation files when relevant


## Cursor rule: `.cursor/rules/rust-style.mdc`

_Rust 2024 edition syntax and styling conventions_

Applies to: `**/*.rs`

# Rust 2024 Style Guide

This project uses Rust 2024 edition. Follow these conventions for consistent, idiomatic code.

## Edition 2024 Features

Prefer these Rust 2024 patterns:

### Use `gen` blocks for iterators (when stabilized)
```rust
// Prefer generator syntax for complex iterators
let iter = gen {
    for item in collection {
        yield transform(item);
    }
};
```

### Precise capturing in closures
Rust 2024 captures only the fields used, not entire structs:
```rust
// This now only captures `self.field`, not all of `self`
let closure = || self.field.method();
```

### RPIT lifetime capture rules
Return position impl Trait now captures all in-scope lifetimes by default.

## Code Style

### Formatting
- Use `rustfmt` defaults - run `cargo fmt` before committing
- Maximum line width: 100 characters
- Use 4 spaces for indentation (no tabs)

### Imports
```rust
// Group imports in this order, separated by blank lines:
// 1. std library
use std::collections::HashMap;
use std::fmt;

// 2. External crates
use bumpalo::Bump;
use rustc_hash::FxHashMap;

// 3. Crate modules
use crate::runtime::Value;
use crate::lexer::Token;

// 4. Super/self
use super::Parser;
```

### Naming Conventions
| Item | Convention | Example |
|------|------------|---------|
| Types, Traits | PascalCase | `TokenKind`, `AstVisitor` |
| Functions, Methods | snake_case | `parse_expression`, `to_string` |
| Variables, Fields | snake_case | `token_stream`, `current_pos` |
| Constants | SCREAMING_SNAKE_CASE | `MAX_STACK_SIZE` |
| Lifetimes | short lowercase | `'a`, `'src`, `'ctx` |
| Type Parameters | single uppercase or descriptive | `T`, `Item`, `Node` |

### Enums
```rust
// Use descriptive variant names
pub enum Value {
    Undefined,
    Null,
    Boolean(bool),
    Number(f64),
    String(String),  // Not Str or StringVal
}

// Match exhaustively, avoid wildcards when possible
match value {
    Value::Undefined => ...,
    Value::Null => ...,
    Value::Boolean(b) => ...,
    Value::Number(n) => ...,
    Value::String(s) => ...,
}
```

### Error Handling
```rust
// Define specific error types, not just String
#[derive(Debug, Clone)]
pub enum ParseError {
    UnexpectedToken { expected: TokenKind, found: TokenKind, span: Span },
    UnexpectedEof { expected: &'static str },
    InvalidSyntax { message: String, span: Span },
}

// Implement std::error::Error
impl std::error::Error for ParseError {}

// Use Result<T, E> for fallible operations
pub fn parse(&mut self) -> Result<Ast, ParseError> { ... }

// Use ? operator for propagation
let token = self.expect(TokenKind::Semicolon)?;

// Reserve .unwrap() and .expect() for truly impossible cases
let value = map.get(&key).expect("key was just inserted");
```

### Option Handling
```rust
// Prefer combinators over match when appropriate
let name = user.name.as_ref().map(|n| n.to_uppercase());

// Use if-let for single-arm matches
if let Some(value) = optional_value {
    process(value);
}

// Use let-else for early returns (Rust 2024 stable)
let Some(value) = optional_value else {
    return Err(Error::MissingValue);
};
```

### Structs
```rust
// Derive common traits
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

// Use Default when there's a sensible default
#[derive(Default)]
pub struct ParserOptions {
    pub strict_mode: bool,
    pub allow_await: bool,
}

// Builder pattern for complex construction
impl EngineBuilder {
    pub fn new() -> Self { ... }
    pub fn with_strict_mode(mut self, strict: bool) -> Self { ... }
    pub fn build(self) -> Engine { ... }
}
```

### Lifetimes
```rust
// Elide lifetimes when possible
fn first(s: &str) -> &str { ... }  // Not fn first<'a>(s: &'a str) -> &'a str

// Name lifetimes descriptively for complex cases
struct Parser<'src> {
    source: &'src str,
    tokens: Vec<Token<'src>>,
}
```

### Documentation
```rust
//! Module-level documentation at the top of the file
//! Describes the purpose and contents of the module.

/// Type/function documentation using triple slash.
///
/// # Examples
///
/// ```rust
/// let lexer = Lexer::new("1 + 2");
/// let tokens = lexer.tokenize()?;
/// ```
///
/// # Errors
///
/// Returns `LexError::InvalidCharacter` if an unrecognized character is encountered.
///
/// # Panics
///
/// Panics if the source string is not valid UTF-8.
pub fn tokenize(&mut self) -> Result<Vec<Token>, LexError> { ... }
```

### Visibility
```rust
// Be explicit about visibility
pub struct Engine { ... }           // Public API
pub(crate) fn helper() { ... }      // Crate-internal
pub(super) fn parent_only() { ... } // Parent module only
fn private() { ... }                // Module-private (default)
```

### Performance Patterns
```rust
// Use &str instead of String when not taking ownership
fn parse(source: &str) -> Result<Ast, Error> { ... }

// Use Cow<str> when you might need to own
fn normalize(s: &str) -> Cow<'_, str> { ... }

// Prefer iterators over collecting into Vec
tokens.iter().filter(|t| t.is_keyword()).count()

// Use entry API for maps
map.entry(key).or_insert_with(Vec::new).push(value);
```

### Testing
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_descriptive_name() {
        // Arrange
        let input = "1 + 2";

        // Act
        let result = parse(input);

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    #[should_panic(expected = "index out of bounds")]
    fn test_panic_case() { ... }
}
```

## Project-Specific Conventions

### AST Nodes
- Use `Box<T>` for recursive types
- Implement `Clone` for all AST types
- Include source spans for error reporting

### Value Types
- Keep `Value` enum variants simple
- Use internal mutability sparingly (prefer `&mut self`)

### Memory Management
- Use arena allocation (`bumpalo`) for AST nodes during parsing
- Avoid unnecessary cloning - use references where possible


## Cursor rule: `.cursor/rules/test-coverage.mdc`

# Test Coverage Requirements

All code must achieve **90% test coverage** for both unit tests and documentation tests.

## Coverage Targets

| Metric | Minimum |
|--------|---------|
| Line coverage | 90% |
| Branch coverage | 85% |
| Function coverage | 90% |
| Doc-test coverage | 90% of public APIs |

## Running Coverage

Use `cargo-llvm-cov` for accurate coverage measurement:

```bash
# Install (once)
cargo install cargo-llvm-cov

# Run with coverage report
cargo llvm-cov --all-features --workspace

# Generate HTML report
cargo llvm-cov --all-features --workspace --html

# Check coverage threshold (CI)
cargo llvm-cov --all-features --workspace --fail-under-lines 90
```

## Unit Testing Requirements

### Every module must have tests

```rust
// At the bottom of each module
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_name() {
        // Arrange
        let input = setup();

        // Act
        let result = function_under_test(input);

        // Assert
        assert_eq!(result, expected);
    }
}
```

### Test naming conventions

```rust
#[test]
fn test_<function>_<scenario>_<expected_behavior>() { }

// Examples:
fn test_parse_valid_number_returns_value() { }
fn test_parse_empty_string_returns_error() { }
fn test_gc_collect_removes_unreachable_objects() { }
```

### Edge cases to always test

- Empty inputs
- Null/None values
- Boundary conditions (0, MAX, MIN)
- Error conditions
- Concurrent access (for async/parallel code)

## Documentation Testing Requirements

### All public items must have doc tests

```rust
/// Parses a JavaScript expression from source code.
///
/// # Arguments
///
/// * `source` - The JavaScript source code to parse
///
/// # Returns
///
/// The parsed AST expression, or an error if parsing fails.
///
/// # Examples
///
/// ```
/// use spacey_spidermonkey::Parser;
///
/// let mut parser = Parser::new("1 + 2");
/// let expr = parser.parse_expression().unwrap();
/// ```
///
/// # Errors
///
/// Returns `ParseError` if the source contains invalid syntax:
///
/// ```
/// use spacey_spidermonkey::Parser;
///
/// let mut parser = Parser::new("1 +");
/// assert!(parser.parse_expression().is_err());
/// ```
pub fn parse_expression(&mut self) -> Result<Expression, ParseError> { }
```

### Doc test patterns

```rust
/// # Examples
///
/// Basic usage:
/// ```
/// let result = my_function(42);
/// assert_eq!(result, 84);
/// ```
///
/// Error handling:
/// ```
/// let result = my_function(-1);
/// assert!(result.is_err());
/// ```
///
/// With async (requires tokio):
/// ```
/// # tokio_test::block_on(async {
/// let result = async_function().await;
/// assert!(result.is_ok());
/// # });
/// ```
```

### Hiding setup code in doc tests

```rust
/// ```
/// # use spacey_spidermonkey::Engine;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let engine = Engine::new();
/// let result = engine.eval("1 + 1")?;
/// assert_eq!(result.to_string(), "2");
/// # Ok(())
/// # }
/// ```
```

## Coverage Exceptions

Some code legitimately cannot be tested:

```rust
// Mark unreachable code
#[cfg(not(tarpaulin_include))]
fn unreachable_in_tests() { }

// Or use coverage attribute
#[coverage(off)]
fn platform_specific_code() { }
```

Valid exceptions:
- Platform-specific code not available in CI
- Panic handlers (tested via `#[should_panic]`)
- FFI bindings (tested via integration tests)
- Debug-only code

## CI Integration

Add to `.github/workflows/ci.yml`:

```yaml
coverage:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
      with:
        components: llvm-tools-preview
    - uses: taiki-e/install-action@cargo-llvm-cov
    - name: Generate coverage
      run: cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info
    - name: Check coverage threshold
      run: cargo llvm-cov --all-features --workspace --fail-under-lines 90
    - name: Upload coverage
      uses: codecov/codecov-action@v3
      with:
        files: lcov.info
```

## Pre-commit Hook

Add coverage check to `.husky/pre-push`:

```bash
#!/bin/bash
# Check test coverage before push
if command -v cargo-llvm-cov &> /dev/null; then
    cargo llvm-cov --all-features --workspace --fail-under-lines 90 || {
        echo "Coverage below 90%. Please add more tests."
        exit 1
    }
fi
```

## Improving Coverage

When coverage is below 90%:

1. Run `cargo llvm-cov --html` and open `target/llvm-cov/html/index.html`
2. Identify uncovered lines (highlighted in red)
3. Add tests for:
   - Uncovered functions
   - Uncovered branches (if/else, match arms)
   - Error paths
   - Edge cases

### Common coverage gaps

| Gap | Solution |
|-----|----------|
| Error handling | Test error conditions explicitly |
| Match arms | Test each variant |
| Early returns | Test conditions that trigger them |
| Default trait impls | Test `Default::default()` |
| Debug/Display | Test formatting output |

