# Enums

Enums define a fixed set of variants:

```adamantium
enum State { ready, stopped }
var state=State.ready;
```

Enums may be matched with `match`. Add `pub` when an enum must be available from another module.

## Exhaustive matching

Every enum `match` must handle every variant:

```adamantium
enum State { ready, stopped }

fun show(state:State) result:None {
    match state {
        State.ready => { print.newline("ready"); }
        State.stopped => { print.newline("stopped"); }
    }
}
```

The `_` branch handles every value not matched by an earlier branch, so it can be used instead of listing all remaining variants:

```adamantium
match state {
    State.ready => { print.newline("ready"); }
    _ => { print.newline("not ready"); }
}
```

`_` must be the final branch. A branch after `_`, or an `_` branch after all enum variants have already been handled, is unreachable and produces a compiler error. For a non-exhaustive enum match, the diagnostic lists the missing variants. Matches on non-enum values do not require an `_` branch because their possible values are not finite.
