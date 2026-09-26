# Adamantium - TODO

---

## 1. Project Foundation

* [x] Create the `adamantium` CLI - `build` and `run` with an optional project directory
* [x] Add version information
* [x] Add `--help`
* [x] Add `--version`
* [x] Add compiler error handling
* [x] Add compiler warning system - nonfatal warnings with codes and source locations
* [x] Create basic documentation structure
* [x] Keep the CLI version in the workspace package version
* [x] Document every CLI command, option, and common example in `--help`
* [x] Validate the native toolchain with `adamantium doctor`
* [x] Keep `clean` and `clear` as aliases that remove only the project target directory
* [x] Validate project names, destination paths, and existing directories in `new`
* [x] Initialize existing directories with `adamantium init`
* [x] Support global environment configuration and project configuration
* [x] Use stable CLI exit-code classes for usage, syntax, type, configuration, and toolchain errors
* [x] Suggest corrections for unknown commands and reject unknown options
* [x] Generate Bash, Zsh, Fish, and PowerShell completions
* [x] Color terminal errors and support `--no-color`/`NO_COLOR`
* [x] Support global `--verbose` and `--quiet` output modes

---

## 2. Project Structure

Implement the standard Adamantium project structure:

```text
MyProject/
├── project.toml
├── requirements.toml
├── code/
│   ├── main.ad
│   ├── utils.ad
│   └── tests.ad
└── target/
```

* [x] Implement `project.toml`
* [x] Implement `requirement.toml`
* [x] Implement `/code`
* [x] Implement `/target`
* [x] Validate project structure - required files and metadata; `target` is created when needed
* [x] Detect invalid project structures - missing source/manifests and invalid TOML/metadata
* [x] Add project name
* [x] Add project version
* [x] Add project description
* [x] Add project authors

---

# 3. Compiler Pipeline

Implement:

```text
Adamantium
    ↓
Lexer
    ↓
Parser
    ↓
AST
    ↓
Name Resolution
    ↓
Type Checker
    ↓
Semantic Analysis
    ↓
Code Generation
    ↓
NASM Assembly
    ↓
Object File
    ↓
Linker
    ↓
Executable
```

* [x] Implement lexer
* [x] Implement parser
* [x] Implement AST
* [x] Implement name resolution
* [x] Implement scope resolution - function-local variables; nested block scopes are pending
* [x] Implement type checking
* [x] Implement semantic analysis - names, arity, initialization, named returns, and mutability
* [x] Implement unreachable-code detection - statements after unconditional `return`
* [x] Implement unused-variable detection - reachable reads of locals and parameters
* [x] Implement unused-function detection - call graph rooted at `main`, including disconnected cycles
* [x] Implement invalid-access detection - current names, mutability, calls, methods and unsupported member/index/qualified access
* [x] Implement invalid-import detection - missing modules/symbols, conflicts, paths and dependency cycles
* [x] Implement code generation
* [x] Implement NASM backend
* [x] Implement object file generation
* [x] Implement linker integration
* [x] Implement executable generation

---

# 4. Basic Syntax

* [x] Implement statements
* [x] Implement expressions - typed numeric arithmetic and function calls
* [x] Implement blocks - function and control-flow bodies
* [x] Implement semicolons
* [x] Implement identifiers
* [x] Implement literals - integers, floats, strings, booleans, and `None`
* [x] Implement function calls
* [x] Implement operators - numeric `+`, `-`, `*`, `/`, `%`, comparisons, boolean operators and compound assignments
* [x] Implement operator precedence
* [x] Implement parentheses
* [x] Implement `{ }` blocks - function and control-flow bodies

---

# 5. Comments

Single-line comments:

```adamantium
// comment
```

Multi-line comments:

```adamantium
/*
    comment
*/
```

* [x] Implement `//`
* [x] Implement `/* ... */` - non-nested block comments, including multiple lines
* [x] Detect unterminated multi-line comments - report the opening line and column

---

# 6. Variables

Basic syntax:

```adamantium
var a = 10;
```

* [x] Implement `var`
* [x] Implement `variable`
* [x] Implement variable declaration
* [x] Implement variable assignment
* [x] Implement `=+`, `=-`, `=*`, and `=/` compound assignments
* [x] Implement one-time `clamp(min,max)` on changeable numeric variables
* [x] Implement variable scope - one local scope per function call
* [x] Implement variable shadowing rules
* [x] Prevent use of removed variables
* [x] Prevent invalid reassignment
* [x] Implement fixed variable types

---

# 7. Variable Mutability

Default variables are changeable.

```adamantium
var a = 10;
```

Explicit forms:

```adamantium
var ch a = 10;
var stc b = 20;
```

Full forms:

```adamantium
variable changeable a = 10;
variable static b = 20;
```

* [x] Implement `ch`
* [x] Implement `changeable`
* [x] Implement `stc`
* [x] Implement `static`
* [x] Make variables `changeable` by default
* [x] Prevent modification of static variables
* [x] Add compiler diagnostics for invalid modification

---

# 8. Types

Implement:

### Signed integers

* [x] `i8`
* [x] `i16`
* [x] `i32`
* [x] `i64`

### Unsigned integers

* [x] Remove the experimental `u4` type
* [x] `u8`
* [x] `u16`
* [x] `u32`
* [x] `u64`

### Floating point

* [x] `f32`
* [x] `f64`
* [x] `f128`

### Other types

* [x] `string`
* [x] `bool`
* [x] `None`
* [x] `offset`
* [x] `List` - homogeneous literals, explicit element types, indexing and element assignment
* [x] `enum` - named variants with distinct inferred types
* [x] `class` - required typed fields, methods, construction and value copying
* [x] `fun` - callable function values through variable aliases

### Type aliases

```adamantium
define Numbers = List[int];
```

* [x] Implement `define`
* [x] Make aliases refer to the original type
* [x] Prevent aliases from creating distinct runtime types

---

# 9. Type Inference

Examples:

```adamantium
var a = 10;       // i32
var b = 10.5;     // f64
var c = "Hello";  // string
```

* [x] Implement integer literal inference
* [x] Implement default `i32`
* [x] Implement float literal inference
* [x] Implement default `f64`
* [x] Implement string inference
* [x] Implement boolean inference
* [x] Implement inferred list types - inspect every element and promote compatible numeric types
* [x] Detect ambiguous types - empty and incompatible lists require an explicit element type

---

# 10. Type Conversion

Implement:

```adamantium
var a = b.as(i32);
```

* [x] Implement `as(Type)`
* [x] Implement explicit numeric conversions - numeric type suffixes and `as(Type)`
* [x] Implement automatic numeric promotion
* [x] Define safe conversion rules
* [x] Define narrowing conversion rules
* [x] Detect invalid conversions
* [x] Preserve type safety

Example:

```adamantium
var b = 10.5:f64;
var c = 5:u32;
var a = b + c;
```

* [x] Automatically promote `c` to `f64`
* [x] Infer `a` as `f64`

---

# 11. Optional Values

Implement optional function parameters and class fields:

```adamantium
fun example($value:int) result:None {}
class Example(&value:int) { ... }
```

Possible value:

```text
None
```

or:

```text
true
```

