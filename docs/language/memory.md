# Memory and aliases

Normal assignments copy values. Scalar aliases created with `.as_variable` share changes until disconnected.

An offset stores the address of another variable:

```adamantium
var source=10;
var address=source.offset;
var current=address.by_offset;
```

The compiler tracks the offset's source type and rejects known dangling offsets, removed origins, and offsets that escape their valid lifetime. See [Memory safety](../MEMORY_SAFETY.md) for the current model.
