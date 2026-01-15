# ECMAScript 2021 (ES12) Compatibility Checklist

This document outlines features added in ES2021 (ECMA-262 12th Edition, June 2021). ES2021 introduced String.replaceAll, Promise.any, logical assignment operators, numeric separators, and WeakRefs.

**Prerequisites**: Complete ES2020 implementation first.

**Reference**: [ECMA-262 12th Edition](https://262.ecma-international.org/12.0/)

---

## Table of Contents

1. [String.prototype.replaceAll](#1-stringprototypereplaceall)
2. [Promise.any and AggregateError](#2-promiseany-and-aggregateerror)
3. [Logical Assignment Operators](#3-logical-assignment-operators)
4. [Numeric Separators](#4-numeric-separators)
5. [WeakRefs and FinalizationRegistry](#5-weakrefs-and-finalizationregistry)

---

## 1. String.prototype.replaceAll

### 1.1 Basic Functionality
- [ ] `str.replaceAll(searchValue, replaceValue)` replaces all occurrences
- [ ] Returns new string, does not modify original
- [ ] `searchValue` can be string or RegExp
- [ ] If RegExp, must have `g` flag (`TypeError` otherwise)

### 1.2 String Search
- [ ] `"aaa".replaceAll("a", "b")` returns `"bbb"`
- [ ] Empty string: `"abc".replaceAll("", "-")` returns `"-a-b-c-"`
- [ ] Case sensitive by default
- [ ] No RegExp special characters interpreted

### 1.3 Replacement String
- [ ] `$$` inserts literal `$`
- [ ] `$&` inserts matched substring
- [ ] `` $` `` inserts portion before match
- [ ] `$'` inserts portion after match
- [ ] `$n` inserts nth capture group (RegExp only)
- [ ] `$<name>` inserts named capture group (RegExp only)

### 1.4 Replacement Function
- [ ] `str.replaceAll(search, function)` - function called for each match
- [ ] Function receives `(match, p1, p2, ..., offset, string, groups)`
- [ ] Function return value is used as replacement
- [ ] Called once per match

### 1.5 Comparison with replace
- [ ] `replace` only replaces first match (for string pattern)
- [ ] `replaceAll` replaces all matches
- [ ] Both equivalent for RegExp with `g` flag

---

## 2. Promise.any and AggregateError

### 2.1 Promise.any
- [ ] `Promise.any(iterable)` returns promise
- [ ] Resolves when first promise fulfills
- [ ] Result is value of first fulfilled promise
- [ ] Rejects only if all promises reject
- [ ] Short-circuits on first fulfillment

### 2.2 AggregateError
- [ ] `new AggregateError(errors, message)` constructor
- [ ] `AggregateError(errors, message)` called as function
- [ ] `error.errors` array of individual errors
- [ ] `error.message` error message
- [ ] Extends `Error`

### 2.3 Promise.any Rejection
- [ ] Rejects with `AggregateError` when all promises reject
- [ ] `error.errors` contains all rejection reasons
- [ ] Order matches input iterable order
- [ ] Empty iterable rejects with `AggregateError` (empty errors array)

### 2.4 Comparison with Promise.race
- [ ] `Promise.race` settles with first settled (fulfill or reject)
- [ ] `Promise.any` settles with first fulfilled (ignores rejections)

---

## 3. Logical Assignment Operators

### 3.1 Logical OR Assignment
- [ ] `a ||= b` assigns `b` to `a` if `a` is falsy
- [ ] Equivalent to `a || (a = b)` (not `a = a || b`)
- [ ] Short-circuits: `b` not evaluated if `a` is truthy
- [ ] No assignment occurs if `a` is truthy

### 3.2 Logical AND Assignment
- [ ] `a &&= b` assigns `b` to `a` if `a` is truthy
- [ ] Equivalent to `a && (a = b)` (not `a = a && b`)
- [ ] Short-circuits: `b` not evaluated if `a` is falsy
- [ ] No assignment occurs if `a` is falsy

### 3.3 Nullish Coalescing Assignment
- [ ] `a ??= b` assigns `b` to `a` if `a` is nullish (`null` or `undefined`)
- [ ] Equivalent to `a ?? (a = b)` (not `a = a ?? b`)
- [ ] Short-circuits: `b` not evaluated if `a` is not nullish
- [ ] No assignment occurs if `a` is not nullish

### 3.4 Property Assignment
- [ ] `obj.prop ||= value` works on properties
- [ ] `obj.prop &&= value` works on properties
- [ ] `obj.prop ??= value` works on properties
- [ ] `obj[key] ||= value` computed property access
- [ ] Getter called only once for short-circuiting check

### 3.5 Short-Circuit Semantics
- [ ] Side effects avoided when short-circuiting
- [ ] `obj.x ||= computeValue()` - `computeValue` not called if `obj.x` truthy
- [ ] Setter not called when short-circuiting

---

## 4. Numeric Separators

### 4.1 Underscore Separators
- [ ] `_` allowed as separator in numeric literals
- [ ] `1_000_000` equals `1000000`
- [ ] Improves readability for large numbers
- [ ] No effect on value

### 4.2 Allowed Positions
- [ ] Between digits: `1_234`
- [ ] In decimal: `3.14_15`
- [ ] In exponential: `1e1_0`
- [ ] In binary: `0b1010_0001`
- [ ] In octal: `0o12_34`
- [ ] In hexadecimal: `0xDE_AD_BE_EF`
- [ ] In BigInt: `1_000_000n`

### 4.3 Not Allowed
- [ ] At start: `_123` is identifier, not number
- [ ] At end: `123_` is `SyntaxError`
- [ ] Adjacent to `.`: `3_.14` or `3._14` is `SyntaxError`
- [ ] Adjacent to `e`/`E`: `1_e10` or `1e_10` is `SyntaxError`
- [ ] Adjacent to `x`/`o`/`b`: `0_x10` is `SyntaxError`
- [ ] Adjacent to `n`: `100_n` is `SyntaxError`
- [ ] Multiple consecutive: `1__2` is `SyntaxError`

### 4.4 Not in String Parsing
- [ ] `parseInt("1_000")` returns `1` (stops at `_`)
- [ ] `parseFloat("1_000")` returns `1`
- [ ] `Number("1_000")` returns `NaN`
- [ ] Only works in source code literals

---

## 5. WeakRefs and FinalizationRegistry

### 5.1 WeakRef Constructor
- [ ] `new WeakRef(target)` creates weak reference
- [ ] `target` must be an object or symbol
- [ ] `TypeError` for non-object primitive targets

### 5.2 WeakRef.prototype.deref
- [ ] `weakRef.deref()` returns target if still alive
- [ ] Returns `undefined` if target was garbage collected
- [ ] May return target even if eligible for collection

### 5.3 WeakRef Semantics
- [ ] Does not prevent target from being garbage collected
- [ ] Collection timing is non-deterministic
- [ ] Cannot observe collection immediately

### 5.4 FinalizationRegistry Constructor
- [ ] `new FinalizationRegistry(callback)` creates registry
- [ ] `callback` called when registered object is collected
- [ ] Callback receives held value (not the collected object)

### 5.5 FinalizationRegistry.prototype.register
- [ ] `registry.register(target, heldValue)` registers target
- [ ] `registry.register(target, heldValue, unregisterToken)` with token
- [ ] `target` is object to observe
- [ ] `heldValue` passed to cleanup callback
- [ ] `unregisterToken` used to unregister later

### 5.6 FinalizationRegistry.prototype.unregister
- [ ] `registry.unregister(token)` removes registration
- [ ] Returns `true` if registration existed
- [ ] Returns `false` if no registration found
- [ ] Uses `unregisterToken` from registration

### 5.7 Cleanup Callback
- [ ] Called in a separate job (not synchronously)
- [ ] May be called multiple times if multiple objects collected
- [ ] May never be called (implementation discretion)
- [ ] Should not throw (uncaught errors are swallowed)

### 5.8 Usage Warnings
- [ ] Do not rely on cleanup for correctness
- [ ] GC timing is unpredictable
- [ ] Use only when necessary (resource management hints)
- [ ] Avoid in most application code

---

## Summary Statistics

| Category | Items |
|----------|-------|
| String.prototype.replaceAll | ~16 |
| Promise.any and AggregateError | ~12 |
| Logical Assignment Operators | ~18 |
| Numeric Separators | ~18 |
| WeakRefs and FinalizationRegistry | ~22 |
| **Total** | **~86** |

---

## References

- [ECMA-262 12th Edition (ES2021)](https://262.ecma-international.org/12.0/)
- [MDN String.prototype.replaceAll](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/replaceAll)
- [MDN Promise.any](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise/any)
- [MDN WeakRef](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WeakRef)
- [MDN FinalizationRegistry](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/FinalizationRegistry)
- [Kangax ES2016+ Compatibility Table](https://kangax.github.io/compat-table/es2016plus/)