* [x] Implement `$name:Type` optional function parameters for supported value types
* [x] Implement `&name:Type` optional class fields for supported value types
* [x] Allow omitted optional class fields
* [x] Allow optional function parameters
* [x] Represent missing values as `None`
* [x] Add optional-value type checking
* [x] Prevent arithmetic and other unsafe access to optional values
* [x] Support optional strings, f128 values and nested class values
* [x] Add a recursive type representation for nested and optional `List` values

---

# 12. Functions

Syntax:

```adamantium
fun add(a:int, b:int) r:int {
    r = a + b;
}
```

* [x] Implement functions
* [x] Implement parameters - supported value types, passed by value
* [x] Implement named return variables
* [x] Implement return types - supported value types and `None`
* [x] Implement implicit final return
* [x] Implement `return` - `return <named-result>;` only
* [x] Implement early returns
* [x] Implement `None` return type
* [x] Implement function calls
* [x] Implement recursion
* [x] Implement function values - `var callable = function();` and explicit `as_variable`

---

# 13. `main`

Every executable project has:

```adamantium
fun main(...) {
}
```

* [x] Detect `main`
* [x] Validate `main`
* [x] Generate executable entry point
* [x] Implement command-line arguments - named `--name value` pairs
* [x] Map CLI arguments to `main` parameters
* [x] Warn about missing required arguments
* [x] Warn about unknown arguments
* [x] Error on invalid argument types
* [x] Implement optional `$` arguments

Example:

```text
adamantium run --numberone 1 --numbertwo 2
```

---

# 14. Visibility

Implement:

```adamantium
pub
priv
```

* [x] Implement private functions by default
* [x] Implement `pub`
* [x] Implement `priv`
* [x] Implement public classes
* [x] Implement public enums
* [x] Prevent access to private symbols from other modules
* [x] Make `main` special and not require `pub`

---

# 15. Modules and `pack`

Implement:

```adamantium
pack utils;
```

* [x] Implement `pack`
* [x] Find explicitly packed `.ad` files
* [x] Resolve module paths relative to `code`
* [x] Implement nested module paths such as `utils/tools`
* [x] Implement module namespaces and `module:symbol` access
* [x] Detect circular module dependencies
* [x] Detect missing modules

Example:

```adamantium
pack tools.utils;
```

---

# 16. `use`

Implement:

```adamantium
use utils::funkcja;
```

Multiple imports:

```adamantium
use utils::[dodawanie, odejmowanie];
```

* [x] Implement imports
* [x] Implement multiple imports with `module:[a,b]`
* [x] Implement qualified module access with `module:symbol`
* [x] Import only public symbols
* [x] Detect duplicate and conflicting imports
* [x] Detect missing symbols

---

# 17. Classes

Implement:

```adamantium
class Player (
    name: str,
    hp: i32
) {
    ...
}
```

* [x] Implement class declarations
* [x] Implement fields - required named initialization; class-typed fields pending deep copy support
* [x] Implement field types
* [x] Implement methods
* [x] Implement `self`
* [x] Implement class visibility - private by default with `pub` and explicit `priv`
* [x] Implement object creation - named field arguments with value-copy semantics
* [x] Implement object field access
* [x] Implement object method calls
* [x] Implement optional fields
* [x] Implement class scope rules - module-level declarations and class-owned members with `self`

---

# 18. Class Lifecycle

Implement:

```adamantium
__new__
__change__
__remove__
```

* [x] Implement `__new__`
* [x] Automatically call `__new__` on object creation
* [x] Implement `__change__`
* [x] Call `__change__` on class/object changes
* [x] Implement `__remove__`
* [x] Call `__remove__` when an object is removed
* [x] Define lifecycle ordering
* [x] Prevent unsafe recursive lifecycle behavior

---

# 19. Enums

Implement:

```adamantium
enum Direction {
    North,
    South,
    East,
    West
}
```

* [x] Implement enum declarations
* [x] Implement enum variants
* [x] Implement enum values - assignment, function arguments/results and numeric-index printing
* [x] Implement enum comparison - `==` and `!=`
* [x] Implement enum matching
* [x] Implement public enums
* [x] Prevent access to private enums from other modules

---

# 20. Control Flow

## `if`

* [x] Implement `if`
* [x] Remove the legacy `then` keyword from `if`
* [x] Implement `else`
* [x] Implement nested conditions

Example:

```adamantium
if hp <= 0 {
    ...
}
```

## `match`

```adamantium
match value {
    1 => ...
    2 => ...
    _ => ...
}
```

* [x] Implement `match`
* [x] Implement literal and enum pattern matching
* [x] Implement wildcard `_`
* [x] Implement enum matching
* [x] Detect non-exhaustive matches where required
* [x] Detect unreachable branches after wildcard `_`
* [x] Detect duplicate match patterns

---

# 21. Loops

## `for`

```adamantium
for i in 0..10 {
    ...
}
```

* [x] Implement `for`
* [x] Implement exclusive integer ranges
* [x] Implement iteration over lists
* [x] Implement iteration over supported collections - `List` is the current collection type

## `while`

```adamantium
while hp > 0 {
    ...
}
```

* [x] Implement `while`

## `until`

```adamantium
until ready == true {
    ...
}
```

* [x] Implement `until` - repeat while the condition is false

## `loop`

```adamantium
loop {
    ...
}
```

* [x] Implement infinite loops

## Loop control

* [x] Implement `break`
* [x] Implement `continue`
* [x] Validate loop-only statements

---

# 22. Error Handling

## `try`

```adamantium
var err = try {
    ...
}
```

* [x] Implement `try`
* [x] Return `None` on success
* [x] Return error string on failure
* [x] Prevent handled runtime errors from terminating the program or reaching stderr

## `panic`

```adamantium
panic("Something went wrong!");
```

* [x] Implement `panic`
* [x] Print panic message
* [x] Print source line
* [x] Terminate program with exit code 2

## `warn`

```adamantium
warn("HP is low!");
```

* [x] Implement runtime warnings
* [x] Print source line
* [x] Continue program execution

---

# 23. Program Exit

Implement:

```adamantium
exit();
```

and:

```adamantium
exit(code = 45);
```

* [x] Implement normal program termination
* [x] Implement exit codes - automatic codes 0, 1, and 2; explicit exit API is pending
* [x] Return exit code to operating system
* [x] Ensure `exit()` produces no error output

---

# 24. Printing

Implement:

```adamantium
print.newline("Hello");
print.sameline("Hello ");
```

* [x] Implement `print.newline`
* [x] Implement `print.sameline`
* [x] Support strings - UTF-8 literals and string variables
* [x] Support numbers - signed/unsigned integers and f32/f64/f128
* [x] Support booleans
* [x] Support objects where appropriate â€” classes print their names and public fields
* [x] Reject invalid `print(...)` syntax

---

# 25. Alias System

Implement:

```adamantium
var a = b.as_variable;
```

* [x] Implement variable aliases with `as_variable`
* [x] Ensure connected aliases share values instead of copying
* [x] Implement alias chains through shared storage
* [x] Implement function, enum and class symbol aliases
* [x] Allow symbol aliases to be redirected to another symbol
* [x] Implement `get_parent()`
* [x] Implement `get_root()`
* [x] Implement `is_alias()`
* [x] Implement `is_synced()`
* [x] Implement `alias_of()`
* [x] Implement `alias_count()`
* [x] Implement `desync()`
* [x] Implement `change_only()`
* [x] Implement `sync()`
* [x] Implement `disconect` and `disconnect` for scalar value aliases
* [x] Implement `detach()` as a separate API
* [x] Implement `reattach()`
* [x] Define alias lifetime rules
* [x] Prevent alias use-after-lifetime
* [x] Integrate aliases with memory safety - shared slots remain live until the last name is removed

