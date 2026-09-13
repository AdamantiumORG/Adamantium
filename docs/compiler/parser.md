# Parser

The parser is currently implemented in `src/syntax.rs`. It tokenizes Adamantium source, expands generic declarations, combines project modules, validates visibility and imports, and creates the syntax-level program.

Nested blocks have lexical scopes. Names declared in a child block expire at its closing brace, while parent bindings remain accessible. Local variables may shadow parent variables. Classes, enums, traits, type aliases, and imported symbols have protected namespace rules.

The planned split is `syntax/tokens.rs`, `syntax/lexer.rs`, `syntax/ast.rs`, `syntax/parser.rs`, `syntax/modules.rs`, and `syntax/generics.rs`. This will happen after their shared source-location and error interfaces are defined.
