# ECMAScript 2019 (ES10) Compatibility Checklist

This document outlines features added in ES2019 (ECMA-262 10th Edition, June 2019). ES2019 introduced array flattening, Object.fromEntries, string trimming methods, and several smaller improvements.

**Prerequisites**: Complete ES2018 implementation first.

**Reference**: [ECMA-262 10th Edition](https://262.ecma-international.org/10.0/)

---

## Table of Contents

1. [Array.prototype.flat and flatMap](#1-arrayprototypeflat-and-flatmap)
2. [Object.fromEntries](#2-objectfromentries)
3. [String.prototype.trimStart and trimEnd](#3-stringprototypetrimstart-and-trimend)
4. [Symbol.prototype.description](#4-symbolprototypedescription)
5. [Optional Catch Binding](#5-optional-catch-binding)
6. [Array.prototype.sort Stability](#6-arrayprototypesort-stability)
7. [JSON Superset](#7-json-superset)
8. [Well-formed JSON.stringify](#8-well-formed-jsonstringify)
9. [Function.prototype.toString Revision](#9-functionprototypetostring-revision)

---

## 1. Array.prototype.flat and flatMap

### 1.1 Array.prototype.flat
- [ ] `array.flat()` flattens by one level (default depth = 1)
- [ ] `array.flat(depth)` flattens by specified depth
- [ ] `array.flat(0)` returns shallow copy (no flattening)
- [ ] `array.flat(Infinity)` flattens all nested arrays
- [ ] Removes holes (sparse array positions)
- [ ] Returns new array, does not modify original
- [ ] Generic (works on array-like objects)

### 1.2 Flat Examples
- [ ] `[1, [2, 3]].flat()` returns `[1, 2, 3]`
- [ ] `[1, [2, [3]]].flat()` returns `[1, 2, [3]]`
- [ ] `[1, [2, [3]]].flat(2)` returns `[1, 2, 3]`
- [ ] `[1, , 3].flat()` returns `[1, 3]` (holes removed)

### 1.3 Array.prototype.flatMap
- [ ] `array.flatMap(callback)` maps then flattens by one level
- [ ] `array.flatMap(callback, thisArg)` with this binding
- [ ] Callback receives `(element, index, array)`
- [ ] Only flattens one level (unlike `flat(Infinity)`)
- [ ] More efficient than `map().flat()`
- [ ] Returns new array

### 1.4 FlatMap Examples
- [ ] `[1, 2].flatMap(x => [x, x * 2])` returns `[1, 2, 2, 4]`
- [ ] `["a b", "c d"].flatMap(s => s.split(" "))` returns `["a", "b", "c", "d"]`
- [ ] Can filter by returning empty array: `arr.flatMap(x => x > 0 ? [x] : [])`

---

## 2. Object.fromEntries

### 2.1 Basic Functionality
- [ ] `Object.fromEntries(iterable)` creates object from key-value pairs
- [ ] Accepts any iterable of `[key, value]` pairs
- [ ] Inverse of `Object.entries()`
- [ ] `Object.fromEntries(Object.entries(obj))` shallow clones object

### 2.2 Input Sources
- [ ] `Object.fromEntries(array)` from array of pairs
- [ ] `Object.fromEntries(map)` from Map
- [ ] `Object.fromEntries(map.entries())` from Map entries
- [ ] `Object.fromEntries(generator)` from generator yielding pairs

### 2.3 Key Coercion
- [ ] Keys are coerced to strings (like regular property assignment)
- [ ] Symbol keys are preserved as symbols
- [ ] Duplicate keys: later values overwrite earlier

### 2.4 Transforming Objects
- [ ] Filter entries: `Object.fromEntries(Object.entries(obj).filter(...))`
- [ ] Map entries: `Object.fromEntries(Object.entries(obj).map(...))`

---

## 3. String.prototype.trimStart and trimEnd

### 3.1 String.prototype.trimStart
- [ ] `str.trimStart()` removes leading whitespace
- [ ] Removes all whitespace characters (same as `trim`)
- [ ] Returns new string, does not modify original
- [ ] `String.prototype.trimLeft` is alias (for web compatibility)

### 3.2 String.prototype.trimEnd
- [ ] `str.trimEnd()` removes trailing whitespace
- [ ] Removes all whitespace characters
- [ ] Returns new string
- [ ] `String.prototype.trimRight` is alias (for web compatibility)

### 3.3 Whitespace Characters Removed
- [ ] Space (U+0020)
- [ ] Tab (U+0009)
- [ ] Line feed (U+000A)
- [ ] Carriage return (U+000D)
- [ ] Form feed (U+000C)
- [ ] Vertical tab (U+000B)
- [ ] No-break space (U+00A0)
- [ ] Byte order mark (U+FEFF)
- [ ] Unicode Zs category (space separators)
- [ ] Line separator (U+2028)
- [ ] Paragraph separator (U+2029)

---

## 4. Symbol.prototype.description

### 4.1 Description Property
- [ ] `symbol.description` getter returns description string
- [ ] Returns `undefined` if symbol has no description
- [ ] `Symbol().description` is `undefined`
- [ ] `Symbol("").description` is `""` (empty string)
- [ ] `Symbol("foo").description` is `"foo"`
- [ ] `Symbol.for("foo").description` is `"foo"`
- [ ] Read-only property (no setter)

---

## 5. Optional Catch Binding

### 5.1 Syntax
- [ ] `try { } catch { }` without binding parameter
- [ ] Parameter is optional when error value not needed
- [ ] Still executes catch block on exception

### 5.2 Examples
- [ ] `try { JSON.parse(x) } catch { return null }` - ignore error details
- [ ] Useful when only handling, not inspecting error

---

## 6. Array.prototype.sort Stability

### 6.1 Stable Sort Requirement
- [ ] `Array.prototype.sort` must be stable
- [ ] Elements comparing equal retain their relative order
- [ ] Previously implementation-defined (unstable allowed)

### 6.2 TypedArray Sort
- [ ] `TypedArray.prototype.sort` must also be stable

---

## 7. JSON Superset

### 7.1 Unescaped Line/Paragraph Separators
- [ ] U+2028 (Line Separator) allowed in strings
- [ ] U+2029 (Paragraph Separator) allowed in strings
- [ ] Previously required escaping as `\u2028` and `\u2029`
- [ ] Makes JSON a syntactic subset of ECMAScript

---

## 8. Well-formed JSON.stringify

### 8.1 Lone Surrogates
- [ ] `JSON.stringify` outputs escape sequences for lone surrogates
- [ ] `JSON.stringify("\uD800")` returns `"\"\\ud800\""` not `"\"\uD800\""`
- [ ] Ensures output is valid Unicode (well-formed UTF-8/UTF-16)
- [ ] Unpaired surrogates: U+D800 to U+DFFF

### 8.2 Paired Surrogates
- [ ] Valid surrogate pairs still output as-is
- [ ] `JSON.stringify("\uD83D\uDE00")` returns the emoji directly

---

## 9. Function.prototype.toString Revision

### 9.1 Source Text Requirement
- [ ] `toString()` returns original source text when available
- [ ] Includes whitespace, comments
- [ ] For built-in functions: `"function name() { [native code] }"`
- [ ] For bound functions: `"function () { [native code] }"`
- [ ] For proxies (callable): `"function () { [native code] }"`

### 9.2 Native Function Format
- [ ] Must return `"function " + name + "() { [native code] }"`
- [ ] Or `"function () { [native code] }"` if no name
- [ ] Line breaks within `[native code]` are implementation-defined

### 9.3 Dynamic Functions
- [ ] Functions created with `Function` constructor return synthesized source
- [ ] `new Function("a", "b", "return a + b").toString()` includes full definition

---

## Summary Statistics

| Category | Items |
|----------|-------|
| Array.prototype.flat and flatMap | ~18 |
| Object.fromEntries | ~10 |
| String.prototype.trimStart and trimEnd | ~14 |
| Symbol.prototype.description | ~7 |
| Optional Catch Binding | ~3 |
| Array.prototype.sort Stability | ~2 |
| JSON Superset | ~4 |
| Well-formed JSON.stringify | ~4 |
| Function.prototype.toString Revision | ~8 |
| **Total** | **~70** |

---

## References

- [ECMA-262 10th Edition (ES2019)](https://262.ecma-international.org/10.0/)
- [MDN Array.prototype.flat](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/flat)
- [MDN Array.prototype.flatMap](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/flatMap)
- [MDN Object.fromEntries](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/fromEntries)
- [Kangax ES2016+ Compatibility Table](https://kangax.github.io/compat-table/es2016plus/)