---

# 26. Memory and Offsets

Implement:

```adamantium
var offset = a.get_offset();
var value = offset.value_by_offset;
```

* [x] Implement `offset` and `.offset`
* [x] Implement `get_offset()`
* [x] Implement `value_by_offset` and `by_offset`
* [x] Preserve original value type
* [x] Define offset lifetime rules - local variables in the current function only
* [x] Prevent invalid memory access - offsets cannot escape through signatures or containers
* [x] Prevent use-after-free - dereferencing a removed target is rejected
* [x] Integrate offsets with compile-time memory-safety checks

---

# 27. Variable Renaming

Implement:

```adamantium
b.changename(a);
```

* [x] Implement `changename`
* [x] Change identifier without moving the underlying storage
* [x] Preserve type
* [x] Preserve value
* [x] Preserve memory location
* [x] Update compiler symbol tables
* [x] Define behavior with aliases

---

# 28. Decorators

Implement:

```adamantium
#[test]
fun test_add() {
    ...
}
```

* [x] Implement decorator syntax
* [x] Implement function decorators
* [x] Implement multiple decorators
* [x] Implement decorator execution
* [x] Implement decorators accepting functions
* [x] Implement class decorators
* [x] Apply class decorators to methods
* [x] Implement decorator exclusion

Example:

```adamantium
#[!log]
pub fun attack() {
}
```

---

# 29. Professional Mode

Configuration:

```toml
professional = true
```

* [x] Implement `professional` setting
* [x] Require explicit types for all variable declarations
* [x] Make variables static by default and require `ch` for mutation
* [x] Require explicit concrete types
* [x] Reject implicit variable declarations where required
* [x] Reject generic `int` where an exact type is required
* [x] Improve compiler diagnostics for professional mode

Example:

```adamantium
var stc a = 10:i32;
```

---

# 30. Generics

* [x] Design generic syntax
* [x] Implement generic functions
* [x] Implement generic classes
* [x] Implement generic lists
* [x] Implement generic constraints
* [x] Implement generic type checking
* [x] Implement generic code generation
* [x] Produce useful generic compiler errors

---

# 31. Traits / Interfaces

* [x] Design `trait` syntax
* [x] Implement traits
* [x] Implement trait methods
* [x] Implement trait requirements
* [x] Implement `implements`
* [x] Implement trait type checking
* [x] Implement trait-based generic constraints

---

# 32. Operator Overloading

* [x] Design operator overload syntax - public `__add__`, `__sub__`, `__mul__`, `__div__`, `__eq__`, `__ne__`, `__lt__`, `__le__`, `__gt__`, and `__ge__` methods
* [x] Implement `+`
* [x] Implement `-`
* [x] Implement `*`
* [x] Implement `/`
* [x] Implement comparison operators
* [x] Implement equality operators
* [x] Validate operator implementations
* [x] Prevent unsafe operator behavior - public methods, one required same-class operand, checked result types

---

# 33. Async

Async is an official Adamantium feature provided through:

```toml
[dependencies]
adamantium-async = "1.0"
```

* [ ] Create `adamantium-async`
* [ ] Implement async runtime
* [ ] Implement `async`
* [ ] Implement `await`
* [ ] Implement async functions
* [ ] Implement async return values
* [ ] Implement task spawning
* [ ] Implement task joining
* [ ] Implement async error handling
* [ ] Integrate async with the compiler
* [ ] Integrate async with memory safety
* [x] Add async design documentation
* [ ] Add async tests

---

# 34. File System - `AdamantiumFiles`

Official file-system library.

* [x] Create `AdamantiumFiles`
* [x] Implement file opening
* [x] Implement file reading
* [x] Implement file writing
* [x] Implement file appending
* [x] Implement file creation
* [x] Implement file deletion
* [x] Implement file existence checks
* [x] Implement directory creation
* [x] Implement directory deletion
* [x] Implement directory existence checks
* [x] Implement directory listing
* [x] Implement file metadata
* [x] Implement safe file errors
* [x] Add documentation
* [x] Add tests

---

# 35. JSON - `AdamantiumJson`

Official JSON library.

* [x] Create `AdamantiumJson`
* [x] Implement JSON parsing
* [x] Implement JSON serialization
* [x] Implement JSON objects
* [x] Implement JSON arrays
* [x] Implement JSON strings
* [x] Implement JSON numbers
* [x] Implement JSON booleans
* [x] Implement JSON `null`
* [x] Implement JSON type checking
* [x] Implement JSON file loading
* [x] Implement JSON file saving
* [x] Implement malformed JSON errors
* [x] Add documentation
* [x] Add tests

---

# 36. Test System

Tests are stored in:

```text
/code/tests.ad
```

Example:

```adamantium
&TestsFile:Parallel[5]
&TestsFile:StopOnFailed:DontStopStarted

#[test]
fun test_add() {
    assert(2 + 2 == 4);
}
```

* [x] Implement `tests.ad`
* [x] Implement `#[test]`
* [x] Discover test functions
* [x] Implement test runner
* [x] Implement test result reporting
* [x] Implement passed tests
* [x] Implement failed tests
* [x] Implement test errors
* [x] Implement test filtering
* [x] Implement isolated test execution where required

---

# 37. Test CLI

Implement:

```text
adamantium test run
adamantium test list
adamantium test run <test_name>
```

* [x] Implement `adamantium test`
* [x] Implement `adamantium test run`
* [x] Implement `adamantium test list`
* [x] Implement running a single test
* [x] Implement test summaries
* [x] Implement exit codes for CI
* [x] Implement readable test output
* [x] Implement verbose test output

---

# 38. Test File Directives

Implement:

```adamantium
&TestsFile:Parallel
```

and:

```adamantium
&TestsFile:Parallel[5]
```

* [x] Implement `Parallel`
* [x] Implement `Parallel[n]`
* [x] Automatically determine parallelism when no limit is specified
* [x] Limit concurrently running tests
* [x] Implement `StopOnFailed`
* [x] Implement `StopOnFailed:DontStopStarted`
* [x] Stop scheduling new tests after failure
* [x] Allow already-started tests to finish
* [x] Wait for started tests before finishing
* [x] Report all completed results
* [x] Validate invalid `TestsFile` directives

---

# 39. Assertions

Implement:

```adamantium
assert(value);
```

and:

```adamantium
assert(value, "message");
```

* [x] Implement `assert`
* [x] Implement assertion messages
* [x] Show source line on failure
* [x] Show expected/actual values where possible
* [x] Integrate assertions with the test runner

---

# 40. Package / Dependency Manager

