# Variables

Variables are changeable by default:

```adamantium
var value = 10;
variable other = 20;
value = value + other;
```

`var` and `variable` are aliases. `ch` and `changeable` are optional explicit modifiers. `static` and `stc` create an immutable binding. Variables use lexical block scope and may shadow a parent variable with a separate value.

Use `value.remove;` to remove a binding. Use `.as_variable` for a synchronized scalar alias and `.disconnect` or `.disconect` to copy its last value into an independent binding.
