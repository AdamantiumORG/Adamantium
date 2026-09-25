# Adamantium examples

Each directory is a complete Adamantium project. From the repository root, run:

```text
cargo run -p adamantium-cli -- run examples/hello_world
```

| Example | Demonstrates |
| --- | --- |
| [hello_world](hello_world/code/main.ad) | Minimal native program |
| [fibonacci](fibonacci/code/main.ad) | Functions, recursion, and loops |
| [calculator](calculator/code/main.ad) | Enums, match, and arithmetic |
| [classes](classes/code/main.ad) | Construction, fields, and methods |
| [generics](generics/code/main.ad) | Generic function specialization |
| [enums](enums/code/main.ad) | Exhaustive enum matching |
| [traits](traits/code/main.ad) | Trait implementation by a class |
| [lists](lists/code/main.ad) | List mutation and iteration |

Examples only use behavior supported by the checked-in compiler. File access
is demonstrated separately by the AdamantiumFiles package, and an HTTP example
will be added when a supported networking package exists.
