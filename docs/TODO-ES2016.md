# ECMAScript 2016 (ES7) Compatibility Checklist

This document outlines features added in ES2016 (ECMA-262 7th Edition, June 2016). ES2016 was the first release under TC39's new annual release cycle, containing only two new features.

**Prerequisites**: Complete ES2015 implementation first.

**Reference**: [ECMA-262 7th Edition](https://262.ecma-international.org/7.0/)

---

## Table of Contents

1. [Array.prototype.includes](#1-arrayprototypeincludes)
2. [Exponentiation Operator](#2-exponentiation-operator)

---

## 1. Array.prototype.includes

### 1.1 Basic Functionality
- [ ] `array.includes(searchElement)` returns boolean
- [ ] `array.includes(searchElement, fromIndex)` with start position
- [ ] Returns `true` if element found, `false` otherwise
- [ ] Uses SameValueZero comparison (like Map/Set)

### 1.2 Special Cases
- [ ] `NaN` is found: `[NaN].includes(NaN)` returns `true`
- [ ] `+0` and `-0` are treated as equal
- [ ] `undefined` is found in sparse arrays: `[,,,].includes(undefined)` returns `true`
- [ ] Negative `fromIndex` counts from end
- [ ] `fromIndex >= length` returns `false` without searching
- [ ] `fromIndex < -length` searches entire array

### 1.3 Generic Behavior
- [ ] Works on array-like objects via `ToObject`
- [ ] Uses `ToLength` on `length` property
- [ ] Accesses properties via `[[Get]]`

---

## 2. Exponentiation Operator

### 2.1 Basic Syntax
- [ ] `base ** exponent` syntax
- [ ] Right-to-left associativity: `2 ** 3 ** 2` equals `2 ** 9` equals `512`
- [ ] Higher precedence than multiplication/division

### 2.2 Assignment Operator
- [ ] `**=` compound assignment: `x **= y` equivalent to `x = x ** y`

### 2.3 Semantics
- [ ] Equivalent to `Math.pow(base, exponent)`
- [ ] `2 ** 3` equals `8`
- [ ] Fractional exponents: `4 ** 0.5` equals `2`
- [ ] Negative exponents: `2 ** -1` equals `0.5`
- [ ] `0 ** 0` equals `1`
- [ ] `Infinity ** 0` equals `1`
- [ ] `NaN ** 0` equals `1`
- [ ] `1 ** Infinity` equals `NaN`

### 2.4 Syntax Restrictions
- [ ] Unary operators before base require parentheses
- [ ] `(-2) ** 2` is valid (equals `4`)
- [ ] `-2 ** 2` is `SyntaxError` (ambiguous)
- [ ] `+2 ** 2` is `SyntaxError` (ambiguous)
- [ ] `!x ** 2` is `SyntaxError` (ambiguous)
- [ ] `delete x ** 2` is `SyntaxError` (ambiguous)
- [ ] `typeof x ** 2` is `SyntaxError` (ambiguous)
- [ ] `void x ** 2` is `SyntaxError` (ambiguous)

---

## Summary Statistics

| Category | Items |
|----------|-------|
| Array.prototype.includes | ~10 |
| Exponentiation Operator | ~18 |
| **Total** | **~28** |

---

## References

- [ECMA-262 7th Edition (ES2016)](https://262.ecma-international.org/7.0/)
- [MDN Array.prototype.includes](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/includes)
- [MDN Exponentiation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Exponentiation)
- [Kangax ES2016+ Compatibility Table](https://kangax.github.io/compat-table/es2016plus/)
