# ECMAScript 2015 (ES6) Compatibility Checklist

This document outlines features added in ES2015 (ECMA-262 6th Edition, June 2015). ES2015 is the largest single update to JavaScript, introducing classes, modules, iterators, promises, and many other features.

**Prerequisites**: Complete ES5 implementation first.

**Reference**: [ECMA-262 6th Edition](https://262.ecma-international.org/6.0/)

---

## Table of Contents

1. [Let and Const Declarations](#1-let-and-const-declarations)
2. [Arrow Functions](#2-arrow-functions)
3. [Classes](#3-classes)
4. [Enhanced Object Literals](#4-enhanced-object-literals)
5. [Template Literals](#5-template-literals)
6. [Destructuring](#6-destructuring)
7. [Default, Rest, and Spread](#7-default-rest-and-spread)
8. [Iterators and For-Of](#8-iterators-and-for-of)
9. [Generators](#9-generators)
10. [Promises](#10-promises)
11. [Modules](#11-modules)
12. [Symbols](#12-symbols)
13. [Collections (Map, Set, WeakMap, WeakSet)](#13-collections-map-set-weakmap-weakset)
14. [Proxies and Reflect](#14-proxies-and-reflect)
15. [New Built-in Methods](#15-new-built-in-methods)
16. [Typed Arrays](#16-typed-arrays)
17. [Miscellaneous Syntax](#17-miscellaneous-syntax)
18. [Unicode and RegExp](#18-unicode-and-regexp)
19. [Tail Call Optimization](#19-tail-call-optimization)

---

## 1. Let and Const Declarations

### 1.1 Let Declaration
- [ ] `let` keyword
- [ ] Block scoping (not function scoping)
- [ ] No hoisting to block start (Temporal Dead Zone)
- [ ] `ReferenceError` when accessing before declaration
- [ ] No redeclaration in same scope
- [ ] Redeclaration allowed in nested blocks
- [ ] `let` in `for` loop creates fresh binding per iteration
- [ ] `let` in `for-in`/`for-of` creates fresh binding per iteration

### 1.2 Const Declaration
- [ ] `const` keyword
- [ ] Block scoping (same as `let`)
- [ ] Temporal Dead Zone (same as `let`)
- [ ] Must have initializer
- [ ] `SyntaxError` if no initializer
- [ ] `TypeError` on reassignment (strict mode)
- [ ] Silent failure on reassignment (sloppy mode)
- [ ] Object/array values are still mutable
- [ ] `const` in `for-of`/`for-in` allowed (fresh binding each iteration)
- [ ] `const` in C-style `for` loop is an error if loop variable mutated

### 1.3 Block Scoping Semantics
- [ ] Block creates new lexical environment
- [ ] Variables not visible outside block
- [ ] Shadowing of outer variables
- [ ] `[[ThisBindingStatus]]` for lexical environments

---

## 2. Arrow Functions

### 2.1 Syntax
- [ ] `(params) => expression` (expression body)
- [ ] `(params) => { statements }` (block body)
- [ ] `param => expression` (single parameter, no parens)
- [ ] `() => expression` (no parameters)
- [ ] `(a, b) => expression` (multiple parameters)
- [ ] Parentheses required for zero or 2+ parameters
- [ ] Object literal return requires parens: `() => ({ key: value })`

### 2.2 Lexical Bindings
- [ ] Lexical `this` (inherits from enclosing scope)
- [ ] No own `this` binding
- [ ] Lexical `arguments` (inherits from enclosing scope)
- [ ] No own `arguments` object
- [ ] Lexical `super` (inherits from enclosing scope)
- [ ] Lexical `new.target` (inherits from enclosing scope)

### 2.3 Restrictions
- [ ] Cannot be used as constructor (`new` throws `TypeError`)
- [ ] No `prototype` property
- [ ] Cannot use `yield` (not a generator)
- [ ] Cannot change `this` via `call`/`apply`/`bind`

---

## 3. Classes

### 3.1 Class Declaration
- [ ] `class Name { }` syntax
- [ ] Class declarations are not hoisted (TDZ applies)
- [ ] Class body is implicitly strict mode
- [ ] Duplicate method names allowed (last wins)
- [ ] `constructor` method defines the constructor function

### 3.2 Class Expression
- [ ] `const C = class { }` (anonymous)
- [ ] `const C = class Name { }` (named, `Name` only visible inside class)

### 3.3 Constructor
- [ ] `constructor(...args) { }` method
- [ ] Default constructor if not specified (empty for base, super call for derived)
- [ ] `new.target` inside constructor
- [ ] `new.target` is `undefined` when called without `new`

### 3.4 Instance Methods
- [ ] Method definition syntax `methodName() { }`
- [ ] Computed method names `[expression]() { }`
- [ ] Methods are non-enumerable
- [ ] Methods are configurable
- [ ] Methods are writable

### 3.5 Static Methods
- [ ] `static methodName() { }`
- [ ] Static methods on constructor, not prototype
- [ ] `static` computed method names
- [ ] `this` in static method refers to constructor

### 3.6 Getter and Setter Methods
- [ ] `get propertyName() { }`
- [ ] `set propertyName(value) { }`
- [ ] `static get` and `static set`
- [ ] Computed getter/setter names

### 3.7 Inheritance
- [ ] `class Child extends Parent { }`
- [ ] `extends` expression (can be any constructor)
- [ ] `extends null` for no prototype chain
- [ ] Derived class must call `super()` before `this` access
- [ ] `ReferenceError` if `this` accessed before `super()` in derived
- [ ] `super()` calls parent constructor
- [ ] `super.method()` calls parent method
- [ ] `super.property` accesses parent property
- [ ] `[[ConstructorKind]]` internal slot ("base" or "derived")

### 3.8 Class Semantics
- [ ] Class creates two objects: constructor and prototype
- [ ] `typeof ClassName` is `"function"`
- [ ] Calling class without `new` throws `TypeError`
- [ ] `ClassName.prototype` is non-writable
- [ ] `ClassName.prototype.constructor === ClassName`

---

## 4. Enhanced Object Literals

### 4.1 Shorthand Property Names
- [ ] `{ x, y }` equivalent to `{ x: x, y: y }`
- [ ] Works with any identifier

### 4.2 Shorthand Method Definitions
- [ ] `{ method() { } }` equivalent to `{ method: function() { } }`
- [ ] Shorthand methods are non-enumerable (unlike ES5 style)
- [ ] Shorthand methods have `[[HomeObject]]` for `super`

### 4.3 Computed Property Names
- [ ] `{ [expression]: value }`
- [ ] `{ [expression]() { } }` (computed method name)
- [ ] `{ get [expression]() { } }` (computed getter)
- [ ] `{ set [expression](v) { } }` (computed setter)
- [ ] Expression evaluated at object creation time

### 4.4 __proto__ Property
- [ ] `{ __proto__: obj }` sets prototype (special syntax, not computed)
- [ ] Only one `__proto__` allowed per literal
- [ ] `__proto__` as computed key `{ ["__proto__"]: x }` is regular property

### 4.5 Super in Object Literals
- [ ] `super.property` in concise methods
- [ ] `super.method()` in concise methods
- [ ] Only works in concise method syntax, not `function` properties

---

## 5. Template Literals

### 5.1 Basic Template Literals
- [ ] Backtick delimiters `` `string` ``
- [ ] Multi-line strings (embedded newlines preserved)
- [ ] Expression interpolation `${expression}`
- [ ] Nested template literals `` `${`nested ${x}`}` ``

### 5.2 Escape Sequences
- [ ] Standard escapes work (`\n`, `\t`, `\\`, etc.)
- [ ] Backtick escape `` \` ``
- [ ] Dollar-brace escape `\${`
- [ ] Unicode escapes `\u{XXXXX}` (code point escapes)

### 5.3 Tagged Templates
- [ ] `` tag`string` `` syntax
- [ ] Tag function receives (strings, ...values)
- [ ] `strings` is array of literal portions
- [ ] `strings.raw` contains raw (unescaped) strings
- [ ] Tag function can return any value
- [ ] `String.raw` built-in tag function

### 5.4 Template Literal Semantics
- [ ] Template strings array is frozen
- [ ] Same template literal returns same strings array (cached)
- [ ] `strings.raw` is also frozen

---

## 6. Destructuring

### 6.1 Array Destructuring
- [ ] `const [a, b] = array`
- [ ] `let [a, b] = array`
- [ ] `var [a, b] = array`
- [ ] Skip elements with holes `[a, , b]`
- [ ] Rest element `[a, ...rest]`
- [ ] Rest must be last element
- [ ] Nested array destructuring `[a, [b, c]]`
- [ ] Default values `[a = 1, b = 2]`
- [ ] Default evaluated only if value is `undefined`
- [ ] Works on any iterable, not just arrays

### 6.2 Object Destructuring
- [ ] `const { a, b } = obj`
- [ ] `let { a, b } = obj`
- [ ] `var { a, b } = obj`
- [ ] Renaming `{ a: newName }`
- [ ] Default values `{ a = 1 }`
- [ ] Renaming with default `{ a: newName = 1 }`
- [ ] Nested object destructuring `{ a: { b } }`
- [ ] Computed property names `{ [expr]: x }`
- [ ] Rest properties `{ a, ...rest }` (ES2018, but pattern established)

### 6.3 Assignment Destructuring
- [ ] `[a, b] = array` (assign to existing variables)
- [ ] `({ a, b } = obj)` (parens required for object)
- [ ] Destructuring in `for-of`: `for (const [k, v] of map)`
- [ ] Destructuring in `for-in`: `for (const { x } in obj)`

### 6.4 Parameter Destructuring
- [ ] `function f([a, b]) { }`
- [ ] `function f({ a, b }) { }`
- [ ] `function f({ a = 1, b = 2 } = {}) { }` (default for whole param)
- [ ] Arrow functions `([a, b]) => a + b`

### 6.5 Destructuring Semantics
- [ ] `TypeError` when destructuring `null` or `undefined`
- [ ] Missing properties yield `undefined`
- [ ] Evaluation order: left to right
- [ ] Defaults can reference earlier bindings: `[a, b = a]`

---

## 7. Default, Rest, and Spread

### 7.1 Default Parameters
- [ ] `function f(a = 1) { }`
- [ ] Default evaluated at call time (not definition time)
- [ ] Default only used when argument is `undefined`
- [ ] Explicit `undefined` triggers default
- [ ] `null` does not trigger default
- [ ] Defaults can reference earlier parameters: `f(a, b = a)`
- [ ] Defaults can call functions: `f(a = getDefault())`
- [ ] Defaults create own scope (TDZ for parameters)
- [ ] `arguments` does not reflect defaults (unlike ES5)
- [ ] `function.length` excludes parameters with defaults

### 7.2 Rest Parameters
- [ ] `function f(...args) { }`
- [ ] `function f(a, b, ...rest) { }`
- [ ] Rest parameter is a real Array
- [ ] Rest must be last parameter
- [ ] `SyntaxError` if not last
- [ ] Only one rest parameter allowed
- [ ] `function.length` excludes rest parameter
- [ ] No `arguments` object needed with rest

### 7.3 Spread Operator (in calls and arrays)
- [ ] `f(...array)` spreads into function arguments
- [ ] `f(a, ...array, b)` spread in middle
- [ ] `[...array]` spreads into array literal
- [ ] `[a, ...array, b]` spread in middle of array
- [ ] Works on any iterable
- [ ] `[...str]` spreads string into characters
- [ ] `[...map]` spreads Map into [key, value] pairs
- [ ] `new Constructor(...args)` spread in `new` expression

---

## 8. Iterators and For-Of

### 8.1 Iterator Protocol
- [ ] `Symbol.iterator` method
- [ ] Iterator object has `next()` method
- [ ] `next()` returns `{ value, done }`
- [ ] `done: true` signals completion
- [ ] `value` is `undefined` when `done: true` (optional)
- [ ] Iterator is single-pass (consumed after iteration)

### 8.2 Iterable Protocol
- [ ] Object with `[Symbol.iterator]()` method
- [ ] Method returns an iterator
- [ ] Built-in iterables: Array, String, Map, Set, TypedArray
- [ ] `arguments` is iterable
- [ ] NodeList and other DOM collections are iterable

### 8.3 For-Of Loop
- [ ] `for (const x of iterable) { }`
- [ ] `for (let x of iterable) { }`
- [ ] `for (var x of iterable) { }`
- [ ] `for (x of iterable) { }` (assignment to existing)
- [ ] Destructuring: `for (const [a, b] of iterable) { }`
- [ ] Calls `[Symbol.iterator]()` once at start
- [ ] Calls `next()` before each iteration
- [ ] Loop ends when `done: true`
- [ ] `break` exits loop (calls `return()` if present)
- [ ] `throw` in loop body (calls `return()` if present)

### 8.4 Iterator Return and Throw
- [ ] Optional `return()` method for cleanup
- [ ] `return()` called on early exit (break, throw, return)
- [ ] Optional `throw()` method (used by generators)

### 8.5 Built-in Iterators
- [ ] `Array.prototype[Symbol.iterator]` (same as `values`)
- [ ] `Array.prototype.keys()` returns index iterator
- [ ] `Array.prototype.values()` returns value iterator
- [ ] `Array.prototype.entries()` returns [index, value] iterator
- [ ] `String.prototype[Symbol.iterator]` (code point iteration)
- [ ] `Map.prototype[Symbol.iterator]` (same as `entries`)
- [ ] `Map.prototype.keys()`
- [ ] `Map.prototype.values()`
- [ ] `Map.prototype.entries()`
- [ ] `Set.prototype[Symbol.iterator]` (same as `values`)
- [ ] `Set.prototype.keys()` (same as `values`)
- [ ] `Set.prototype.values()`
- [ ] `Set.prototype.entries()` returns [value, value]

### 8.6 %IteratorPrototype%
- [ ] Common prototype for all built-in iterators
- [ ] `%IteratorPrototype%[Symbol.iterator]()` returns `this`

---

## 9. Generators

### 9.1 Generator Function Declaration
- [ ] `function* name() { }` syntax
- [ ] Generator functions return generator objects
- [ ] Body not executed until `next()` called
- [ ] `yield` expression pauses execution
- [ ] `yield value` yields a value
- [ ] `yield` without value yields `undefined`

### 9.2 Generator Function Expression
- [ ] `const g = function* () { }`
- [ ] `const g = function* name() { }` (named)

### 9.3 Generator Methods
- [ ] `{ *method() { } }` in object literals
- [ ] `class { *method() { } }` in classes
- [ ] `static *method()` for static generator methods
- [ ] Computed names `{ *[expr]() { } }`

### 9.4 Yield Expression
- [ ] `yield` is an expression, has a value
- [ ] Value of `yield` is argument to next `next(arg)`
- [ ] First `next()` argument is discarded
- [ ] `yield` has low precedence (use parens in expressions)

### 9.5 Yield* Delegation
- [ ] `yield* iterable` delegates to another iterable
- [ ] Iterates through all values of inner iterable
- [ ] `yield*` expression value is return value of inner generator
- [ ] Passes `next()` arguments through to inner generator
- [ ] Passes `throw()` through to inner generator
- [ ] Passes `return()` through to inner generator

### 9.6 Generator Object
- [ ] Generator objects are iterators
- [ ] `next(value)` resumes execution
- [ ] `return(value)` terminates generator
- [ ] `throw(error)` throws into generator
- [ ] `[Symbol.iterator]()` returns `this`
- [ ] Generator objects have `[[GeneratorState]]`
- [ ] States: "suspendedStart", "suspendedYield", "executing", "completed"

### 9.7 Generator Return
- [ ] `return value` in generator
- [ ] Returns `{ value, done: true }`
- [ ] Implicit return yields `{ value: undefined, done: true }`
- [ ] `finally` blocks execute on `return()`

### 9.8 Generator Throw
- [ ] `generator.throw(error)` throws at yield point
- [ ] Can be caught with `try/catch` inside generator
- [ ] Uncaught throw propagates out
- [ ] `finally` blocks execute on uncaught throw

---

## 10. Promises

### 10.1 Promise Constructor
- [ ] `new Promise(executor)` syntax
- [ ] Executor receives `(resolve, reject)` functions
- [ ] Executor runs synchronously
- [ ] `resolve(value)` fulfills promise
- [ ] `reject(reason)` rejects promise
- [ ] `resolve(thenable)` adopts thenable's state
- [ ] Throwing in executor rejects the promise
- [ ] Calling `resolve`/`reject` multiple times has no effect after first

### 10.2 Promise States
- [ ] Pending state (initial)
- [ ] Fulfilled state (resolved with value)
- [ ] Rejected state (rejected with reason)
- [ ] State transitions are one-way and permanent
- [ ] `[[PromiseState]]` internal slot
- [ ] `[[PromiseResult]]` internal slot

### 10.3 Promise.prototype.then
- [ ] `promise.then(onFulfilled, onRejected)`
- [ ] Returns new promise
- [ ] `onFulfilled` called with fulfillment value
- [ ] `onRejected` called with rejection reason
- [ ] Handlers called asynchronously (microtask queue)
- [ ] Non-function handlers are ignored (pass-through)
- [ ] Handler return value resolves returned promise
- [ ] Handler throw rejects returned promise
- [ ] Chaining: `p.then(f).then(g)`

### 10.4 Promise.prototype.catch
- [ ] `promise.catch(onRejected)`
- [ ] Equivalent to `promise.then(undefined, onRejected)`
- [ ] Returns new promise

### 10.5 Promise.resolve
- [ ] `Promise.resolve(value)` returns fulfilled promise
- [ ] `Promise.resolve(promise)` returns same promise if same constructor
- [ ] `Promise.resolve(thenable)` adopts thenable's state

### 10.6 Promise.reject
- [ ] `Promise.reject(reason)` returns rejected promise
- [ ] Does not unwrap thenables

### 10.7 Promise.all
- [ ] `Promise.all(iterable)` returns promise
- [ ] Resolves when all promises resolve
- [ ] Result is array of values in order
- [ ] Rejects immediately if any promise rejects
- [ ] Rejection reason is first rejection
- [ ] Non-promise values are treated as resolved

### 10.8 Promise.race
- [ ] `Promise.race(iterable)` returns promise
- [ ] Resolves/rejects with first settled promise
- [ ] Empty iterable: promise never settles

### 10.9 Thenable Assimilation
- [ ] Object with `then` method is a thenable
- [ ] `then` method called with resolve/reject
- [ ] Thenable errors caught and cause rejection
- [ ] Recursively unwraps nested thenables

---

## 11. Modules

### 11.1 Export Declarations
- [ ] `export { name }` named export
- [ ] `export { name as alias }` renamed export
- [ ] `export { name1, name2 }` multiple exports
- [ ] `export const name = value` inline export
- [ ] `export let name = value`
- [ ] `export var name = value`
- [ ] `export function name() { }` function export
- [ ] `export function* name() { }` generator export
- [ ] `export class Name { }` class export
- [ ] `export default expression` default export
- [ ] `export default function() { }` default function (anonymous)
- [ ] `export default function name() { }` default function (named)
- [ ] `export default class { }` default class
- [ ] `export { name } from 'module'` re-export
- [ ] `export { name as alias } from 'module'` re-export with rename
- [ ] `export * from 'module'` re-export all (excludes default)

### 11.2 Import Declarations
- [ ] `import { name } from 'module'` named import
- [ ] `import { name as alias } from 'module'` renamed import
- [ ] `import { name1, name2 } from 'module'` multiple imports
- [ ] `import defaultExport from 'module'` default import
- [ ] `import * as namespace from 'module'` namespace import
- [ ] `import defaultExport, { name } from 'module'` combined
- [ ] `import defaultExport, * as ns from 'module'` combined
- [ ] `import 'module'` side-effect only import

### 11.3 Module Semantics
- [ ] Modules are strict mode by default
- [ ] Module-level `this` is `undefined`
- [ ] Top-level declarations are module-scoped
- [ ] Imports are live bindings (read-only views)
- [ ] Exports are live bindings (reflect current value)
- [ ] `TypeError` on assignment to imported binding
- [ ] Cyclic dependencies supported via live bindings
- [ ] Static module structure (imports/exports at top level only)
- [ ] `SyntaxError` for import/export not at top level

### 11.4 Module Loading (Host-Defined)
- [ ] `[[HostResolveImportedModule]]` hook
- [ ] Module specifier resolution
- [ ] Module caching (same specifier = same module)
- [ ] `[[RequestedModules]]` internal slot

---

## 12. Symbols

### 12.1 Symbol Primitive
- [ ] `Symbol()` creates unique symbol
- [ ] `Symbol(description)` with description
- [ ] `typeof symbol === "symbol"`
- [ ] Symbols are unique: `Symbol() !== Symbol()`
- [ ] Same description does not mean same symbol
- [ ] Cannot convert to number (`TypeError`)
- [ ] Cannot use `new Symbol()` (`TypeError`)

### 12.2 Symbol.for and Symbol.keyFor
- [ ] `Symbol.for(key)` global symbol registry
- [ ] Same key returns same symbol
- [ ] `Symbol.keyFor(symbol)` returns key for registered symbol
- [ ] `Symbol.keyFor` returns `undefined` for non-registered symbols

### 12.3 Well-Known Symbols
- [ ] `Symbol.hasInstance` - `instanceof` behavior
- [ ] `Symbol.isConcatSpreadable` - `Array.prototype.concat` behavior
- [ ] `Symbol.iterator` - default iterator
- [ ] `Symbol.match` - `String.prototype.match` behavior
- [ ] `Symbol.replace` - `String.prototype.replace` behavior
- [ ] `Symbol.search` - `String.prototype.search` behavior
- [ ] `Symbol.species` - constructor for derived objects
- [ ] `Symbol.split` - `String.prototype.split` behavior
- [ ] `Symbol.toPrimitive` - type coercion behavior
- [ ] `Symbol.toStringTag` - `Object.prototype.toString` tag
- [ ] `Symbol.unscopables` - `with` statement exclusions

### 12.4 Symbol Property Keys
- [ ] Symbols as property keys: `obj[sym] = value`
- [ ] Symbol properties are non-enumerable by default
- [ ] `Object.getOwnPropertySymbols(obj)` returns symbol keys
- [ ] `for-in` does not enumerate symbol properties
- [ ] `Object.keys()` does not include symbols
- [ ] `Object.getOwnPropertyNames()` does not include symbols
- [ ] `JSON.stringify()` ignores symbol properties

### 12.5 Symbol.prototype
- [ ] `Symbol.prototype.toString()` returns `"Symbol(description)"`
- [ ] `Symbol.prototype.valueOf()` returns symbol primitive
- [ ] `Symbol.prototype[Symbol.toStringTag]` is `"Symbol"`

---

## 13. Collections (Map, Set, WeakMap, WeakSet)

### 13.1 Map
- [ ] `new Map()` creates empty map
- [ ] `new Map(iterable)` from [key, value] pairs
- [ ] `map.set(key, value)` returns map (chainable)
- [ ] `map.get(key)` returns value or `undefined`
- [ ] `map.has(key)` returns boolean
- [ ] `map.delete(key)` returns boolean
- [ ] `map.clear()` removes all entries
- [ ] `map.size` property (getter)
- [ ] Keys can be any value (including objects)
- [ ] Key equality uses SameValueZero (`NaN === NaN`, `+0 === -0`)
- [ ] Insertion order is preserved

### 13.2 Map Iteration
- [ ] `map.keys()` returns key iterator
- [ ] `map.values()` returns value iterator
- [ ] `map.entries()` returns [key, value] iterator
- [ ] `map[Symbol.iterator]` same as `entries`
- [ ] `map.forEach(callback [, thisArg])`
- [ ] Callback receives `(value, key, map)`

### 13.3 Set
- [ ] `new Set()` creates empty set
- [ ] `new Set(iterable)` from values
- [ ] `set.add(value)` returns set (chainable)
- [ ] `set.has(value)` returns boolean
- [ ] `set.delete(value)` returns boolean
- [ ] `set.clear()` removes all values
- [ ] `set.size` property (getter)
- [ ] Value equality uses SameValueZero
- [ ] Insertion order is preserved

### 13.4 Set Iteration
- [ ] `set.keys()` same as `values`
- [ ] `set.values()` returns value iterator
- [ ] `set.entries()` returns [value, value] iterator
- [ ] `set[Symbol.iterator]` same as `values`
- [ ] `set.forEach(callback [, thisArg])`
- [ ] Callback receives `(value, value, set)`

### 13.5 WeakMap
- [ ] `new WeakMap()` creates empty weak map
- [ ] `new WeakMap(iterable)` from [key, value] pairs
- [ ] `weakmap.set(key, value)` returns weakmap
- [ ] `weakmap.get(key)` returns value or `undefined`
- [ ] `weakmap.has(key)` returns boolean
- [ ] `weakmap.delete(key)` returns boolean
- [ ] Keys must be objects (`TypeError` otherwise)
- [ ] No `size`, `clear`, or iteration (keys are weak)
- [ ] Keys are held weakly (GC can collect)

### 13.6 WeakSet
- [ ] `new WeakSet()` creates empty weak set
- [ ] `new WeakSet(iterable)` from values
- [ ] `weakset.add(value)` returns weakset
- [ ] `weakset.has(value)` returns boolean
- [ ] `weakset.delete(value)` returns boolean
- [ ] Values must be objects (`TypeError` otherwise)
- [ ] No `size`, `clear`, or iteration (values are weak)

---

## 14. Proxies and Reflect

### 14.1 Proxy Constructor
- [ ] `new Proxy(target, handler)`
- [ ] Target can be any object (including functions, arrays)
- [ ] Handler is object with trap methods
- [ ] Missing traps forward to target
- [ ] `Proxy.revocable(target, handler)` returns `{ proxy, revoke }`
- [ ] Revoked proxy throws `TypeError` on any operation

### 14.2 Proxy Traps
- [ ] `handler.getPrototypeOf(target)` - `Object.getPrototypeOf`
- [ ] `handler.setPrototypeOf(target, proto)` - `Object.setPrototypeOf`
- [ ] `handler.isExtensible(target)` - `Object.isExtensible`
- [ ] `handler.preventExtensions(target)` - `Object.preventExtensions`
- [ ] `handler.getOwnPropertyDescriptor(target, prop)` - `Object.getOwnPropertyDescriptor`
- [ ] `handler.defineProperty(target, prop, desc)` - `Object.defineProperty`
- [ ] `handler.has(target, prop)` - `prop in obj`
- [ ] `handler.get(target, prop, receiver)` - property get
- [ ] `handler.set(target, prop, value, receiver)` - property set
- [ ] `handler.deleteProperty(target, prop)` - `delete obj.prop`
- [ ] `handler.ownKeys(target)` - `Object.keys`, `Object.getOwnPropertyNames`, etc.
- [ ] `handler.apply(target, thisArg, args)` - function call
- [ ] `handler.construct(target, args, newTarget)` - `new` operator

### 14.3 Proxy Invariants
- [ ] `getPrototypeOf` must return object or null
- [ ] `getPrototypeOf` must match target if target is non-extensible
- [ ] `setPrototypeOf` must return false if target is non-extensible and proto differs
- [ ] `isExtensible` must match `Object.isExtensible(target)`
- [ ] `preventExtensions` must return false if target is still extensible
- [ ] `getOwnPropertyDescriptor` must match non-configurable properties
- [ ] `defineProperty` must respect non-configurable/non-writable
- [ ] `has` must return true for non-configurable own properties
- [ ] `get` must return correct value for non-configurable non-writable data properties
- [ ] `set` must return false for non-configurable non-writable data properties
- [ ] `deleteProperty` must return false for non-configurable properties
- [ ] `ownKeys` must include all non-configurable own properties

### 14.4 Reflect Object
- [ ] `Reflect.getPrototypeOf(target)`
- [ ] `Reflect.setPrototypeOf(target, proto)`
- [ ] `Reflect.isExtensible(target)`
- [ ] `Reflect.preventExtensions(target)`
- [ ] `Reflect.getOwnPropertyDescriptor(target, prop)`
- [ ] `Reflect.defineProperty(target, prop, desc)`
- [ ] `Reflect.has(target, prop)`
- [ ] `Reflect.get(target, prop [, receiver])`
- [ ] `Reflect.set(target, prop, value [, receiver])`
- [ ] `Reflect.deleteProperty(target, prop)`
- [ ] `Reflect.ownKeys(target)`
- [ ] `Reflect.apply(target, thisArg, args)`
- [ ] `Reflect.construct(target, args [, newTarget])`

### 14.5 Reflect vs Object Methods
- [ ] `Reflect` methods return booleans instead of throwing
- [ ] `Reflect.defineProperty` returns `false` on failure
- [ ] `Reflect.setPrototypeOf` returns `false` on failure
- [ ] `Reflect` methods are 1:1 with proxy traps

---

## 15. New Built-in Methods

### 15.1 Object Methods
- [ ] `Object.assign(target, ...sources)` - copy properties
- [ ] `Object.is(value1, value2)` - SameValue comparison
- [ ] `Object.setPrototypeOf(obj, proto)` - set prototype
- [ ] `Object.getOwnPropertySymbols(obj)` - symbol properties

### 15.2 Array Methods
- [ ] `Array.from(arrayLike [, mapFn [, thisArg]])` - create from iterable/array-like
- [ ] `Array.of(...items)` - create array from arguments
- [ ] `Array.prototype.copyWithin(target, start [, end])` - copy within array
- [ ] `Array.prototype.fill(value [, start [, end]])` - fill with value
- [ ] `Array.prototype.find(predicate [, thisArg])` - find first matching element
- [ ] `Array.prototype.findIndex(predicate [, thisArg])` - find index of first match
- [ ] `Array.prototype.keys()` - index iterator
- [ ] `Array.prototype.values()` - value iterator
- [ ] `Array.prototype.entries()` - [index, value] iterator
- [ ] `Array.prototype[Symbol.iterator]` - same as values

### 15.3 String Methods
- [ ] `String.fromCodePoint(...codePoints)` - create from code points
- [ ] `String.raw(template, ...subs)` - raw template tag
- [ ] `String.prototype.codePointAt(pos)` - code point at position
- [ ] `String.prototype.normalize([form])` - Unicode normalization (NFC, NFD, NFKC, NFKD)
- [ ] `String.prototype.repeat(count)` - repeat string
- [ ] `String.prototype.startsWith(search [, pos])` - check prefix
- [ ] `String.prototype.endsWith(search [, endPos])` - check suffix
- [ ] `String.prototype.includes(search [, pos])` - check contains
- [ ] `String.prototype[Symbol.iterator]` - code point iterator

### 15.4 Number Methods and Properties
- [ ] `Number.isFinite(value)` - strict finite check
- [ ] `Number.isInteger(value)` - integer check
- [ ] `Number.isNaN(value)` - strict NaN check
- [ ] `Number.isSafeInteger(value)` - safe integer check
- [ ] `Number.parseFloat(string)` - same as global
- [ ] `Number.parseInt(string [, radix])` - same as global
- [ ] `Number.EPSILON` - smallest difference between 1 and next representable
- [ ] `Number.MAX_SAFE_INTEGER` - 2^53 - 1
- [ ] `Number.MIN_SAFE_INTEGER` - -(2^53 - 1)

### 15.5 Math Methods
- [ ] `Math.acosh(x)` - hyperbolic arc-cosine
- [ ] `Math.asinh(x)` - hyperbolic arc-sine
- [ ] `Math.atanh(x)` - hyperbolic arc-tangent
- [ ] `Math.cbrt(x)` - cube root
- [ ] `Math.clz32(x)` - count leading zeros (32-bit)
- [ ] `Math.cosh(x)` - hyperbolic cosine
- [ ] `Math.expm1(x)` - e^x - 1
- [ ] `Math.fround(x)` - nearest 32-bit float
- [ ] `Math.hypot(...values)` - square root of sum of squares
- [ ] `Math.imul(a, b)` - 32-bit integer multiplication
- [ ] `Math.log1p(x)` - ln(1 + x)
- [ ] `Math.log10(x)` - base-10 logarithm
- [ ] `Math.log2(x)` - base-2 logarithm
- [ ] `Math.sign(x)` - sign of number (-1, 0, 1)
- [ ] `Math.sinh(x)` - hyperbolic sine
- [ ] `Math.tanh(x)` - hyperbolic tangent
- [ ] `Math.trunc(x)` - truncate to integer

---

## 16. Typed Arrays

### 16.1 ArrayBuffer
- [ ] `new ArrayBuffer(byteLength)`
- [ ] `ArrayBuffer.isView(arg)` - check if typed array or DataView
- [ ] `ArrayBuffer.prototype.byteLength` (getter)
- [ ] `ArrayBuffer.prototype.slice(begin [, end])`
- [ ] `ArrayBuffer[Symbol.species]`

### 16.2 Typed Array Constructors
- [ ] `Int8Array` - signed 8-bit integers
- [ ] `Uint8Array` - unsigned 8-bit integers
- [ ] `Uint8ClampedArray` - unsigned 8-bit clamped
- [ ] `Int16Array` - signed 16-bit integers
- [ ] `Uint16Array` - unsigned 16-bit integers
- [ ] `Int32Array` - signed 32-bit integers
- [ ] `Uint32Array` - unsigned 32-bit integers
- [ ] `Float32Array` - 32-bit floats
- [ ] `Float64Array` - 64-bit floats

### 16.3 Typed Array Constructor Forms
- [ ] `new TypedArray(length)`
- [ ] `new TypedArray(typedArray)` - copy from typed array
- [ ] `new TypedArray(object)` - from array-like/iterable
- [ ] `new TypedArray(buffer [, byteOffset [, length]])` - view of buffer

### 16.4 Typed Array Static Methods
- [ ] `TypedArray.from(source [, mapFn [, thisArg]])`
- [ ] `TypedArray.of(...items)`
- [ ] `TypedArray.BYTES_PER_ELEMENT`
- [ ] `TypedArray[Symbol.species]`

### 16.5 Typed Array Prototype Properties
- [ ] `buffer` - underlying ArrayBuffer
- [ ] `byteLength` - length in bytes
- [ ] `byteOffset` - offset into buffer
- [ ] `length` - number of elements
- [ ] `BYTES_PER_ELEMENT`

### 16.6 Typed Array Prototype Methods
- [ ] `copyWithin(target, start [, end])`
- [ ] `entries()`
- [ ] `every(callback [, thisArg])`
- [ ] `fill(value [, start [, end]])`
- [ ] `filter(callback [, thisArg])`
- [ ] `find(predicate [, thisArg])`
- [ ] `findIndex(predicate [, thisArg])`
- [ ] `forEach(callback [, thisArg])`
- [ ] `indexOf(searchElement [, fromIndex])`
- [ ] `join([separator])`
- [ ] `keys()`
- [ ] `lastIndexOf(searchElement [, fromIndex])`
- [ ] `map(callback [, thisArg])`
- [ ] `reduce(callback [, initialValue])`
- [ ] `reduceRight(callback [, initialValue])`
- [ ] `reverse()`
- [ ] `set(array [, offset])` - copy values into typed array
- [ ] `slice(start [, end])`
- [ ] `some(callback [, thisArg])`
- [ ] `sort([compareFn])`
- [ ] `subarray(begin [, end])` - new view of same buffer
- [ ] `values()`
- [ ] `[Symbol.iterator]` - same as values
- [ ] `toLocaleString()`
- [ ] `toString()`

### 16.7 DataView
- [ ] `new DataView(buffer [, byteOffset [, byteLength]])`
- [ ] `buffer`, `byteLength`, `byteOffset` properties
- [ ] `getInt8(byteOffset)`
- [ ] `getUint8(byteOffset)`
- [ ] `getInt16(byteOffset [, littleEndian])`
- [ ] `getUint16(byteOffset [, littleEndian])`
- [ ] `getInt32(byteOffset [, littleEndian])`
- [ ] `getUint32(byteOffset [, littleEndian])`
- [ ] `getFloat32(byteOffset [, littleEndian])`
- [ ] `getFloat64(byteOffset [, littleEndian])`
- [ ] `setInt8(byteOffset, value)`
- [ ] `setUint8(byteOffset, value)`
- [ ] `setInt16(byteOffset, value [, littleEndian])`
- [ ] `setUint16(byteOffset, value [, littleEndian])`
- [ ] `setInt32(byteOffset, value [, littleEndian])`
- [ ] `setUint32(byteOffset, value [, littleEndian])`
- [ ] `setFloat32(byteOffset, value [, littleEndian])`
- [ ] `setFloat64(byteOffset, value [, littleEndian])`

---

## 17. Miscellaneous Syntax

### 17.1 Binary and Octal Literals
- [ ] `0b` or `0B` prefix for binary: `0b1010`
- [ ] `0o` or `0O` prefix for octal: `0o777`

### 17.2 Unicode Code Point Escapes
- [ ] `\u{XXXXX}` in strings (1-6 hex digits)
- [ ] `\u{XXXXX}` in identifiers
- [ ] `\u{XXXXX}` in template literals
- [ ] `\u{XXXXX}` in regular expressions (with `u` flag)

### 17.3 New.target
- [ ] `new.target` in functions
- [ ] `undefined` when called without `new`
- [ ] Constructor reference when called with `new`
- [ ] `new.target` in arrow functions (lexical)

### 17.4 Unicode Identifiers
- [ ] Full Unicode identifier support
- [ ] `\u{XXXXX}` escapes in identifiers
- [ ] ID_Start and ID_Continue Unicode categories

---

## 18. Unicode and RegExp

### 18.1 RegExp `u` Flag (Unicode)
- [ ] `u` flag enables Unicode mode
- [ ] Surrogate pairs treated as single code point
- [ ] `\u{XXXXX}` code point escapes
- [ ] `.` matches full code point (including astral)
- [ ] Character classes match full code points
- [ ] Quantifiers apply to full code points
- [ ] Invalid escapes throw `SyntaxError` (strict)

### 18.2 RegExp `y` Flag (Sticky)
- [ ] `y` flag enables sticky matching
- [ ] Match must start at `lastIndex`
- [ ] `lastIndex` updated after match
- [ ] No implicit advance on failure
- [ ] `regexp.sticky` property

### 18.3 RegExp.prototype Properties
- [ ] `RegExp.prototype.flags` - returns flag string
- [ ] `RegExp.prototype.global` (getter)
- [ ] `RegExp.prototype.ignoreCase` (getter)
- [ ] `RegExp.prototype.multiline` (getter)
- [ ] `RegExp.prototype.unicode` (getter)
- [ ] `RegExp.prototype.sticky` (getter)
- [ ] `RegExp.prototype.source` (getter, ES6 refined)

### 18.4 RegExp Constructor Changes
- [ ] `new RegExp(regexp)` copies flags
- [ ] `new RegExp(regexp, flags)` overrides flags
- [ ] `RegExp[Symbol.species]`

---

## 19. Tail Call Optimization

### 19.1 Proper Tail Calls
- [ ] Tail position call does not grow stack
- [ ] Only in strict mode
- [ ] Direct tail call: `return f()`
- [ ] Indirect tail call: `return x ? f() : g()`
- [ ] Tail call in expression: `return a || f()`
- [ ] Not a tail call: `return 1 + f()` (operation after call)
- [ ] Not a tail call: `return f(), g()` (comma operator)
- [ ] `try` blocks disable tail calls for contained calls

---

## Summary Statistics

| Category | Items |
|----------|-------|
| Let and Const Declarations | ~22 |
| Arrow Functions | ~14 |
| Classes | ~32 |
| Enhanced Object Literals | ~14 |
| Template Literals | ~14 |
| Destructuring | ~32 |
| Default, Rest, and Spread | ~26 |
| Iterators and For-Of | ~38 |
| Generators | ~32 |
| Promises | ~42 |
| Modules | ~30 |
| Symbols | ~28 |
| Collections | ~44 |
| Proxies and Reflect | ~42 |
| New Built-in Methods | ~50 |
| Typed Arrays | ~65 |
| Miscellaneous Syntax | ~10 |
| Unicode and RegExp | ~18 |
| Tail Call Optimization | ~8 |
| **Total** | **~561** |

---

## References

- [ECMA-262 6th Edition (ES2015)](https://262.ecma-international.org/6.0/)
- [ES6 Features Overview](https://github.com/lukehoban/es6features)
- [MDN JavaScript Reference](https://developer.mozilla.org/en-US/docs/Web/JavaScript)
- [Kangax ES6 Compatibility Table](https://kangax.github.io/compat-table/es6/)
- [Test262 Test Suite](https://github.com/tc39/test262)
