# Modules

Load project files relative to `code` with `pack`:

```adamantium
pack utils/tools;
use utils/tools:calculate;
```

Call an unimported public symbol with `utils/tools:calculate()`. Use a list to import several symbols: `use utils:[first,second];`.

Installed WASM packages use `mod PackageName;` followed by the same `use` syntax. Project functions, classes, and enums are private by default.
