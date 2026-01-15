# ECMAScript 2022 (ES13) Compatibility Checklist

This document outlines features added in ES2022 (ECMA-262 13th Edition, June 2022). ES2022 introduced class fields, private methods, top-level await, and several utility methods.

**Prerequisites**: Complete ES2021 implementation first.

**Reference**: [ECMA-262 13th Edition](https://262.ecma-international.org/13.0/)

---

## Table of Contents

1. [Class Fields](#1-class-fields)
2. [Private Methods and Accessors](#2-private-methods-and-accessors)
3. [Static Initialization Blocks](#3-static-initialization-blocks)
4. [Ergonomic Brand Checks for Private Fields](#4-ergonomic-brand-checks-for-private-fields)
5. [Top-Level Await](#5-top-level-await)
6. [Array and String .at() Method](#6-array-and-string-at-method)
7. [Object.hasOwn](#7-objecthasown)
8. [Error Cause](#8-error-cause)
9. [RegExp Match Indices](#9-regexp-match-indices)

---

## 1. Class Fields

### 1.1 Public Instance Fields
- [ ] `class { field = value }` field declaration syntax
- [ ] `class { field }` without initializer (undefined)
- [ ] Fields defined on instance, not prototype
- [ ] Field initializers run per instance during construction
- [ ] Can reference `this` in initializer
- [ ] Computed field names `class { [expr] = value }`

### 1.2 Public Static Fields
- [ ] `class { static field = value }` syntax
- [ ] `class { static field }` without initializer
- [ ] Static fields defined on constructor
- [ ] Initialized when class is evaluated
- [ ] Can reference static methods in initializer

### 1.3 Private Instance Fields
- [ ] `class { #field = value }` private field syntax
- [ ] `#field` prefix indicates private
- [ ] Only accessible inside class body
- [ ] `this.#field` access
- [ ] `obj.#field` from within class methods
- [ ] `TypeError` when accessing on wrong object
- [ ] `SyntaxError` for `#field` outside class

### 1.4 Private Static Fields
- [ ] `class { static #field = value }` syntax
- [ ] Accessible only within class body
- [ ] `ClassName.#field` or `this.#field` in static methods

### 1.5 Field Initialization Order
- [ ] Base class fields initialized first (after `super()`)
- [ ] Derived class fields initialized after `super()` returns
- [ ] Static fields initialized in declaration order
- [ ] Instance fields initialized in declaration order

---

## 2. Private Methods and Accessors

### 2.1 Private Instance Methods
- [ ] `class { #method() { } }` private method syntax
- [ ] Only callable from within class body
- [ ] `this.#method()` invocation
- [ ] `obj.#method()` from within class methods
- [ ] `TypeError` when called on wrong object
- [ ] Cannot be overridden in subclasses

### 2.2 Private Static Methods
- [ ] `class { static #method() { } }` syntax
- [ ] Accessible only within class body
- [ ] `ClassName.#method()` or `this.#method()` in static context

### 2.3 Private Instance Getters/Setters
- [ ] `class { get #prop() { } }` private getter
- [ ] `class { set #prop(v) { } }` private setter
- [ ] `this.#prop` access invokes getter
- [ ] `this.#prop = value` invokes setter
- [ ] Getter without setter is read-only
- [ ] Setter without getter is write-only

### 2.4 Private Static Getters/Setters
- [ ] `class { static get #prop() { } }` static private getter
- [ ] `class { static set #prop(v) { } }` static private setter

### 2.5 Private Generator Methods
- [ ] `class { *#method() { } }` private generator
- [ ] `class { static *#method() { } }` static private generator

### 2.6 Private Async Methods
- [ ] `class { async #method() { } }` private async method
- [ ] `class { static async #method() { } }` static private async method
- [ ] `class { async *#method() { } }` private async generator

---

## 3. Static Initialization Blocks

### 3.1 Basic Syntax
- [ ] `class { static { /* code */ } }` static block syntax
- [ ] Code runs when class is evaluated
- [ ] Can access private static fields/methods
- [ ] Multiple static blocks allowed (run in order)

### 3.2 Execution Context
- [ ] `this` refers to constructor function
- [ ] Can access static members
- [ ] Cannot use `await` (unless in async context wrapper)
- [ ] `return` statement not allowed
- [ ] `arguments` not available

### 3.3 Use Cases
- [ ] Complex static field initialization
- [ ] Initialize multiple related static fields
- [ ] Friend class patterns (share private access)

### 3.4 Interleaving with Static Fields
- [ ] Static blocks and fields execute in declaration order
- [ ] `static { }` can appear between field declarations

---

## 4. Ergonomic Brand Checks for Private Fields

### 4.1 in Operator for Private Fields
- [ ] `#field in obj` checks if object has private field
- [ ] Returns `true` if object has the private field
- [ ] Returns `false` otherwise (does not throw)
- [ ] Works with private fields, methods, accessors

### 4.2 Brand Check Semantics
- [ ] Does not access the field value
- [ ] Only checks for presence
- [ ] Useful for `instanceof`-like checks based on private state

### 4.3 Static Private Brand Checks
- [ ] `#staticField in obj` checks for static private field
- [ ] Returns `true` only for the class constructor itself

---

## 5. Top-Level Await

### 5.1 Basic Functionality
- [ ] `await` allowed at module top level
- [ ] No wrapping async function needed
- [ ] Module evaluation pauses until await resolves
- [ ] `await promise` at top level of module

### 5.2 Module Execution
- [ ] Importing module waits for top-level awaits
- [ ] Sibling modules may execute in parallel
- [ ] Dependency modules fully execute before dependents

### 5.3 Restrictions
- [ ] Only in modules (not classic scripts)
- [ ] `SyntaxError` in non-module context
- [ ] Circular dependencies with top-level await may deadlock

### 5.4 Error Handling
- [ ] Rejected await rejects module evaluation
- [ ] Importing modules catch the rejection
- [ ] Can use try/catch around top-level await

---

## 6. Array and String .at() Method

### 6.1 Array.prototype.at
- [ ] `array.at(index)` returns element at index
- [ ] Negative indices count from end: `array.at(-1)` is last element
- [ ] `array.at(-2)` is second-to-last
- [ ] Returns `undefined` for out-of-bounds
- [ ] Equivalent to `array[index]` for non-negative indices

### 6.2 String.prototype.at
- [ ] `str.at(index)` returns character at index
- [ ] Negative indices count from end
- [ ] `str.at(-1)` is last character
- [ ] Returns `undefined` for out-of-bounds
- [ ] Returns single UTF-16 code unit (like `charAt`)

### 6.3 TypedArray.prototype.at
- [ ] `typedArray.at(index)` same behavior as Array.prototype.at
- [ ] Works on all TypedArray types

### 6.4 Index Calculation
- [ ] `at(n)` where `n >= 0`: returns element at `n`
- [ ] `at(n)` where `n < 0`: returns element at `length + n`
- [ ] `at(-0)` returns first element (same as `at(0)`)

---

## 7. Object.hasOwn

### 7.1 Basic Functionality
- [ ] `Object.hasOwn(obj, prop)` checks for own property
- [ ] Returns `true` if object has own property with key
- [ ] Returns `false` otherwise
- [ ] Preferred over `obj.hasOwnProperty(prop)`

### 7.2 Advantages over hasOwnProperty
- [ ] Works when object has no prototype: `Object.create(null)`
- [ ] Works when `hasOwnProperty` is shadowed
- [ ] Does not invoke any user-defined code on object

### 7.3 Symbol Keys
- [ ] Works with symbol property keys
- [ ] `Object.hasOwn(obj, Symbol.for("key"))` works

---

## 8. Error Cause

### 8.1 Error Constructor Option
- [ ] `new Error(message, { cause })` with cause option
- [ ] `error.cause` property holds the cause
- [ ] Cause can be any value
- [ ] Options object is second parameter

### 8.2 Native Error Types
- [ ] `new TypeError(message, { cause })` works
- [ ] `new RangeError(message, { cause })` works
- [ ] `new SyntaxError(message, { cause })` works
- [ ] `new ReferenceError(message, { cause })` works
- [ ] `new URIError(message, { cause })` works
- [ ] `new EvalError(message, { cause })` works
- [ ] `new AggregateError(errors, message, { cause })` works

### 8.3 Use Cases
- [ ] Wrap and rethrow errors with context
- [ ] Chain errors for debugging
- [ ] Preserve original error when adding information

---

## 9. RegExp Match Indices

### 9.1 d Flag
- [ ] `/pattern/d` enables indices
- [ ] `regexp.hasIndices` property (getter)
- [ ] `hasIndices` returns `true` if `d` flag set

### 9.2 Indices Property
- [ ] `result.indices` array on match result
- [ ] Contains `[start, end]` pairs for each capture
- [ ] `result.indices[0]` is full match position
- [ ] `result.indices[1]` is first capture group position
- [ ] Unmatched groups have `undefined` in indices

### 9.3 Named Group Indices
- [ ] `result.indices.groups` object for named captures
- [ ] `result.indices.groups.name` is `[start, end]` for named group
- [ ] `null` if no named groups in pattern

### 9.4 Examples
- [ ] `/a(b)c/d.exec("abc")` - `indices` is `[[0, 3], [1, 2]]`
- [ ] `/(?<letter>a)/d.exec("a")` - `indices.groups.letter` is `[0, 1]`

---

## Summary Statistics

| Category | Items |
|----------|-------|
| Class Fields | ~20 |
| Private Methods and Accessors | ~20 |
| Static Initialization Blocks | ~10 |
| Ergonomic Brand Checks | ~6 |
| Top-Level Await | ~10 |
| Array and String .at() | ~12 |
| Object.hasOwn | ~6 |
| Error Cause | ~12 |
| RegExp Match Indices | ~10 |
| **Total** | **~106** |

---

## Full ES2022 Compliance Checklist Summary

With ES2022 complete, you now have comprehensive checklists for:

| Standard | Document | Items |
|----------|----------|-------|
| ES3 | TODO-ES3.md | ~330 |
| ES5 | TODO-ES5.md | ~117 |
| ES2015 | TODO-ES2015.md | ~561 |
| ES2016 | TODO-ES2016.md | ~28 |
| ES2017 | TODO-ES2017.md | ~94 |
| ES2018 | TODO-ES2018.md | ~90 |
| ES2019 | TODO-ES2019.md | ~70 |
| ES2020 | TODO-ES2020.md | ~130 |
| ES2021 | TODO-ES2021.md | ~86 |
| ES2022 | TODO-ES2022.md | ~106 |
| **Total** | | **~1,612** |

---

## References

- [ECMA-262 13th Edition (ES2022)](https://262.ecma-international.org/13.0/)
- [MDN Class fields](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Classes/Public_class_fields)
- [MDN Private class features](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Classes/Private_class_fields)
- [MDN Array.prototype.at](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/at)
- [MDN Object.hasOwn](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/hasOwn)
- [MDN Error cause](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Error/cause)
- [Kangax ES2016+ Compatibility Table](https://kangax.github.io/compat-table/es2016plus/)