* [x] Design the `wasi-command-v1` package format
* [x] Implement `requirement.toml` package declarations
* [x] Implement dependency resolution - GitHub release URL and version tag mapping
* [x] Implement generic `mod Package` and `use Package:function` bindings
* [x] Execute package functions through the generic WASI command runtime
* [x] Implement dependency installation - download WASM and manifest assets
* [x] Validate package manifests, ABI types, versions, and WASM headers
* [x] Implement dependency versions - `MAJOR.MINOR.PATCH` maps to `adamantium_packet_MAJOR_MINOR_PATCH`
* [x] Implement dependency locking
* [x] Implement package cache
* [x] Implement package publishing
* [ ] Implement package registry
* [x] Detect duplicate dependency declarations through TOML validation
* [x] Detect dependency cycles
* [x] Add `adamantium install`

---

# 41. CLI Commands

Implement:

```text
adamantium run
adamantium build
adamantium check
adamantium install
adamantium test run
adamantium test list
adamantium clean
adamantium clear
adamantium new <project_name_or_path>
```

* [x] Implement `run`
* [x] Implement `build`
* [x] Implement `new`
* [x] Implement `check`
* [x] Implement `install`
* [x] Implement `test`
* [x] Implement `clean`
* [x] Implement `clear`
* [x] Make `clean` and `clear` aliases
* [x] Add command error handling - unknown options and excess arguments are rejected
* [x] Add command help - top-level help documents the available commands

---

# 42. `adamantium check`

Static analysis should detect:

* [x] Type errors
* [x] Unused variables
* [x] Unused functions
* [x] Unreachable code
* [x] Invalid imports
* [x] Private symbol access
* [x] Missing return values
* [x] Invalid assignments
* [x] Invalid conversions
* [x] Invalid aliases
* [x] Invalid offsets
* [x] Invalid decorators
* [x] Invalid class usage
* [x] Invalid enum usage

---

# 43. Memory Safety

Adamantium must remain memory-safe.

* [x] Define ownership model - local value ownership and runtime-managed allocations
* [x] Define borrowing/reference rules - aliases and offsets are local non-owning references
* [x] Define object lifetime rules - runtime storage remains valid until process shutdown
* [x] Define alias lifetime rules - shared storage remains live until its last name is removed
* [x] Define offset lifetime rules - offsets cannot escape their target function
* [x] Prevent use-after-free - no manual free and removed targets cannot be dereferenced
* [x] Prevent double-free - runtime storage has no source-level deallocation operation
* [x] Prevent invalid memory access - typed local references only, without raw pointer arithmetic
* [x] Prevent dangling aliases - removing one name preserves storage used by remaining aliases
* [x] Prevent invalid offsets - validate target type, lifetime, storage and dereference operations
* [x] Validate class lifecycle memory safety - reject recursive change/remove behavior
* [x] Add compiler diagnostics for memory-safety violations
* [x] Add memory-safety tests - aliases, offsets, removals and lifecycle hooks

---

# 44. Runtime

* [x] Design Adamantium runtime - ABI, responsibilities, errors, packages and shutdown are specified
* [x] Implement runtime startup
* [x] Implement runtime shutdown
* [x] Implement printing - strings, integers, floats, booleans, and `None`
* [x] Implement panic handling
* [x] Implement warning handling
* [x] Implement exit codes - normal completion, output failures, arithmetic/range failures
* [x] Implement memory management - process-lifetime allocation with stable addresses and OS reclamation
* [x] Implement string runtime - immutable literal storage and value copies; string operations are pending
* [x] Implement list runtime - contiguous `Value` storage, recursive copies, indexing, iteration and bounds errors
* [x] Implement object runtime - allocation and independent class-value copying
* [x] Report arithmetic overflow, division by zero, and invalid clamp ranges
* [x] Implement error runtime - structured recoverable, panic and package errors integrated with `try`
* [x] Optimize runtime overhead - allocation-free scalar paths and exact-size runtime buffers

---

# 45. NASM Backend

* [x] Generate valid NASM syntax
* [x] Generate functions
* [x] Generate variables - typed local values in stack slots
* [x] Generate arithmetic
* [x] Generate comparisons
* [x] Generate branches
* [x] Generate loops
* [x] Generate function calls
* [x] Generate returns
* [x] Generate classes - runtime-backed field storage, method calls and independent copies
* [x] Generate lists - allocation, copying, indexing, assignment and iteration through NASM
* [x] Generate strings - read-only UTF-8 storage with pointer/length values
* [x] Generate scalar aliases and disconnection copies
* [ ] Generate async support
* [x] Generate runtime calls
* [x] Bundle NASM 3.02 in the portable Windows distribution
* [x] Discover bundled `tools/nasm.exe` automatically
* [ ] Remove the remaining Visual Studio linker and Windows SDK requirement
* [x] Add optimization passes
* [x] Validate generated assembly - NASM assembly and native EXE regression tests

---

# 46. Optimization

* [x] Constant folding
* [x] Constant propagation
* [x] Dead-code elimination
* [x] Dead-function elimination
* [x] Expression simplification
* [x] Inline small stateless functions at `-O2`
* [x] Optimize local variables
* [x] Optimize function calls
* [x] Optimize generated assembly
* [x] Add optimization levels
* [x] Benchmark compiler output
* [x] Benchmark generated programs

---

# 47. Standard Library

Create the official standard library.

* [x] String utilities
* [x] Math
* [x] Collections
* [x] Date/time
* [x] Random numbers
* [x] Environment variables
* [x] Process management
* [x] Networking
* [x] File system
* [x] JSON
* [x] Error handling
* [x] Async
* [x] Testing

---

# 48. Documentation

* [ ] Create official Adamantium documentation
* [x] Language overview
* [x] Installation guide
* [x] Getting started guide
* [x] Variables - currently supported declarations and mutability
* [x] Types - supported scalar types, defaults, and suffix annotations
* [x] Functions - typed parameters and named-result behavior
* [x] Classes
* [x] Enums
* [x] Modules
* [x] Packages
* [x] Aliases
* [x] Memory safety
* [x] Error handling
* [ ] Async
* [x] Testing
* [x] Standard library
* [x] CLI reference
* [x] Compiler reference
* [x] Professional mode
* [x] Examples
* [x] Tutorials

---

# 49. Developer Experience

* [ ] VS Code syntax highlighting
* [ ] VS Code language extension
* [ ] Language Server Protocol (LSP)
* [ ] Autocomplete
* [ ] Go-to-definition
* [ ] Find references
* [ ] Rename symbol
* [ ] Diagnostics
* [ ] Formatting
* [ ] Code snippets
* [ ] Debugging support
* [ ] Integrated test runner

---

# 50. Tooling

* [x] `adamantium fmt`
* [x] `adamantium test`
* [x] `adamantium check`
* [x] `adamantium build`
* [x] `adamantium run`
* [x] `adamantium install`
* [x] `adamantium clean`
* [x] `adamantium doctor`
* [x] `adamantium new`
* [x] `adamantium init`

---

# 51. GitHub Integration

