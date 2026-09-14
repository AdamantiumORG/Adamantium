# Memory and aliases

Normal assignments copy values. Scalar aliases created with `.as_variable` share changes until disconnected.

Use `alias.detach()` or `alias.desync()` for a temporary private copy,
`alias.sync()` to reconnect to its remembered parent, and
`alias.reattach(other)` to redirect it. `alias.change_only(value)` changes only
the alias. `alias.disconnect()` makes the copy permanently independent.

Aliases expose `get_parent()`, `get_root()`, `alias_of()`, `is_alias()`,
`is_synced()`, and `alias_count()`. A binding can be renamed without moving its
storage with `old_name.changename(new_name)`. The complete guarantees are in
[Memory safety](../MEMORY_SAFETY.md#alias-identity-and-synchronization).

An offset stores the address of another variable:

```adamantium
var source=10;
var address=source.offset;
var current=address.by_offset;
```

The compiler tracks the offset's source type and rejects known dangling offsets, removed origins, and offsets that escape their valid lifetime. See [Memory safety](../MEMORY_SAFETY.md) for the current model.
