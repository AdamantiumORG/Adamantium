# Functions

Functions start with `fun`. Non-main functions declare a named result:

```adamantium
fun add(a:int,b:int) result:int {
    result=a+b;
}
```

The named result is returned when execution reaches the end. `return result;` returns earlier. Prefix an optional parameter with `$`. Functions are private by default and `pub` exposes them to other modules.