* [x] Create GitHub Actions workflow
* [x] Build compiler on every push
* [x] Run compiler tests
* [x] Run Adamantium tests - native regression programs on Windows
* [x] Run formatting checks
* [x] Run static analysis
* [x] Check spelling with Typos on all three CI runners
* [x] Build release binaries
* [x] Test installed CLI on Windows x86-64 - project creation, check, EXE build, run, tests, clean and clear
* [x] Test installed CLI on Linux x86-64 - project creation, check, ELF build, run, tests, clean and clear
* [x] Test Windows - compiler checks and native EXE regression tests configured
* [x] Test Linux - compiler tests and native generated executable test
* [x] Test macOS - compiler tests and generated x86-64 programs run in CI
* [x] Build documentation in CI with rustdoc warnings denied
* [x] Create portable Windows packaging workflow - manual runs and version tags
* [x] Publish portable ZIP as a GitHub Actions artifact
* [x] Create portable Linux x86-64 ZIP after Linux CLI tests pass
* [x] Publish Linux portable ZIP and checksum to the Nightly release
* [x] Build a standalone CLI that does not require Rust on user machines
* [x] Bundle and checksum the official NASM Windows binary
* [ ] Publish installer
* [x] Publish packages directly on GitHub Releases

---

# 52. Compiler Testing

* [x] Lexer tests - covered through parser regression tests
* [x] Parser tests
* [x] AST tests
* [x] Type checker tests
* [x] Semantic analysis tests
* [x] Module tests - namespaces, qualified calls, use imports and nested files
* [x] Class tests - parsing, typing, visibility, construction, mutation and native copying
* [x] Enum tests
* [x] Alias tests
* [x] Memory-safety tests
* [x] Code generation tests
* [x] NASM generation tests
* [x] Runtime tests
* [x] CLI tests - command parsing and native Windows, Linux and macOS x86-64 workflows
* [x] Integration tests
* [x] Regression tests

---

# 53. Error Messages

Create clear compiler diagnostics.

* [x] Error codes
* [x] Warning codes
* [x] Source locations
* [x] Line and column information
* [x] Error highlighting
* [x] Suggestions
* [x] "Did you mean?" suggestions
* [x] Multi-error reporting - independent project and requirement manifest errors
* [x] Context-aware diagnostics
* [x] Clear runtime panic messages

---

# 54. Cross-Platform Support

* [x] Windows support - native console executables
* [x] Linux support - native x86-64 ELF executables
* [x] macOS support - native x86-64 Mach-O executables
* [x] x86-64 backend - Windows COFF, Linux ELF64 and macOS Mach-O
* [x] Define future ARM64 support - target triples, ABI, object formats and acceptance tests
* [x] Move target-independent typed IR into `adamantium-ir`
* [x] Decouple typed IR operators from parser syntax
* [x] Keep the NASM backend consuming the shared typed IR
* [x] Implement the LLVM ARM64 backend - target triples, data layouts, LLVM IR emission and object command
* [x] Generate Windows ARM64 native executables - verified on a native Windows ARM64 runner
* [x] Generate Linux ARM64 native executables - verified on a native Linux ARM64 runner
* [ ] Create portable Windows ARM64 ZIP packaging
* [ ] Create portable Linux ARM64 ZIP packaging
* [x] Add ARM64 archive-layout and packaged-binary regression tests
* [x] Cross-platform standard library behavior
* [x] Cross-platform file handling
* [x] Cross-platform process handling

---

# 55. Security

* [ ] Validate package sources
* [ ] Validate package contents
* [ ] Prevent malicious package metadata
* [ ] Secure dependency installation
* [ ] Validate file-system operations
* [ ] Prevent unsafe path traversal
* [ ] Review runtime memory safety
* [ ] Review generated assembly safety
* [ ] Add security tests

---

# 56. Release System

* [x] Define semantic versioning
* [x] Create portable Windows ZIP packaging
* [x] Create portable Linux x86-64 ZIP packaging
* [x] Add pinned SHA-256 verification for bundled NASM
* [x] Include the NASM BSD 2-Clause license in distributions
* [x] Build the Windows CLI with the static CRT
* [x] Bundle a linker and required Windows libraries
* [x] Create Windows installer
* [x] Create release channels
* [x] Stable releases - immutable semantic-version tags
* [x] Development releases - movable `dev` pre-release channel
* [x] Nightly releases
* [x] Release notes - generated by GitHub for tagged releases and package publishing
* [ ] Changelog
* [x] Version compatibility rules
* [x] Package compatibility rules

---

# 57. Final Language Specification

* [x] Freeze syntax - Adamantium 0.1 language contract
* [x] Freeze keyword list
* [x] Freeze type system
* [x] Freeze conversion rules
* [x] Freeze memory model
* [x] Freeze alias semantics
* [x] Freeze class lifecycle
* [x] Freeze module system
* [x] Freeze package system
* [ ] Freeze async semantics
* [x] Freeze test system
* [x] Write complete language specification - `docs/language/specification.md`
* [x] Write compiler specification - `docs/compiler/`
* [x] Write standard library specification

---

# 58. First Stable Release

* [ ] Complete compiler
* [ ] Complete CLI
* [ ] Complete core language
* [x] Complete memory-safety system
* [ ] Complete standard library
* [ ] Complete package manager
* [ ] Complete testing system
* [ ] Complete documentation
* [ ] Complete VS Code support
* [x] Complete CI/CD
* [ ] Complete cross-platform builds
* [ ] Perform security audit
* [x] Perform performance benchmarks - scheduled CI records compile time, run time and output sizes
* [ ] Fix all critical bugs
* [ ] Tag `v1.0.0`
* [ ] Publish Adamantium 1.0
* [ ] Publish official documentation
* [ ] Publish official packages
* [ ] Announce stable release

---

# Current Priority

The recommended implementation order is:

1. [x] Lexer
2. [x] Parser
3. [x] AST
4. [x] Variables
5. [x] Types - current scalar, list, enum, class and function-value types
6. [x] Type checker
7. [x] Functions
8. [x] `if`
9. [x] `match`
10. [x] `for`
11. [x] `while`
12. [x] `until`
13. [x] `loop`
14. [x] Modules / `pack` / `use`
15. [x] Classes
16. [x] Enums
17. [x] Memory-safety model
18. [x] Alias system - core value and symbol aliases
19. [x] NASM code generation
20. [x] Linker integration
21. [x] `adamantium build`
22. [x] `adamantium run`
23. [x] `adamantium check`
24. [x] Test system
25. [x] `AdamantiumFiles`
26. [x] `AdamantiumJson`
27. [ ] Package manager
28. [ ] `adamantium-async`
29. [ ] Async compiler support
30. [x] Generics
31. [x] Traits
32. [x] Optimization
33. [ ] Documentation
34. [ ] VS Code support
35. [x] CI/CD - push and pull-request validation workflows
36. [ ] Stable release

# 59. Sequential Development Roadmap

This section defines the **required implementation order for the next stages of Adamantium**.

Items in this section should be completed **in order** unless a task is explicitly blocked by another task.

The goal of this roadmap is to stabilize the current language and compiler before introducing large new features such as async/await.

---

## Phase 1 - Finish the Current Core

### 1. Complete nested lexical scopes

* [x] Replace function-only variable scope with nested lexical block scopes
* [x] Create a scope for every `{ ... }` block where required
* [x] Allow variables to exist only inside their defining scope
* [x] Allow child scopes to access variables from parent scopes
* [x] Prevent parent scopes from accessing variables declared in child scopes
* [x] Define shadowing rules for nested scopes
* [x] Validate shadowing behavior for variables, functions, classes, enums and imports
* [x] Add compiler diagnostics for invalid scope access
* [x] Add parser/type-checker/code-generation tests for nested scopes

### 2. Complete visibility rules

