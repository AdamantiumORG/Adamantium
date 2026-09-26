# Create and run a first project

Install a portable Adamantium archive or build the CLI from source. Confirm the
toolchain before creating a project:

```text
adamantium --version
adamantium doctor
adamantium new HelloAdamantium
cd HelloAdamantium
```

Replace `code/main.ad` with:

```adamantium
fun add(left:int,right:int) result:int {
    result=left+right;
}

fun main() {
    var first=20;
    var second=22;
    var answer=add(first,second);

    if answer == 42 {
        print.newline("The answer is correct");
    } else {
        print.newline("The answer changed");
    }
}
```

Check the program without invoking native tools, then build and run it:

```text
adamantium check
adamantium run
adamantium build -O2
```

Generated assembly, objects, runtime libraries, and the executable are written
to `target`. Use `adamantium clean` when you want to remove those artifacts.

Variables are changeable unless declared with `static` or `stc`. Professional
Mode reverses that default and also requires explicit concrete types. Continue
with the [types](../language/types.md), [functions](../language/functions.md),
and [control flow](../language/control-flow.md) references.
