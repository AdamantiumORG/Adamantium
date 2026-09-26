# Split a project into modules

Create this layout inside an Adamantium project:

```text
code/
|-- main.ad
`-- math/
    `-- tools.ad
```

Define a public function in `code/math/tools.ad`:

```adamantium
pub fun double(value:int) result:int {
    result=value*2;
}
```

Load the file and import the function in `code/main.ad`:

```adamantium
pack math/tools;
use math/tools:double;

fun main() {
    print.newline(double(21));
}
```

`pack` adds a project source module. `use` imports selected public symbols into
the current file. Without `use`, call a public symbol through its qualified
name, such as `math/tools:double(21)`. Functions, classes, and enums are private
by default, so cross-module APIs require `pub`.

Run `adamantium check` after changing module paths. Missing modules, circular
packs, invalid imports, and private access are reported before code generation.
Installed WASM packages use `mod PackageName;` and the same selective `use`
style, as described in [Creating packages](../CREATING_PACKAGES.md).
