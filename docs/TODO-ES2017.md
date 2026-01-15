# ECMAScript 2017 (ES8) Compatibility Checklist

This document outlines features added in ES2017 (ECMA-262 8th Edition, June 2017). ES2017 introduced async/await, shared memory, and several utility methods.

**Prerequisites**: Complete ES2016 implementation first.

**Reference**: [ECMA-262 8th Edition](https://262.ecma-international.org/8.0/)

---

## Table of Contents

1. [Async Functions](#1-async-functions)
2. [Object Static Methods](#2-object-static-methods)
3. [String Padding](#3-string-padding)
4. [Object.getOwnPropertyDescriptors](#4-objectgetownpropertydescriptors)
5. [Trailing Commas in Function Parameters](#5-trailing-commas-in-function-parameters)
6. [SharedArrayBuffer and Atomics](#6-sharedarraybuffer-and-atomics)

---

## 1. Async Functions

### 1.1 Async Function Declaration
- [ ] `async function name() { }` syntax
- [ ] Async functions always return a Promise
- [ ] Return value is wrapped in `Promise.resolve()`
- [ ] Thrown errors are wrapped in `Promise.reject()`
- [ ] Async function body can contain `await`

### 1.2 Async Function Expression
- [ ] `const f = async function() { }`
- [ ] `const f = async function name() { }` (named)

### 1.3 Async Arrow Functions
- [ ] `async () => expression`
- [ ] `async () => { statements }`
- [ ] `async param => expression`
- [ ] `async (a, b) => expression`

### 1.4 Async Methods
- [ ] `{ async method() { } }` in object literals
- [ ] `class { async method() { } }` in classes
- [ ] `static async method()` for static async methods
- [ ] Computed names `{ async [expr]() { } }`

### 1.5 Await Expression
- [ ] `await expression` pauses async function
- [ ] `await` unwraps promise value
- [ ] `await` on non-promise returns value directly
- [ ] `await` on thenable calls `.then()`
- [ ] Rejected promise throws in async function
- [ ] `await` only valid inside async function body
- [ ] `SyntaxError` for `await` outside async function

### 1.6 Async Function Semantics
- [ ] Creates implicit promise on invocation
- [ ] `await` suspends execution, queues continuation as microtask
- [ ] Multiple `await` in sequence
- [ ] `try/catch` works with `await` for error handling
- [ ] `finally` executes regardless of resolution
- [ ] `return await promise` vs `return promise` (subtle difference in stack traces)

### 1.7 Async Function Properties
- [ ] `AsyncFunction` constructor (not global, access via `async function(){}.constructor`)
- [ ] `asyncFunction.length` - number of parameters
- [ ] `asyncFunction.name` - function name
- [ ] No `prototype` property on async functions (not constructors)
- [ ] `typeof asyncFn` is `"function"`

---

## 2. Object Static Methods

### 2.1 Object.values
- [ ] `Object.values(obj)` returns array of own enumerable string-keyed property values
- [ ] Order matches `for-in` order (insertion order for string keys)
- [ ] Does not include inherited properties
- [ ] Does not include symbol-keyed properties
- [ ] Does not include non-enumerable properties
- [ ] `Object.values("abc")` returns `["a", "b", "c"]`

### 2.2 Object.entries
- [ ] `Object.entries(obj)` returns array of `[key, value]` pairs
- [ ] Order matches `Object.values` order
- [ ] Each entry is a new array (not live)
- [ ] Useful with `for-of`: `for (const [k, v] of Object.entries(obj))`
- [ ] Can convert to Map: `new Map(Object.entries(obj))`
- [ ] `Object.entries("abc")` returns `[["0", "a"], ["1", "b"], ["2", "c"]]`

---

## 3. String Padding

### 3.1 String.prototype.padStart
- [ ] `str.padStart(targetLength)` pads with spaces
- [ ] `str.padStart(targetLength, padString)` pads with custom string
- [ ] Pads at start (left side) of string
- [ ] If `targetLength <= str.length`, returns `str` unchanged
- [ ] `padString` is truncated if too long
- [ ] `padString` is repeated if needed
- [ ] Empty `padString` returns `str` unchanged
- [ ] `"5".padStart(3, "0")` returns `"005"`

### 3.2 String.prototype.padEnd
- [ ] `str.padEnd(targetLength)` pads with spaces
- [ ] `str.padEnd(targetLength, padString)` pads with custom string
- [ ] Pads at end (right side) of string
- [ ] If `targetLength <= str.length`, returns `str` unchanged
- [ ] `padString` is truncated if too long
- [ ] `padString` is repeated if needed
- [ ] `"5".padEnd(3, "0")` returns `"500"`

---

## 4. Object.getOwnPropertyDescriptors

### 4.1 Basic Functionality
- [ ] `Object.getOwnPropertyDescriptors(obj)` returns object
- [ ] Keys are own property names (string and symbol)
- [ ] Values are property descriptors
- [ ] Includes non-enumerable properties
- [ ] Includes symbol-keyed properties

### 4.2 Use Cases
- [ ] Shallow clone with descriptors: `Object.create(Object.getPrototypeOf(obj), Object.getOwnPropertyDescriptors(obj))`
- [ ] Copy getters/setters properly (unlike `Object.assign`)
- [ ] Preserves property attributes

---

## 5. Trailing Commas in Function Parameters

### 5.1 Function Declarations
- [ ] `function f(a, b,) { }` trailing comma in parameters
- [ ] `function f(a, b,) { }` does not affect `f.length`

### 5.2 Function Calls
- [ ] `f(1, 2,)` trailing comma in arguments

### 5.3 Arrow Functions
- [ ] `(a, b,) => { }` trailing comma in parameters

### 5.4 Method Definitions
- [ ] `{ method(a, b,) { } }` trailing comma in parameters

### 5.5 Destructuring Parameters
- [ ] `function f([a, b,]) { }` (array pattern, already allowed)
- [ ] `function f({ a, b, }) { }` (object pattern, already allowed)

---

## 6. SharedArrayBuffer and Atomics

### 6.1 SharedArrayBuffer
- [ ] `new SharedArrayBuffer(byteLength)`
- [ ] `SharedArrayBuffer.prototype.byteLength` (getter)
- [ ] `SharedArrayBuffer.prototype.slice(begin, end)`
- [ ] Buffer can be shared across workers/agents
- [ ] `SharedArrayBuffer[Symbol.species]`

### 6.2 Atomics Object
- [ ] `Atomics` global object (not a constructor)

### 6.3 Atomic Operations
- [ ] `Atomics.add(typedArray, index, value)` - atomic add, returns old value
- [ ] `Atomics.sub(typedArray, index, value)` - atomic subtract, returns old value
- [ ] `Atomics.and(typedArray, index, value)` - atomic AND, returns old value
- [ ] `Atomics.or(typedArray, index, value)` - atomic OR, returns old value
- [ ] `Atomics.xor(typedArray, index, value)` - atomic XOR, returns old value
- [ ] `Atomics.load(typedArray, index)` - atomic read
- [ ] `Atomics.store(typedArray, index, value)` - atomic write, returns value
- [ ] `Atomics.exchange(typedArray, index, value)` - atomic swap, returns old value
- [ ] `Atomics.compareExchange(typedArray, index, expected, replacement)` - CAS

### 6.4 Synchronization
- [ ] `Atomics.wait(int32Array, index, value, timeout)` - block until notified
- [ ] Returns `"ok"`, `"not-equal"`, or `"timed-out"`
- [ ] Only works on `Int32Array` backed by `SharedArrayBuffer`
- [ ] `Atomics.notify(int32Array, index, count)` - wake waiting agents
- [ ] `Atomics.isLockFree(size)` - check if operations are lock-free

### 6.5 Type Restrictions
- [ ] Atomics operations work on `Int8Array`, `Uint8Array`, `Int16Array`, `Uint16Array`, `Int32Array`, `Uint32Array`
- [ ] Must be backed by `SharedArrayBuffer`
- [ ] `TypeError` for non-integer typed arrays or non-shared buffer

---

## Summary Statistics

| Category | Items |
|----------|-------|
| Async Functions | ~30 |
| Object Static Methods | ~12 |
| String Padding | ~16 |
| Object.getOwnPropertyDescriptors | ~6 |
| Trailing Commas | ~6 |
| SharedArrayBuffer and Atomics | ~24 |
| **Total** | **~94** |

---

## References

- [ECMA-262 8th Edition (ES2017)](https://262.ecma-international.org/8.0/)
- [MDN async function](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/async_function)
- [MDN SharedArrayBuffer](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/SharedArrayBuffer)
- [MDN Atomics](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Atomics)
- [Kangax ES2016+ Compatibility Table](https://kangax.github.io/compat-table/es2016plus/)
