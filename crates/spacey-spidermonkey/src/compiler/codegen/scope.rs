//! Scope management for variable resolution during compilation.

use crate::Error;

/// A local variable in a scope.
#[derive(Debug, Clone)]
pub struct Local {
    /// The variable name
    pub name: String,
    /// The scope depth where this was declared
    pub depth: usize,
    /// Whether the variable is mutable (var/let vs const)
    pub mutable: bool,
    /// Whether the variable has been initialized
    pub initialized: bool,
}

/// A scope for variable resolution.
#[derive(Debug, Default)]
pub struct Scope {
    /// Local variables in this scope
    pub locals: Vec<Local>,
    /// Current scope depth (0 = global)
    pub depth: usize,
}

impl Scope {
    /// Creates a new scope.
    pub fn new() -> Self {
        Self {
            locals: Vec::new(),
            depth: 0,
        }
    }

    /// Begin a new scope.
    pub fn begin_scope(&mut self) {
        self.depth += 1;
    }

    /// End the current scope and return the number of locals to pop.
    pub fn end_scope(&mut self) -> usize {
        let mut count = 0;
        while !self.locals.is_empty() && self.locals.last().unwrap().depth == self.depth {
            self.locals.pop();
            count += 1;
        }
        self.depth -= 1;
        count
    }

    /// Declare a local variable.
    pub fn declare(&mut self, name: String, mutable: bool) -> Result<usize, Error> {
        // Check for duplicate in same scope
        for local in self.locals.iter().rev() {
            if local.depth < self.depth {
                break;
            }
            if local.name == name {
                return Err(Error::SyntaxError(format!(
                    "Variable '{}' already declared in this scope",
                    name
                )));
            }
        }

        let index = self.locals.len();
        self.locals.push(Local {
            name,
            depth: self.depth,
            mutable,
            initialized: false,
        });
        Ok(index)
    }

    /// Mark a variable as initialized.
    pub fn mark_initialized(&mut self, index: usize) {
        if index < self.locals.len() {
            self.locals[index].initialized = true;
        }
    }

    /// Resolve a local variable by name, returning its index.
    pub fn resolve(&self, name: &str) -> Option<usize> {
        for (i, local) in self.locals.iter().enumerate().rev() {
            if local.name == name {
                return Some(i);
            }
        }
        None
    }

    /// Check if a variable is a local (vs global).
    pub fn is_local(&self, name: &str) -> bool {
        self.resolve(name).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scope_new() {
        let scope = Scope::new();
        assert_eq!(scope.depth, 0);
        assert!(scope.locals.is_empty());
    }

    #[test]
    fn test_scope_begin_end() {
        let mut scope = Scope::new();
        scope.begin_scope();
        assert_eq!(scope.depth, 1);
        scope.end_scope();
        assert_eq!(scope.depth, 0);
    }

    #[test]
    fn test_scope_declare() {
        let mut scope = Scope::new();
        scope.begin_scope();
        let idx = scope.declare("x".to_string(), true).unwrap();
        assert_eq!(idx, 0);
        assert_eq!(scope.locals.len(), 1);
    }

    #[test]
    fn test_scope_resolve() {
        let mut scope = Scope::new();
        scope.begin_scope();
        scope.declare("x".to_string(), true).unwrap();
        assert_eq!(scope.resolve("x"), Some(0));
        assert_eq!(scope.resolve("y"), None);
    }

    #[test]
    fn test_scope_duplicate_error() {
        let mut scope = Scope::new();
        scope.begin_scope();
        scope.declare("x".to_string(), true).unwrap();
        let result = scope.declare("x".to_string(), true);
        assert!(result.is_err());
    }
}
