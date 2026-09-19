# Professional Mode

Enable Professional Mode in `project.toml`:

```toml
professional = true
```

Every variable must have an explicit concrete type. Variables are static by
default. Add `ch` or `changeable` when a variable must be modified after its
declaration. `static` and `stc` remain available as explicit forms. Function
parameters, function results, class fields and other type annotations must also
use concrete types. Generic numeric names such as `int`, `float`, and `u` are
rejected. Use exact types such as `i32`, `f64`, and `u64`.

```adamantium
fun main() {
    var count=10:i32;
    var ch changing_count=10:i32;
    changing_count=11;
    var static name="Adamantium":string;
}
```

The checks run for every loaded project module before normal parsing and type
checking. Projects without the setting keep the standard inference rules.