* [x] Restrict `use` imports to public symbols only
* [x] Validate public/private access consistently across all symbol types
* [x] Validate public/private access for functions
* [x] Validate public/private access for classes
* [x] Validate public/private access for enums
* [x] Validate public/private access for class members
* [x] Add regression tests for private symbol access

### 3. Complete exhaustive `match`

* [x] Detect non-exhaustive enum matches
* [x] Detect missing enum variants
* [x] Treat `_` as an exhaustive wildcard
* [x] Detect unreachable branches after exhaustive patterns
* [x] Produce diagnostics listing missing patterns
* [x] Add exhaustive-match tests
* [x] Document `match` exhaustiveness rules

### 4. Complete explicit program exit

* [x] Implement `exit()`
* [x] Implement `exit(code = <integer>)`
* [x] Validate exit-code types
* [x] Validate exit-code ranges
* [x] Return explicit exit codes to the operating system
* [x] Add runtime tests for explicit exit codes

---

# Phase 2 - Stabilize Aliases and Memory Safety

### 5. Finalize the alias model

* [x] Define the complete alias data model
* [x] Define alias parent/root semantics
* [x] Define alias lifetime semantics
* [x] Define synchronization semantics
* [x] Define disconnection semantics
* [x] Define detachment semantics
* [x] Define reattachment semantics
* [x] Define behavior when an aliased variable is removed
* [x] Define behavior when an alias is redirected
* [x] Document all alias guarantees

### 6. Implement the remaining alias API

* [x] Implement `get_parent()`
* [x] Implement `get_root()`
* [x] Implement `is_alias()`
* [x] Implement `is_synced()`
* [x] Implement `alias_of()`
* [x] Implement `alias_count()`
* [x] Implement `desync()`
* [x] Implement `change_only()`
* [x] Implement `sync()`
* [x] Implement `detach()`
* [x] Implement `reattach()`
* [x] Define and implement `changename()`

### 7. Complete memory-safety testing

* [x] Add alias lifetime tests
* [x] Add offset lifetime tests
* [x] Add removed-variable tests
* [x] Add invalid dereference tests
* [x] Add escaped-offset tests
* [x] Add class lifecycle safety tests
* [x] Add nested-scope memory-safety tests
* [x] Add regression tests for every discovered memory-safety bug

---

# Phase 3 - Runtime and Data Structures

### 8. Complete the List runtime

* [x] Define the runtime representation of `List`
* [x] Implement list allocation
* [x] Implement list element storage
* [x] Implement list indexing
* [x] Implement list element assignment
* [x] Implement list copying - recursively copy nested Lists
* [x] Implement nested lists
* [x] Implement list iteration
* [x] Implement list bounds checking
* [x] Report invalid list indexes as runtime errors
* [x] Add list runtime tests
* [x] Generate native list operations through the NASM backend

### 9. Complete string operations

* [x] Define string runtime semantics - immutable UTF-8 pointer/byte-length values
* [x] Implement string length - Unicode scalar count through `length` or `length()`
* [x] Implement string comparison
* [x] Implement string concatenation
* [x] Implement string indexing rules - Unicode scalar indexes returning a string
* [x] Define UTF-8 behavior
* [x] Add string runtime tests
* [x] Add string code-generation tests

### 10. Complete runtime error handling

* [x] Define runtime error representation
* [x] Separate recoverable runtime errors from panics
* [x] Integrate runtime errors with `try`
* [x] Add structured runtime error information
* [x] Preserve source locations where possible
* [x] Add runtime error tests

---

# Phase 4 - Compiler Architecture Cleanup

### 11. Refactor the code generator

* [x] Split `codegen.rs` into focused modules
* [x] Separate expression generation
* [x] Separate statement generation
* [x] Separate function generation
* [x] Separate class generation
* [x] Separate list generation
* [x] Separate control-flow generation
* [x] Separate operator generation
* [x] Separate runtime-call generation
* [x] Keep generated assembly behavior unchanged during the refactor
* [x] Run the complete test suite after every refactor stage

### 12. Refactor the syntax/compiler frontend

* [x] Separate lexer implementation
* [x] Separate token definitions
* [x] Separate parser implementation
* [x] Separate AST definitions
* [x] Separate parser diagnostics
* [x] Keep the public compiler behavior unchanged
* [x] Add regression tests for the refactored frontend

### 13. Define compiler phase boundaries

* [x] Clearly define lexer output
* [x] Clearly define parser output
* [x] Clearly define AST invariants
* [x] Clearly define name-resolution output
* [x] Clearly define type-checker output
* [x] Clearly define semantic-analysis output
* [x] Clearly define code-generation input
* [x] Document compiler phase responsibilities

### 13a. Complete the Cargo workspace migration

* [x] Create the root Cargo workspace
* [x] Extract CLI and project loading
* [x] Extract AST, lexer and parser crates
* [x] Extract semantics, types and diagnostics crates
* [x] Extract codegen, NASM and linker crates
* [x] Extract runtime, package and WASM crates
* [x] Extract the language testing runner
* [x] Add the standard library crate
* [ ] Add formatter and LSP crates when implemented
* [x] Keep the CLI and generated-program behavior stable during extraction
* [x] Give every workspace crate a dedicated integration test suite
* [x] Test changed crates and their transitive downstream dependents in CI
* [x] Route production CLI syntax preflight through the extracted recovery parser
* [ ] Move mature AST construction out of `adamantium-cli`
* [ ] Move the mature semantic/type checker out of `adamantium-cli`
* [ ] Move the mature x86-64 generator and optimizer out of `adamantium-cli`

---

# Phase 5 - Testing Infrastructure

### 14. Create a complete language regression suite

* [x] Create `tests/valid/`
* [x] Create `tests/invalid/`
* [x] Add variable tests
* [x] Add type-system tests
* [x] Add function tests
* [x] Add class tests
* [x] Add enum tests
* [x] Add list tests
* [x] Add generic tests
* [x] Add trait tests
* [x] Add module tests
* [x] Add import tests
* [x] Add alias tests
* [x] Add memory-safety tests
* [x] Add control-flow tests
* [x] Add error-handling tests

### 15. Add compile-fail tests

* [x] Store expected compiler errors for invalid programs
* [x] Verify error codes
* [x] Verify source locations
* [x] Verify important diagnostic text
* [x] Verify suggestions where applicable
* [x] Ensure diagnostics do not regress silently

### 16. Complete parallel test execution

* [x] Implement `Parallel`
* [x] Implement `Parallel[n]`
* [x] Detect available CPU parallelism
* [x] Limit concurrent tests
* [x] Implement `StopOnFailed`
* [x] Implement `StopOnFailed:DontStopStarted`
* [x] Wait for already-started tests
* [x] Report all completed tests
* [x] Add test-directive validation

### 17. Implement assertions

* [x] Implement `assert(value)`
* [x] Implement `assert(value, message)`
* [x] Show source location on assertion failure
* [x] Show expected and actual values where possible
* [x] Integrate assertions with the test runner
* [x] Add assertion tests

---

# Phase 6 - Package System

### 18. Finish package installation

* [x] Define the complete package format
* [x] Define package metadata
* [x] Define package version semantics
* [x] Implement package caching
* [x] Implement dependency locking
* [x] Implement dependency graph resolution
* [x] Detect dependency cycles
* [x] Validate package versions
* [x] Validate package metadata

