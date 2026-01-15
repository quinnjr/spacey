# ECMAScript 2020 (ES11) Compatibility Checklist

This document outlines features added in ES2020 (ECMA-262 11th Edition, June 2020). ES2020 introduced BigInt, optional chaining, nullish coalescing, and other significant features.

**Prerequisites**: Complete ES2019 implementation first.

**Reference**: [ECMA-262 11th Edition](https://262.ecma-international.org/11.0/)

---

## Table of Contents

1. [BigInt](#1-bigint)
2. [Optional Chaining](#2-optional-chaining)
3. [Nullish Coalescing](#3-nullish-coalescing)
4. [Promise.allSettled](#4-promiseallsettled)
5. [String.prototype.matchAll](#5-stringprototypematchall)
6. [globalThis](#6-globalthis)
7. [Dynamic Import](#7-dynamic-import)
8. [export * as namespace](#8-export--as-namespace)
9. [import.meta](#9-importmeta)
10. [for-in Order](#10-for-in-order)

---

## 1. BigInt

### 1.1 BigInt Primitive
- [ ] `BigInt(value)` constructor function (not `new`)
- [ ] `123n` literal syntax with `n` suffix
- [ ] `typeof bigint === "bigint"`
- [ ] Arbitrary precision integers
- [ ] Cannot use `new BigInt()` (`TypeError`)

### 1.2 BigInt Literals
- [ ] Decimal: `123n`
- [ ] Hexadecimal: `0xFFn`
- [ ] Octal: `0o77n`
- [ ] Binary: `0b1010n`
- [ ] Negative: `-123n`

### 1.3 BigInt Conversion
- [ ] `BigInt(number)` from integer Number
- [ ] `BigInt("123")` from string
- [ ] `BigInt(true)` returns `1n`, `BigInt(false)` returns `0n`
- [ ] `BigInt(1.5)` throws `RangeError` (non-integer)
- [ ] `BigInt(NaN)` throws `RangeError`
- [ ] `BigInt(Infinity)` throws `RangeError`

### 1.4 BigInt Arithmetic Operators
- [ ] `+` addition: `1n + 2n` returns `3n`
- [ ] `-` subtraction: `5n - 3n` returns `2n`
- [ ] `*` multiplication: `2n * 3n` returns `6n`
- [ ] `/` division (truncates): `5n / 2n` returns `2n`
- [ ] `%` remainder: `5n % 2n` returns `1n`
- [ ] `**` exponentiation: `2n ** 10n` returns `1024n`
- [ ] Unary `-` negation: `-5n`
- [ ] Unary `+` is not allowed (`TypeError`)

### 1.5 BigInt Bitwise Operators
- [ ] `&` bitwise AND
- [ ] `|` bitwise OR
- [ ] `^` bitwise XOR
- [ ] `~` bitwise NOT (assumes infinite sign extension)
- [ ] `<<` left shift
- [ ] `>>` signed right shift (same as unsigned for BigInt)
- [ ] Negative shift counts: `1n << -1n` throws `RangeError`

### 1.6 BigInt Comparison
- [ ] `===` strict equality: `1n === 1n` is `true`
- [ ] `===` type check: `1n === 1` is `false`
- [ ] `==` loose equality: `1n == 1` is `true`
- [ ] `<`, `>`, `<=`, `>=` work with BigInt
- [ ] Comparison with Number: `1n < 2` is `true`
- [ ] Cannot mix BigInt and Number in arithmetic (`TypeError`)

### 1.7 BigInt Methods
- [ ] `BigInt.asIntN(bits, bigint)` - wrap to signed N-bit integer
- [ ] `BigInt.asUintN(bits, bigint)` - wrap to unsigned N-bit integer
- [ ] `BigInt.prototype.toString([radix])` - convert to string
- [ ] `BigInt.prototype.valueOf()` - return primitive BigInt
- [ ] `BigInt.prototype.toLocaleString()` - locale-aware string

### 1.8 BigInt Type Coercion
- [ ] `Boolean(0n)` is `false`, `Boolean(1n)` is `true`
- [ ] `Number(bigint)` may lose precision
- [ ] `String(bigint)` works
- [ ] Cannot convert BigInt to Number implicitly

### 1.9 BigInt JSON
- [ ] `JSON.stringify(bigint)` throws `TypeError`
- [ ] Must define `toJSON` method for custom serialization

---

## 2. Optional Chaining

### 2.1 Property Access
- [ ] `obj?.prop` returns `undefined` if `obj` is `null`/`undefined`
- [ ] `obj?.prop` returns `obj.prop` otherwise
- [ ] Does not short-circuit assignment: `obj?.prop = x` is `SyntaxError`

### 2.2 Bracket Notation
- [ ] `obj?.[expr]` optional computed property access
- [ ] `arr?.[0]` optional array index access

### 2.3 Function Calls
- [ ] `func?.()` returns `undefined` if `func` is `null`/`undefined`
- [ ] `obj.method?.()` optional method call
- [ ] `func?.(arg1, arg2)` with arguments

### 2.4 Chaining
- [ ] `obj?.a?.b?.c` multiple optional accesses
- [ ] Short-circuits on first `null`/`undefined`
- [ ] `obj?.a.b` - if `obj` is `null`, returns `undefined` (doesn't access `.b`)

### 2.5 Short-Circuiting
- [ ] Right side not evaluated if left is nullish
- [ ] `null?.x.y.z` returns `undefined` (no error)
- [ ] Side effects skipped: `obj?.[console.log("hi")]` doesn't log if `obj` is nullish

### 2.6 Interaction with Other Operators
- [ ] `(obj?.prop)` with grouping
- [ ] `delete obj?.prop` returns `true` if `obj` is nullish
- [ ] `typeof obj?.prop` returns `"undefined"` if nullish

### 2.7 Not Allowed
- [ ] `new obj?.()` is `SyntaxError`
- [ ] `obj?.() = 1` is `SyntaxError`
- [ ] Template literal: ``obj?.`template` `` is `SyntaxError`
- [ ] `super?.()` is `SyntaxError`
- [ ] `super?.prop` is `SyntaxError`

---

## 3. Nullish Coalescing

### 3.1 Basic Syntax
- [ ] `a ?? b` returns `b` if `a` is `null` or `undefined`
- [ ] `a ?? b` returns `a` otherwise
- [ ] Distinguishes `null`/`undefined` from falsy values
- [ ] `0 ?? 1` returns `0` (unlike `||`)
- [ ] `"" ?? "default"` returns `""` (unlike `||`)
- [ ] `false ?? true` returns `false` (unlike `||`)

### 3.2 Short-Circuit Evaluation
- [ ] `b` not evaluated if `a` is not nullish
- [ ] `x ?? console.log("hi")` doesn't log if `x` is defined

### 3.3 Assignment Operator
- [ ] `a ??= b` assigns `b` to `a` only if `a` is nullish
- [ ] `obj.prop ??= value` nullish assignment to property

### 3.4 Precedence and Mixing
- [ ] Cannot mix with `&&` or `||` without parentheses
- [ ] `a ?? b || c` is `SyntaxError`
- [ ] `a || b ?? c` is `SyntaxError`
- [ ] `(a ?? b) || c` is valid
- [ ] `a ?? (b || c)` is valid
- [ ] `a ?? b ?? c` chains left-to-right

---

## 4. Promise.allSettled

### 4.1 Basic Functionality
- [ ] `Promise.allSettled(iterable)` returns promise
- [ ] Waits for all promises to settle (fulfill or reject)
- [ ] Never short-circuits (unlike `Promise.all`)
- [ ] Result is array of outcome objects

### 4.2 Outcome Objects
- [ ] Fulfilled: `{ status: "fulfilled", value: ... }`
- [ ] Rejected: `{ status: "rejected", reason: ... }`
- [ ] Order matches input order

### 4.3 Non-Promise Values
- [ ] Non-promise values treated as fulfilled
- [ ] `Promise.allSettled([1, 2])` returns `[{ status: "fulfilled", value: 1 }, ...]`

---

## 5. String.prototype.matchAll

### 5.1 Basic Functionality
- [ ] `str.matchAll(regexp)` returns iterator
- [ ] `regexp` must have `g` flag (`TypeError` otherwise)
- [ ] Each iteration yields match array (like `exec`)
- [ ] Includes `index` and `input` properties
- [ ] Includes `groups` for named capture groups

### 5.2 Iterator Behavior
- [ ] Returns `RegExpStringIterator`
- [ ] Can convert to array: `[...str.matchAll(re)]`
- [ ] Can use in `for-of`
- [ ] Does not modify `regexp.lastIndex`
- [ ] Creates internal copy of regexp

### 5.3 Comparison with exec Loop
- [ ] `matchAll` is cleaner than `while (exec())` loop
- [ ] `matchAll` doesn't require manual `lastIndex` reset

---

## 6. globalThis

### 6.1 Global Object Access
- [ ] `globalThis` provides universal access to global object
- [ ] Same as `window` in browsers
- [ ] Same as `global` in Node.js
- [ ] Same as `self` in workers
- [ ] Works in all environments

### 6.2 Properties
- [ ] `globalThis` is writable, configurable
- [ ] Initial value is global object
- [ ] `globalThis === globalThis` is `true`

---

## 7. Dynamic Import

### 7.1 import() Syntax
- [ ] `import(specifier)` returns Promise
- [ ] Resolves to module namespace object
- [ ] Can be used anywhere (not just top-level)
- [ ] Works in non-module scripts

### 7.2 Module Namespace Object
- [ ] Contains all exports
- [ ] Default export at `.default`
- [ ] Named exports as properties
- [ ] `import("./module.js").then(m => m.default)`

### 7.3 Use Cases
- [ ] Conditional imports: `if (condition) import("module")`
- [ ] Computed specifiers: `import(\`./locale/${lang}.js\`)`
- [ ] Lazy loading

### 7.4 Error Handling
- [ ] Rejects if module not found
- [ ] Rejects if module has syntax error
- [ ] Rejects if module throws during evaluation

---

## 8. export * as namespace

### 8.1 Syntax
- [ ] `export * as name from "module"` re-exports namespace
- [ ] Creates named export containing all exports from module
- [ ] Includes default export of source module

### 8.2 Example
- [ ] `export * as utils from "./utils.js"` - access as `import { utils } from "..."`
- [ ] Combines with other exports: `export * as ns from "m"; export { foo } from "m";`

---

## 9. import.meta

### 9.1 Meta Properties
- [ ] `import.meta` object available in modules
- [ ] Host-defined properties
- [ ] `import.meta.url` - URL of current module (common property)

### 9.2 Restrictions
- [ ] Only available in module code
- [ ] `SyntaxError` in non-module scripts
- [ ] Cannot destructure: `const { url } = import.meta` works at runtime
- [ ] `import.meta` itself cannot be assigned to

---

## 10. for-in Order

### 10.1 Property Enumeration Order
- [ ] `for-in` order is now specified (was implementation-defined)
- [ ] Own properties before inherited
- [ ] Integer indices in ascending order
- [ ] Other string keys in creation order
- [ ] Note: still some edge cases that are implementation-defined

---

## Summary Statistics

| Category | Items |
|----------|-------|
| BigInt | ~48 |
| Optional Chaining | ~22 |
| Nullish Coalescing | ~14 |
| Promise.allSettled | ~8 |
| String.prototype.matchAll | ~10 |
| globalThis | ~5 |
| Dynamic Import | ~10 |
| export * as namespace | ~4 |
| import.meta | ~5 |
| for-in Order | ~4 |
| **Total** | **~130** |

---

## References

- [ECMA-262 11th Edition (ES2020)](https://262.ecma-international.org/11.0/)
- [MDN BigInt](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/BigInt)
- [MDN Optional chaining](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Optional_chaining)
- [MDN Nullish coalescing](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Nullish_coalescing)
- [Kangax ES2016+ Compatibility Table](https://kangax.github.io/compat-table/es2016plus/)
