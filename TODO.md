# Spacey SpiderMonkey - ES5 Compatibility Roadmap

**Status**: ES3 ✅ Complete (191/191 tests passing) → ES5 🚧 In Progress  
**Target**: Full ES5 compliance within 8 weeks  
**Reference**: [ECMA-262 5.1 Edition](https://262.ecma-international.org/5.1/)

---

## Quick Links

- [Detailed ES5 Spec](docs/specs/es5-implementation.md)
- [ES3 Completion Status](docs/specs/es3-implementation.md)
- [Test262 ES5 Tests](https://github.com/tc39/test262)

---

## Phase 1: Property Descriptors & Object Methods (Week 1-2) 🔴 HIGH PRIORITY

The foundation for ES5 - all other features depend on property descriptors.

### 1.1 Property Descriptor Infrastructure
- [ ] Implement data descriptor type: `{ value, writable, enumerable, configurable }`
- [ ] Implement accessor descriptor type: `{ get, set, enumerable, configurable }`
- [ ] Update `[[DefineOwnProperty]]` internal method
- [ ] Implement property attribute validation rules
- [ ] Add `[[Extensible]]` internal property to all objects

### 1.2 Object Constructor Methods (Introspection)
- [ ] `Object.getPrototypeOf(O)`
- [ ] `Object.getOwnPropertyDescriptor(O, P)`
- [ ] `Object.getOwnPropertyNames(O)` - includes non-enumerable
- [ ] `Object.keys(O)` - enumerable own properties only

### 1.3 Object Constructor Methods (Creation)
- [ ] `Object.create(O)` - create with prototype
- [ ] `Object.create(O, Properties)` - create with properties

### 1.4 Object Constructor Methods (Property Definition)
- [ ] `Object.defineProperty(O, P, Attributes)`
- [ ] `Object.defineProperties(O, Properties)`

### 1.5 Object Constructor Methods (Integrity)
- [ ] `Object.preventExtensions(O)` - prevent adding new properties
- [ ] `Object.seal(O)` - non-extensible + all properties non-configurable
- [ ] `Object.freeze(O)` - sealed + all data properties non-writable
- [ ] `Object.isExtensible(O)`
- [ ] `Object.isSealed(O)`
- [ ] `Object.isFrozen(O)`

**Milestone 1 Deliverable**: Property descriptors work, `Object.defineProperty` and `Object.freeze` functional

---

## Phase 2: Array Iteration Methods (Week 2-3) 🔴 HIGH PRIORITY

Most-used ES5 features in real-world code.

### 2.1 Array Constructor Methods
- [ ] `Array.isArray(arg)`

### 2.2 Index Search Methods
- [ ] `Array.prototype.indexOf(searchElement [, fromIndex])`
- [ ] `Array.prototype.lastIndexOf(searchElement [, fromIndex])`

### 2.3 Iteration Methods
- [ ] `Array.prototype.forEach(callbackfn [, thisArg])`
- [ ] `Array.prototype.map(callbackfn [, thisArg])`
- [ ] `Array.prototype.filter(callbackfn [, thisArg])`
- [ ] `Array.prototype.every(callbackfn [, thisArg])`
- [ ] `Array.prototype.some(callbackfn [, thisArg])`

### 2.4 Reduction Methods
- [ ] `Array.prototype.reduce(callbackfn [, initialValue])`
- [ ] `Array.prototype.reduceRight(callbackfn [, initialValue])`

### 2.5 Callback Behavior
- [ ] Ensure callback receives `(element, index, array)`
- [ ] Implement `thisArg` parameter for `this` binding
- [ ] Skip holes in sparse arrays
- [ ] Make methods generic (work on array-like objects)

**Milestone 2 Deliverable**: All array iteration methods work, including `map`, `filter`, `reduce`

---

## Phase 3: JSON Support (Week 3-4) 🔴 HIGH PRIORITY

Essential for modern web applications.

### 3.1 JSON Object
- [ ] Add `JSON` global object

### 3.2 JSON.parse
- [ ] `JSON.parse(text)` - basic parsing
- [ ] `JSON.parse(text, reviver)` - with transform function
- [ ] Reviver called bottom-up (leaves first)
- [ ] Throw `SyntaxError` on invalid JSON

### 3.3 JSON.stringify
- [ ] `JSON.stringify(value)` - basic serialization
- [ ] `JSON.stringify(value, replacer)` - array or function replacer
- [ ] `JSON.stringify(value, replacer, space)` - indentation
- [ ] Handle `toJSON()` method on objects
- [ ] Return `undefined` for functions, symbols, undefined
- [ ] Throw `TypeError` on circular references
- [ ] `NaN` and `Infinity` become `null`

### 3.4 JSON Grammar Compliance
- [ ] Strict JSON grammar (no trailing commas, no single quotes)
- [ ] Unicode `\uXXXX` escapes
- [ ] Numbers (no hex, no leading zeros except `0.x`)

**Milestone 3 Deliverable**: `JSON.parse` and `JSON.stringify` fully functional

---

## Phase 4: Strict Mode - Syntax (Week 4-5) 🟡 MEDIUM PRIORITY

### 4.1 Directive Prologue
- [ ] Recognize `"use strict";` directive in scripts
- [ ] Recognize `"use strict";` directive in function body
- [ ] Propagate strict mode to nested functions

### 4.2 Syntax Errors in Strict Mode
- [ ] Octal numeric literals forbidden (`0123`)
- [ ] Octal escape sequences forbidden (`"\123"`)
- [ ] `with` statement forbidden
- [ ] `delete` on unqualified identifier forbidden
- [ ] Duplicate parameter names forbidden
- [ ] Duplicate property names in object literals (syntax error)
- [ ] Assignment to `eval` or `arguments` forbidden
- [ ] Reserved words as identifiers forbidden:
  - [ ] `implements`, `interface`, `let`, `package`
  - [ ] `private`, `protected`, `public`, `static`, `yield`

### 4.3 Parser Updates
- [ ] Track strict mode context during parsing
- [ ] Report strict mode syntax errors at parse time
- [ ] Handle strict mode propagation in function declarations

**Milestone 4 Deliverable**: Parser detects and enforces strict mode syntax rules

---

## Phase 5: Strict Mode - Runtime (Week 5-6) 🟡 MEDIUM PRIORITY

### 5.1 `this` Binding Changes
- [ ] `this` is `undefined` (not global) in unbound function calls
- [ ] Preserve `null`/`undefined` as `this` (no boxing)

### 5.2 Assignment Restrictions
- [ ] Assignment to undeclared variable throws `ReferenceError`
- [ ] Assignment to non-writable property throws `TypeError`
- [ ] Assignment to getter-only property throws `TypeError`
- [ ] Adding property to non-extensible object throws `TypeError`
- [ ] `delete` on non-configurable property throws `TypeError`

### 5.3 eval Changes
- [ ] `eval` does not introduce variables into enclosing scope
- [ ] `eval` has its own variable environment

### 5.4 arguments Changes
- [ ] `arguments` object is not linked to parameters
- [ ] `arguments.callee` throws `TypeError`
- [ ] `arguments.caller` throws `TypeError`

### 5.5 Function Restrictions
- [ ] `func.caller` access throws `TypeError`
- [ ] `func.arguments` access throws `TypeError`

**Milestone 5 Deliverable**: Strict mode runtime behavior fully implemented

---

## Phase 6: Function.prototype.bind (Week 6) 🟡 MEDIUM PRIORITY

### 6.1 Implementation
- [ ] `Function.prototype.bind(thisArg [, arg1 [, arg2, ...]])`
- [ ] Returns bound function with fixed `this`
- [ ] Partial application (prepended arguments)
- [ ] Bound function's `length` = original length - bound args
- [ ] Bound function has no `prototype` property
- [ ] `new` on bound function uses original `[[Construct]]`

### 6.2 Internal Slots
- [ ] `[[BoundThis]]` - the bound `this` value
- [ ] `[[BoundArgs]]` - prepended arguments
- [ ] `[[TargetFunction]]` - original function

**Milestone 6 Deliverable**: `Function.prototype.bind` works including `new` calls

---

## Phase 7: Remaining Features (Week 6-7) 🟢 LOW PRIORITY

### 7.1 String Enhancements
- [ ] `String.prototype.trim()`
- [ ] Property access by index (`str[0]`)

### 7.2 Date Enhancements
- [ ] `Date.now()`
- [ ] `Date.prototype.toISOString()`
- [ ] `Date.prototype.toJSON(key)`
- [ ] `Date.parse` for ISO 8601 format

### 7.3 Syntax Additions
- [ ] Getter/setter in object literals: `{ get x() {}, set x(v) {} }`
- [ ] Trailing commas in array literals clarified
- [ ] Reserved words as property names: `obj.class`, `obj.if`

### 7.4 Other Changes
- [ ] `undefined` is non-writable in global scope
- [ ] `parseInt` doesn't treat leading `0` as octal by default

**Milestone 7 Deliverable**: All ES5 features implemented

---

## Phase 8: Testing & Compliance (Week 7-8) 🔴 HIGH PRIORITY

### 8.1 Test Suite
- [ ] Create ES5-specific unit tests
- [ ] Add property descriptor tests
- [ ] Add array method tests
- [ ] Add strict mode tests
- [ ] Add JSON tests

### 8.2 Test262 Compliance
- [ ] Set up Test262 ES5 test harness
- [ ] Run `test/language/` tests
- [ ] Run `test/built-ins/` tests
- [ ] Track pass rate (target: 100%)

### 8.3 Edge Cases
- [ ] Test sparse arrays with all iteration methods
- [ ] Test frozen/sealed objects
- [ ] Test strict mode in eval'd code
- [ ] Test bound functions with `new`

**Milestone 8 Deliverable**: 100% ES5 Test262 pass rate

---

## Implementation Order (Recommended)

```
Week 1:  Property descriptors infrastructure
Week 2:  Object.defineProperty, Object.create, Object.keys
Week 3:  Array iteration methods (forEach, map, filter, reduce)
Week 4:  JSON.parse and JSON.stringify
Week 5:  Strict mode syntax (parser changes)
Week 6:  Strict mode runtime + Function.prototype.bind
Week 7:  String.trim, Date enhancements, syntax additions
Week 8:  Testing, edge cases, Test262 compliance
```

---

## Quick Reference: Files to Modify

| Feature | Primary File(s) |
|---------|-----------------|
| Property Descriptors | `src/runtime/object.rs`, `src/runtime/value.rs` |
| Object Methods | `src/builtins/object.rs` |
| Array Methods | `src/builtins/array.rs` |
| JSON | `src/builtins/json.rs` (new) |
| Strict Mode (syntax) | `src/parser/parser.rs`, `src/lexer/scanner.rs` |
| Strict Mode (runtime) | `src/vm/interpreter.rs`, `src/runtime/context.rs` |
| Function.bind | `src/builtins/function.rs` |
| String.trim | `src/builtins/string.rs` |
| Date enhancements | `src/builtins/date.rs` |

---

## Dependencies

```
Property Descriptors ──┬──► Object Methods
                       │
                       ├──► Array Methods (generic behavior)
                       │
                       └──► Strict Mode (TypeError on violations)

JSON ──► Object.toJSON behavior

Strict Mode Syntax ──► Strict Mode Runtime
```

---

## Progress Tracking

| Phase | Description | Status | Tests |
|-------|-------------|--------|-------|
| 1 | Property Descriptors | ⬜ Not Started | 0/25 |
| 2 | Array Methods | ⬜ Not Started | 0/15 |
| 3 | JSON Support | ⬜ Not Started | 0/20 |
| 4 | Strict Mode (Syntax) | ⬜ Not Started | 0/15 |
| 5 | Strict Mode (Runtime) | ⬜ Not Started | 0/15 |
| 6 | Function.bind | ⬜ Not Started | 0/7 |
| 7 | Remaining Features | ⬜ Not Started | 0/15 |
| 8 | Testing & Compliance | ⬜ Not Started | 0/? |

**Overall ES5 Progress**: 0/~117 items (0%)

---

## Commands

```bash
# Run ES5 tests
cargo test --features es5

# Run specific ES5 feature tests
cargo test test_property_descriptors
cargo test test_array_methods
cargo test test_json
cargo test test_strict_mode

# Run Test262 ES5 tests
cargo run -- --test262 tests/test262/test/language/
cargo run -- --test262 tests/test262/test/built-ins/

# Check ES5 compliance percentage
cargo run -- --compliance es5
```

---

## Notes

- **Start with Property Descriptors** - Everything else depends on them
- **Array methods are high-impact** - Most used ES5 features in real code
- **JSON is essential** - Required for any web application
- **Strict mode can be phased** - Syntax first, then runtime behavior
- **Test continuously** - Run Test262 after each milestone

---

## References

- [ECMA-262 5.1 Edition (HTML)](https://262.ecma-international.org/5.1/)
- [Annotated ES5](https://es5.github.io/)
- [Test262 ES5 Tests](https://github.com/tc39/test262)
- [MDN JavaScript Reference](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference)
- [ES5 Compatibility Table](https://kangax.github.io/compat-table/es5/)
