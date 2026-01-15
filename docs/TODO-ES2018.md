# ECMAScript 2018 (ES9) Compatibility Checklist

This document outlines features added in ES2018 (ECMA-262 9th Edition, June 2018). ES2018 introduced asynchronous iteration, rest/spread for objects, and significant RegExp improvements.

**Prerequisites**: Complete ES2017 implementation first.

**Reference**: [ECMA-262 9th Edition](https://262.ecma-international.org/9.0/)

---

## Table of Contents

1. [Asynchronous Iteration](#1-asynchronous-iteration)
2. [Object Rest/Spread Properties](#2-object-restspread-properties)
3. [Promise.prototype.finally](#3-promiseprototypefinally)
4. [RegExp Improvements](#4-regexp-improvements)
5. [Template Literal Revision](#5-template-literal-revision)

---

## 1. Asynchronous Iteration

### 1.1 Async Iterator Protocol
- [ ] `Symbol.asyncIterator` method
- [ ] Async iterator has `next()` returning `Promise<{ value, done }>`
- [ ] Optional `return()` method returning promise
- [ ] Optional `throw()` method returning promise

### 1.2 Async Iterable Protocol
- [ ] Object with `[Symbol.asyncIterator]()` method
- [ ] Method returns an async iterator

### 1.3 For-Await-Of Loop
- [ ] `for await (const x of asyncIterable) { }` syntax
- [ ] Only valid inside async function or async generator
- [ ] `SyntaxError` outside async context
- [ ] Awaits each `next()` result
- [ ] Works with sync iterables too (wraps values in promises)
- [ ] `break` calls `return()` if present
- [ ] Destructuring: `for await (const [a, b] of asyncIterable) { }`

### 1.4 Async Generator Functions
- [ ] `async function* name() { }` syntax
- [ ] `async function*` returns async generator object
- [ ] Can use both `yield` and `await`
- [ ] `yield` yields a promise that resolves to the value
- [ ] `yield*` delegates to async or sync iterable
- [ ] `yield* asyncIterable` awaits each value

### 1.5 Async Generator Object
- [ ] `next(value)` returns `Promise<{ value, done }>`
- [ ] `return(value)` returns `Promise<{ value, done: true }>`
- [ ] `throw(error)` returns `Promise` (rejects or returns)
- [ ] `[Symbol.asyncIterator]()` returns `this`

### 1.6 Async Generator Methods
- [ ] `{ async *method() { } }` in object literals
- [ ] `class { async *method() { } }` in classes
- [ ] `static async *method()` for static async generators

---

## 2. Object Rest/Spread Properties

### 2.1 Object Spread in Literals
- [ ] `{ ...source }` spreads own enumerable properties
- [ ] `{ ...a, ...b }` merges objects (later wins)
- [ ] `{ x: 1, ...obj }` combines with regular properties
- [ ] `{ ...obj, x: 1 }` regular properties override spread
- [ ] Spread copies values, not descriptors (unlike `Object.assign`)
- [ ] Getters are invoked during spread
- [ ] Does not copy prototype
- [ ] Symbol-keyed properties are copied
- [ ] `{ ...null }` and `{ ...undefined }` are valid (empty spread)

### 2.2 Object Rest in Destructuring
- [ ] `const { a, ...rest } = obj` collects remaining own enumerable properties
- [ ] Rest must be last in pattern
- [ ] Rest variable gets a plain object
- [ ] Does not include already-destructured properties
- [ ] Does not include non-enumerable properties
- [ ] Does not include inherited properties
- [ ] Does not include symbol-keyed properties
- [ ] `let { a, ...rest } = obj` with let
- [ ] `var { a, ...rest } = obj` with var

### 2.3 Rest in Parameter Destructuring
- [ ] `function f({ a, ...rest }) { }` rest in parameters
- [ ] `({ a, ...rest } = obj)` rest in assignment destructuring

---

## 3. Promise.prototype.finally

### 3.1 Basic Functionality
- [ ] `promise.finally(onFinally)` syntax
- [ ] `onFinally` called regardless of fulfillment or rejection
- [ ] `onFinally` receives no arguments
- [ ] Returns new promise

### 3.2 Promise Chain Behavior
- [ ] Fulfilled promise passes through value: `p.finally(f).then(v => ...)` gets original value
- [ ] Rejected promise passes through reason: `p.finally(f).catch(e => ...)` gets original reason
- [ ] `onFinally` return value is ignored (unless it throws or returns rejected promise)
- [ ] `onFinally` throwing rejects returned promise
- [ ] `onFinally` returning rejected promise rejects returned promise
- [ ] If `onFinally` returns pending promise, waits for it

### 3.3 Non-Function Argument
- [ ] Non-function `onFinally` is ignored (pass-through)

---

## 4. RegExp Improvements

### 4.1 Named Capture Groups
- [ ] `(?<name>pattern)` syntax for named groups
- [ ] Access via `match.groups.name`
- [ ] Access via `result.groups.name` in `exec()`
- [ ] Named backreference: `\k<name>`
- [ ] Named groups can also be accessed by number
- [ ] `groups` is `null` if no named groups in pattern
- [ ] `groups` object has `null` prototype

### 4.2 Named Groups in String.prototype.replace
- [ ] `$<name>` substitution in replacement string
- [ ] `$<name>` for unmatched group inserts empty string
- [ ] Named groups in replacement function: `function(match, ...groups, offset, string, groups) { }`

### 4.3 RegExp Unicode Property Escapes
- [ ] `\p{Property}` matches characters with property (requires `u` flag)
- [ ] `\P{Property}` matches characters without property
- [ ] `\p{Script=Greek}` script properties
- [ ] `\p{Script_Extensions=Greek}` script extensions
- [ ] `\p{General_Category=Letter}` or `\p{Letter}` or `\p{L}`
- [ ] `\p{Lowercase}` binary properties
- [ ] `\p{Emoji}` emoji properties
- [ ] Invalid property names throw `SyntaxError`

### 4.4 RegExp Lookbehind Assertions
- [ ] `(?<=pattern)` positive lookbehind
- [ ] `(?<!pattern)` negative lookbehind
- [ ] Lookbehind can be variable length
- [ ] Lookbehind group captures work (right-to-left evaluation)
- [ ] Backreferences in lookbehind are evaluated right-to-left

### 4.5 RegExp `s` (dotAll) Flag
- [ ] `s` flag enables dotAll mode
- [ ] `.` matches any character including line terminators
- [ ] `.` matches `\n`, `\r`, `\u2028`, `\u2029`
- [ ] `regexp.dotAll` property (getter)

---

## 5. Template Literal Revision

### 5.1 Relaxed Escape Sequence Restrictions
- [ ] Invalid escapes allowed in tagged templates
- [ ] `strings` array contains `undefined` for invalid escapes
- [ ] `strings.raw` contains the raw source text
- [ ] Invalid escapes still forbidden in untagged templates
- [ ] Enables DSLs with custom escape handling (e.g., LaTeX)

### 5.2 Examples of Now-Valid Tagged Templates
- [ ] `` tag`\unicode` `` - invalid unicode escape
- [ ] `` tag`\u{` `` - incomplete unicode escape
- [ ] `` tag`\xgg` `` - invalid hex escape
- [ ] `` tag`\01` `` - octal escape (invalid in template)

---

## Summary Statistics

| Category | Items |
|----------|-------|
| Asynchronous Iteration | ~26 |
| Object Rest/Spread Properties | ~20 |
| Promise.prototype.finally | ~10 |
| RegExp Improvements | ~28 |
| Template Literal Revision | ~6 |
| **Total** | **~90** |

---

## References

- [ECMA-262 9th Edition (ES2018)](https://262.ecma-international.org/9.0/)
- [MDN for-await...of](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/for-await...of)
- [MDN Named capture groups](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Regular_expressions/Groups_and_backreferences)
- [MDN Promise.prototype.finally](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise/finally)
- [Kangax ES2016+ Compatibility Table](https://kangax.github.io/compat-table/es2016plus/)