### 19. Implement package security

* [ ] Validate package sources
* [ ] Validate package contents
* [ ] Prevent malicious package metadata
* [ ] Prevent unsafe extraction paths
* [ ] Prevent path traversal
* [ ] Validate downloaded package integrity
* [ ] Add package-security tests

### 20. Implement WASM package loading

* [x] Define the Adamantium WASM package ABI
* [x] Implement WASM module loading
* [x] Validate WASM modules before execution
* [x] Implement package initialization
* [x] Implement exported function discovery
* [x] Implement Adamantium-to-WASM function calls
* [x] Implement WASM-to-Adamantium value conversion
* [x] Define supported WASM value types
* [x] Define package error propagation
* [x] Add WASM package integration tests

### 21. Implement package publishing

* [x] Define package publishing format
* [x] Implement package metadata generation
* [x] Implement package release generation
* [x] Implement package checksums
* [x] Implement GitHub Release publishing
* [x] Document package publishing

---

# Phase 7 - Official Libraries

### 22. Complete `AdamantiumFiles`

* [x] Implement file opening
* [x] Implement file reading
* [x] Implement file writing
* [x] Implement file appending
* [x] Implement file creation
* [x] Implement file deletion
* [x] Implement file existence checks
* [x] Implement directory creation
* [x] Implement directory deletion
* [x] Implement directory existence checks
* [x] Implement directory listing
* [x] Implement file metadata
* [x] Implement safe file errors
* [x] Define cross-platform behavior
* [x] Add documentation
* [x] Add tests

### 23. Complete `AdamantiumJson`

* [x] Implement JSON parsing
* [x] Implement JSON serialization
* [x] Implement JSON objects
* [x] Implement JSON arrays
* [x] Implement JSON strings
* [x] Implement JSON numbers
* [x] Implement JSON booleans
* [x] Implement JSON `null`
* [x] Implement JSON type checking
* [x] Implement JSON file loading
* [x] Implement JSON file saving
* [x] Implement malformed JSON errors
* [x] Add documentation
* [x] Add tests

### 24. Create the core standard library

* [x] Create the official standard-library repository
* [x] Define standard-library module structure
* [x] Implement string utilities
* [x] Implement math utilities
* [x] Implement collections
* [x] Implement date/time
* [x] Implement random numbers
* [x] Implement environment variables
* [x] Implement process management
* [x] Implement networking
* [x] Implement error utilities
* [x] Document the standard library
* [x] Add standard-library tests

---

# Phase 8 - Professional Mode

### 25. Implement Professional Mode

* [x] Implement `professional = true`
* [x] Require explicit types for variables, function signatures and class fields
* [x] Make variables static by default and require `ch` for mutation
* [x] Require explicit concrete types
* [x] Reject implicit declarations where required
* [x] Reject generic `int` where an exact integer type is required
* [x] Validate all Professional Mode restrictions during semantic analysis
* [x] Improve Professional Mode diagnostics
* [x] Add Professional Mode tests
* [x] Document Professional Mode

---

# Phase 9 - Compiler Optimization

### 26. Implement safe optimization passes

* [x] Implement constant folding
* [x] Implement constant propagation
* [x] Implement dead-code elimination
* [x] Implement dead-function elimination
* [x] Implement expression simplification
* [x] Optimize local variables
* [x] Optimize function calls
* [x] Optimize generated assembly
* [x] Verify every optimization preserves program behavior
* [x] Add optimization regression tests

### 27. Add optimization levels

* [x] Define `-O0`
* [x] Define `-O1`
* [x] Define `-O2`
* [x] Define optimization defaults
* [x] Add optimization CLI options
* [x] Add optimization tests
* [x] Benchmark compiler performance
* [x] Benchmark generated program performance

---

# Phase 10 - Tooling and Developer Experience

### 28. Implement `adamantium fmt`

* [x] Define formatting rules
* [x] Implement formatter
* [x] Make formatting deterministic
* [x] Preserve comments
* [x] Format nested structures correctly
* [x] Add formatter tests
* [x] Add `adamantium fmt`

### 29. Implement `adamantium doctor`

* [x] Detect missing compiler dependencies
* [x] Detect missing NASM
* [x] Detect missing linker dependencies
* [x] Detect invalid project configuration
* [x] Detect invalid package configuration
* [x] Provide actionable diagnostics
* [x] Add `adamantium doctor`

### 30. Implement VS Code support

* [ ] Create syntax-highlighting extension
* [ ] Add Adamantium file recognition
* [ ] Add keywords
* [ ] Add types
* [ ] Add comments
* [ ] Add strings
* [ ] Add diagnostics
* [ ] Add snippets
* [ ] Publish development extension

### 31. Implement Language Server Protocol

* [ ] Create Adamantium language server
* [ ] Implement diagnostics
* [ ] Implement autocomplete
* [ ] Implement go-to-definition
* [ ] Implement find references
* [ ] Implement symbol information
* [ ] Implement rename symbol
* [ ] Implement document formatting
* [ ] Integrate with VS Code

---

# Phase 11 - Cross-Platform and Release Infrastructure

### 32. Remove the Windows linker dependency

* [x] Bundle a supported linker - LLVM `lld-link`
* [x] Bundle required Windows libraries where legally and technically appropriate
* [x] Remove the remaining Visual Studio requirement
* [x] Test clean Windows machines - portable smoke test clears `PATH`, `LIB`, and `LIBPATH`
* [x] Update portable distribution
* [x] Add Windows linker regression tests

### 33. Complete cross-platform behavior

* [x] Define cross-platform standard-library behavior
* [x] Define cross-platform file handling
* [x] Define cross-platform process handling
* [x] Add macOS compiler support
* [x] Add macOS backend support - x86-64 Mach-O through NASM and Apple `cc`
* [x] Add macOS CI - compile and execute generated Adamantium programs
* [x] Add cross-platform integration tests - Windows, Linux and Intel macOS

### 34. Complete release infrastructure

* [x] Define semantic versioning rules
* [x] Define compatibility rules
* [x] Define development releases
* [x] Define nightly releases
* [x] Define stable releases
* [ ] Generate changelogs
* [x] Generate release notes
* [x] Publish release binaries - stable, development, and Nightly archives
* [x] Publish package releases
* [x] Create Windows installer

---

# Phase 12 - Language Specification Freeze

### Compiler hardening baseline

* [x] Freeze core syntax
* [x] Freeze primitive type semantics
* [x] Freeze memory semantics
* [x] Improve diagnostics with stable codes, source highlighting and suggestions
* [x] Complete parser error recovery for multiple grammar errors in one module
* [x] Prevent compiler panics from escaping on user input
* [x] Maintain the complete language regression suite
* [x] Add deterministic malformed-input testing and `cargo-fuzz` targets for the frontend, codegen, runtime values and package loader

### 35. Freeze the core language

* [x] Freeze syntax
* [x] Freeze keywords
* [x] Freeze type system
* [x] Freeze type inference
* [x] Freeze conversion rules
* [x] Freeze function semantics
* [x] Freeze class semantics
* [x] Freeze enum semantics
* [x] Freeze generic semantics
* [x] Freeze trait semantics
* [x] Freeze memory model
* [x] Freeze alias semantics
* [x] Freeze offset semantics
* [x] Freeze module system
* [x] Freeze package system
* [x] Freeze error handling
* [x] Freeze test system

