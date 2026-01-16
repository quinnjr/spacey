# ECMAScript 5 (ES5) Additions Checklist

This document outlines features **added in ES5** (ECMA-262 5th Edition, December 2009) that are not present in ES3. For ES3 baseline features, see [TODO-ES3.md](./TODO-ES3.md).

**Prerequisites**: Complete ES3 implementation first.

**Reference**: [ECMA-262 5.1 Edition](https://262.ecma-international.org/5.1/)

---

## Table of Contents

1. [Strict Mode](#1-strict-mode)
2. [JSON Support](#2-json-support)
3. [Object Enhancements](#3-object-enhancements)
4. [Array Enhancements](#4-array-enhancements)
5. [Function Enhancements](#5-function-enhancements)
6. [String Enhancements](#6-string-enhancements)
7. [Date Enhancements](#7-date-enhancements)
8. [Syntax Additions](#8-syntax-additions)
9. [Other Changes](#9-other-changes)

---

## 1. Strict Mode

### 1.1 Enabling Strict Mode
- [ ] `"use strict";` directive prologue in scripts
- [ ] `"use strict";` directive prologue in function body
- [ ] Strict mode propagation to nested functions

### 1.2 Syntax Errors in Strict Mode
- [ ] Octal numeric literals forbidden (`0123`)
- [ ] Octal escape sequences forbidden (`"\123"`)
- [ ] `with` statement forbidden
- [ ] `delete` on unqualified identifier forbidden
- [ ] Duplicate parameter names forbidden
- [ ] Duplicate property names in object literals (syntax error)
- [ ] Assignment to `eval` or `arguments` forbidden
- [ ] `eval` and `arguments` as parameter names forbidden
- [ ] `eval` and `arguments` as variable names forbidden
- [ ] `eval` and `arguments` as function names forbidden
- [ ] `eval` and `arguments` as catch parameter forbidden

### 1.3 Strict Mode Reserved Words
Cannot be used as identifiers in strict mode:
- [ ] `implements`
- [ ] `interface`
- [ ] `let`
- [ ] `package`
- [ ] `private`
- [ ] `protected`
- [ ] `public`
- [ ] `static`
- [ ] `yield`

### 1.4 Runtime Behavior Changes in Strict Mode
- [ ] `this` is `undefined` (not global) in unbound function calls
- [ ] Assignment to undeclared variable throws `ReferenceError`
- [ ] Assignment to non-writable property throws `TypeError`
- [ ] Assignment to getter-only property throws `TypeError`
- [ ] Adding property to non-extensible object throws `TypeError`
- [ ] `delete` on non-configurable property throws `TypeError`
- [ ] `eval` does not introduce variables into enclosing scope
- [ ] `eval` has its own variable environment
- [ ] `arguments` object is not linked to parameters
- [ ] `arguments.callee` throws `TypeError`
- [ ] `arguments.caller` throws `TypeError`
- [ ] `func.caller` access throws `TypeError`
- [ ] `func.arguments` access throws `TypeError`

---

## 2. JSON Support

### 2.1 JSON Object
- [x] `JSON` global object

### 2.2 JSON.parse
- [x] `JSON.parse(text)`
- [ ] `JSON.parse(text, reviver)`
- [ ] Reviver function receives (key, value), returns transformed value
- [ ] Reviver called bottom-up (leaves first)
- [x] Throw `SyntaxError` on invalid JSON

### 2.3 JSON.stringify
- [x] `JSON.stringify(value)`
- [ ] `JSON.stringify(value, replacer)`
- [ ] `JSON.stringify(value, replacer, space)`
- [ ] Replacer as array (whitelist of keys)
- [ ] Replacer as function (key, value) → transformed value
- [ ] Space parameter for indentation (number or string)
- [ ] Handle `toJSON()` method on objects
- [x] Return `undefined` for functions, symbols, undefined
- [x] Throw `TypeError` on circular references
- [x] `NaN` and `Infinity` become `null`

### 2.4 JSON Grammar Support
- [x] Objects `{ "key": value }`
- [x] Arrays `[ value, ... ]`
- [x] Strings (with limited escapes)
- [x] Numbers (no hex, no leading zeros except `0.x`, no `+` prefix)
- [x] `true`, `false`, `null`
- [x] Unicode `\uXXXX` escapes in strings
- [x] No trailing commas
- [x] No single quotes
- [x] No unquoted keys

---

## 3. Object Enhancements

### 3.1 Property Descriptors
- [ ] Data descriptor: `{ value, writable, enumerable, configurable }`
- [ ] Accessor descriptor: `{ get, set, enumerable, configurable }`
- [ ] `[[DefineOwnProperty]]` internal method
- [ ] Property attribute validation rules

### 3.2 Object Constructor Methods (ES5 Additions)

#### Introspection
- [ ] `Object.getPrototypeOf(O)`
- [ ] `Object.getOwnPropertyDescriptor(O, P)`
- [ ] `Object.getOwnPropertyNames(O)`
- [ ] `Object.keys(O)` (enumerable own properties only)

#### Creation
- [ ] `Object.create(O)`
- [ ] `Object.create(O, Properties)`

#### Property Definition
- [ ] `Object.defineProperty(O, P, Attributes)`
- [ ] `Object.defineProperties(O, Properties)`

#### Object Integrity
- [ ] `Object.preventExtensions(O)` - prevent adding properties
- [ ] `Object.seal(O)` - preventExtensions + all properties non-configurable
- [ ] `Object.freeze(O)` - seal + all data properties non-writable
- [ ] `Object.isExtensible(O)`
- [ ] `Object.isSealed(O)`
- [ ] `Object.isFrozen(O)`

### 3.3 Internal Property Changes
- [ ] `[[Extensible]]` internal property
- [ ] Updated `[[DefineOwnProperty]]` semantics
- [ ] `[[Get]]` checks accessor descriptors
- [ ] `[[Put]]` checks writable attribute

---

## 4. Array Enhancements

### 4.1 Array Constructor Methods
- [ ] `Array.isArray(arg)`

### 4.2 Array Prototype Methods (ES5 Additions)

#### Index Search Methods
- [ ] `Array.prototype.indexOf(searchElement [, fromIndex])`
- [ ] `Array.prototype.lastIndexOf(searchElement [, fromIndex])`

#### Iteration Methods
- [ ] `Array.prototype.every(callbackfn [, thisArg])`
  - Returns `true` if callback returns truthy for all elements
- [ ] `Array.prototype.some(callbackfn [, thisArg])`
  - Returns `true` if callback returns truthy for any element
- [ ] `Array.prototype.forEach(callbackfn [, thisArg])`
  - Calls callback for each element, returns `undefined`
- [ ] `Array.prototype.map(callbackfn [, thisArg])`
  - Returns new array with callback results
- [ ] `Array.prototype.filter(callbackfn [, thisArg])`
  - Returns new array with elements where callback returned truthy

#### Reduction Methods
- [ ] `Array.prototype.reduce(callbackfn [, initialValue])`
  - Left-to-right reduction
  - Callback: `(accumulator, currentValue, index, array)`
- [ ] `Array.prototype.reduceRight(callbackfn [, initialValue])`
  - Right-to-left reduction

### 4.3 Callback Behavior
- [ ] Callback receives `(element, index, array)`
- [ ] `thisArg` parameter for `this` binding
- [ ] Holes in sparse arrays are skipped
- [ ] Generic (works on array-like objects)

---

## 5. Function Enhancements

### 5.1 Function.prototype.bind
- [ ] `Function.prototype.bind(thisArg [, arg1 [, arg2, ...]])`
- [ ] Returns bound function with fixed `this`
- [ ] Partial application (prepended arguments)
- [ ] Bound function's `length` = original length - bound args
- [ ] Bound function has no `prototype` property
- [ ] `new` on bound function uses original `[[Construct]]`
- [ ] `[[BoundThis]]`, `[[BoundArgs]]`, `[[TargetFunction]]` internals

---

## 6. String Enhancements

### 6.1 String.prototype.trim
- [ ] `String.prototype.trim()`
- [ ] Removes leading and trailing whitespace
- [ ] Whitespace includes: space, tab, nbsp, BOM, Unicode Zs
- [ ] Line terminators: LF, CR, LS, PS

### 6.2 Property Access by Index
- [ ] `str[0]` syntax for character access (was implementation-defined in ES3)
- [ ] Indexed properties are non-writable, non-configurable

---

## 7. Date Enhancements

### 7.1 Date Constructor Methods
- [ ] `Date.now()`
  - Returns current time in milliseconds since epoch

### 7.2 Date Prototype Methods
- [ ] `Date.prototype.toISOString()`
  - Returns ISO 8601 format: `YYYY-MM-DDTHH:mm:ss.sssZ`
  - Throws `RangeError` for invalid dates
- [ ] `Date.prototype.toJSON(key)`
  - Used by `JSON.stringify`
  - Calls `toISOString()` or returns `null` for invalid dates

### 7.3 Date.parse Improvements
- [ ] Parse ISO 8601 format strings
- [ ] `YYYY-MM-DD`
- [ ] `YYYY-MM-DDTHH:mm:ss.sssZ`
- [ ] Timezone offset `±HH:mm`

---

## 8. Syntax Additions

### 8.1 Getter/Setter in Object Literals
- [ ] `{ get propertyName() { ... } }`
- [ ] `{ set propertyName(value) { ... } }`
- [ ] Getter without setter = read-only
- [ ] Setter without getter = write-only

### 8.2 Trailing Commas
- [ ] Trailing comma in array literals (was ambiguous in ES3)
- [ ] `[1, 2, 3,]` has length 3, not 4

### 8.3 Reserved Word Property Names
- [ ] Reserved words allowed as property names: `obj.class`, `obj.if`
- [ ] Reserved words allowed as property keys in object literals

---

## 9. Other Changes

### 9.1 `undefined` Behavior
- [ ] `undefined` is non-writable in global scope (was writable in ES3)

### 9.2 parseInt Behavior
- [ ] `parseInt` no longer treats leading `0` as octal by default
- [ ] Must explicitly pass radix `8` for octal

### 9.3 Array.prototype Methods Generic
- [ ] ES5 array methods work on array-like objects
- [ ] Use `ToObject` and `ToUint32(length)`

### 9.4 Error Stack Traces (Non-Standard but Common)
- [ ] `Error.prototype.stack` (implementation-defined, not in spec)

---

## Summary Statistics

| Category | Items | Priority |
|----------|-------|----------|
| Strict Mode | ~30 | High |
| JSON Support | ~20 | High |
| Object Enhancements | ~25 | High |
| Array Enhancements | ~15 | High |
| Function Enhancements | ~7 | High |
| String Enhancements | ~4 | Medium |
| Date Enhancements | ~6 | Medium |
| Syntax Additions | ~5 | Medium |
| Other Changes | ~5 | Low |
| **Total** | **~117** | |

---

## Milestone Targets

*After ES3 compliance is achieved:*

| Milestone | Description | Target |
|-----------|-------------|--------|
| ES5-M1 | Property descriptors & Object methods | Week 2 |
| ES5-M2 | Array iteration methods | Week 3 |
| ES5-M3 | JSON support | Week 4 |
| ES5-M4 | Strict mode (syntax) | Week 5 |
| ES5-M5 | Strict mode (runtime) | Week 6 |
| ES5-M6 | Remaining features | Week 7 |
| ES5-M7 | Full ES5 compliance | Week 8 |

---

## Test262 ES5 Coverage

After implementation, validate against Test262:

```bash
# Run ES5-specific tests
cargo run -- --test262 tests/test262/test/language/
cargo run -- --test262 tests/test262/test/built-ins/
```

Target: 100% pass rate on ES5-applicable Test262 tests.

---

## References

- [ECMA-262 5.1 Edition (HTML)](https://262.ecma-international.org/5.1/)
- [Annotated ES5](https://es5.github.io/)
- [Test262 ES5 Tests](https://github.com/tc39/test262)
- [MDN JavaScript Reference](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference)
- [ES5 Compatibility Table](https://kangax.github.io/compat-table/es5/)
