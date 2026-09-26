# Variables

Variables are changeable by default:

```adamantium
var value = 10;
variable other = 20;
value = value + other;
```

`var` and `variable` are aliases. `ch` and `changeable` are optional explicit modifiers. `static` and `stc` create an immutable binding. Variables use lexical block scope and may shadow a parent variable with a separate value.

Use `value.remove;` to remove a binding. Use `.as_variable` for a synchronized
scalar alias and `.disconnect()` to copy its last value into an independent
binding. The misspelled `.disconect` form remains temporarily source-compatible
and produces deprecation warning `W004`. New code must use `.disconnect()`.

`old_name.changename(new_name)` renames a source binding. It does not create,
redirect, synchronize, or copy an alias. The value keeps the same storage,
type, mutability, alias parent, root, and synchronization state. This operation
belongs to binding management, while `.as_variable`, `.sync()`, `.detach()`,
and `.disconnect()` manage alias relationships.