### 36. Write the complete specifications

* [x] Write the complete Adamantium Language Specification - `docs/language/specification.md`
* [x] Write the compiler specification - `docs/compiler/`
* [x] Write the memory-safety specification - `docs/MEMORY_SAFETY.md`
* [x] Write the package specification - `docs/CREATING_PACKAGES.md` and `docs/packages/WASM_ABI.md`
* [x] Write the WASM ABI specification
* [x] Write the standard-library specification
* [x] Write the CLI specification - command reference in `README.md`

---

# Phase 13 - Async

Async should be implemented **only after the core language, runtime, package system and language specification are stable**.

### 37. Design async

* [ ] Define `async` semantics
* [ ] Define `await` semantics
* [ ] Define task semantics
* [ ] Define task ownership
* [ ] Define task lifetime
* [ ] Define cancellation semantics
* [ ] Define async error propagation
* [ ] Define async memory-safety rules
* [ ] Define executor semantics

### 38. Implement `adamantium-async`

* [ ] Create `adamantium-async`
* [ ] Implement async runtime
* [ ] Implement executor
* [ ] Implement tasks
* [ ] Implement task spawning
* [ ] Implement task joining
* [ ] Implement cancellation
* [ ] Implement async errors
* [ ] Add async tests
* [x] Add async design documentation

### 39. Integrate async with the compiler

* [ ] Implement `async`
* [ ] Implement `await`
* [ ] Implement async functions
* [ ] Implement async return values
* [ ] Generate async state machines
* [ ] Integrate async with memory safety
* [ ] Integrate async with classes
* [ ] Integrate async with aliases
* [ ] Integrate async with packages
* [ ] Add async code-generation tests

---

# Phase 14 - Security Audit

### 40. Perform a complete security review

* [ ] Audit package installation
* [ ] Audit package extraction
* [ ] Audit path handling
* [ ] Audit WASM execution
* [ ] Audit runtime memory handling
* [ ] Audit generated assembly
* [ ] Audit compiler input handling
* [ ] Audit dependency handling
* [ ] Add security regression tests
* [ ] Fix all critical security issues

---

# Phase 15 - Adamantium 1.0

### 41. Prepare the stable release

* [ ] Complete core language
* [x] Complete memory-safety system
* [ ] Complete runtime
* [ ] Complete standard library
* [ ] Complete package manager
* [x] Complete WASM package system
* [ ] Complete testing system
* [ ] Complete CLI
* [ ] Complete documentation
* [ ] Complete VS Code support
* [x] Complete CI/CD
* [ ] Complete supported platform builds
* [ ] Complete security audit
* [x] Complete performance benchmarks - CI rejects compiler, execution and output-size regressions
* [ ] Fix all critical bugs
* [ ] Freeze the 1.0 language specification
* [ ] Tag `v1.0.0`
* [ ] Publish Adamantium 1.0
* [ ] Publish official documentation
* [ ] Publish official packages
* [ ] Publish release notes
* [ ] Announce the stable release

---

# Current Execution Order

The following foundation stages are the **single source of truth for what should be worked on next**.

Do not add more language features until the unfinished frontend foundation work
is complete. Work from the top of each stage and do not skip ahead unless the
current item is blocked.

## Stage 1 - Frontend foundation

* [x] Define byte-based `Span`
* [x] Define `SourceFile` and offset-to-line/column conversion
* [x] Define `Token` as `TokenKind` plus `Span`
* [x] Stabilize the complete `TokenKind` contract
* [x] Define structured `LexError` variants
* [x] Implement the character scanner
* [x] Centralize the keyword table
* [x] Implement maximal-munch operator recognition
* [x] Implement string and number scanning
* [x] Handle line and block comments in the lexer
* [x] Emit EOF explicitly
* [x] Define UTF-8 source and byte-offset behavior

## Stage 2 - Lexer verification

* [x] Test valid token streams
* [x] Test lexical edge cases
* [x] Test malformed input and precise error spans
* [x] Test Unicode source and strings
* [x] Test comments and unterminated comments
* [x] Test maximal-munch operators
* [x] Test valid and invalid numbers
* [x] Test string escapes and unterminated strings
* [x] Fuzz arbitrary UTF-8 input and prevent lexer panics

## Stage 3 - Parser foundation

* [x] Give the parser a token stream instead of raw source parsing
* [x] Implement lookahead, advance, take and expect
* [x] Implement parser error recovery and synchronization points - collect independent errors and resume at `;`, `}` or EOF
* [x] Implement Pratt expression parsing
* [x] Parse statements from tokens
* [x] Parse declarations from tokens
* [x] Parse nested blocks from tokens

## Stage 4 - AST contract

* [x] Store source spans on canonical AST nodes
* [x] Keep lexer tokens out of the AST
* [x] Represent operators and language meaning with semantic AST enums
* [x] Preserve spans through resolution, HIR, MIR, IR and code generation

## Stage 5 - Unified diagnostics

* [x] Define common severity, code and message fields
* [x] Define primary spans and labeled secondary spans
* [x] Define notes and actionable help
* [x] Identify lexer, parser, name, type and semantic diagnostic stages
* [x] Connect lexer and parser errors to the common diagnostic model
* [x] Connect name-resolution and semantic errors to the common diagnostic model
* [x] Connect every mature type-checker error to the common diagnostic model
* [x] Render secondary labels, notes and help consistently in the CLI
* [x] Add parser recovery regression and fuzz tests

## Later feature backlog

Resume this backlog only after all five foundation stages are complete.

1. [x] Complete nested lexical scopes
2. [x] Complete public/private import enforcement
3. [x] Implement exhaustive `match`
4. [x] Implement explicit `exit(code = ...)`
5. [x] Finalize alias semantics
6. [x] Implement remaining alias APIs
7. [x] Complete memory-safety regression tests
8. [x] Complete the List runtime
9. [x] Complete string operations
10. [x] Complete runtime error handling
11. [x] Refactor `codegen.rs`
12. [x] Refactor the compiler frontend
13. [x] Define compiler phase boundaries
14. [x] Create the complete language regression suite
15. [x] Create compile-fail tests
16. [x] Complete parallel test execution
17. [x] Implement assertions
18. [x] Complete package installation
19. [ ] Implement package security
20. [x] Implement WASM package loading
21. [x] Implement package publishing
22. [x] Complete `AdamantiumFiles`
23. [x] Complete `AdamantiumJson`
24. [x] Create the core standard library
25. [x] Implement Professional Mode
26. [x] Implement compiler optimizations
27. [x] Add optimization levels
28. [x] Implement `adamantium fmt`
29. [x] Implement `adamantium doctor`
30. [ ] Implement VS Code syntax highlighting
31. [ ] Implement the Adamantium LSP
32. [x] Remove the Windows linker dependency
33. [ ] Complete cross-platform behavior
34. [ ] Complete release infrastructure
35. [x] Freeze the core language
36. [ ] Write the complete language specifications
37. [ ] Design async
38. [ ] Implement `adamantium-async`
39. [ ] Integrate async with the compiler
40. [ ] Perform the complete security audit
41. [ ] Prepare Adamantium 1.0

