# Enums

Enums define a fixed set of variants:

```adamantium
enum State { ready, stopped }
var state=State.ready;
```

Enums may be matched with `match`. Add `pub` when an enum must be available from another module.
