# Adamantium memory-safety model

This document defines the memory rules implemented by the current compiler and
runtime. The model intentionally excludes raw pointers and manual allocation.

## Ownership

Every local variable owns its value. Assigning ordinary values creates a value
copy. Class assignments copy their outer runtime storage. List assignments
recursively copy nested Lists and copy class elements according to class value
semantics. Immutable
string data can be shared because Adamantium does not permit mutation of string
storage.

Runtime allocations currently remain alive until process shutdown. `remove`
invalidates a source-level name and runs lifecycle behavior, but it does not
call a raw memory deallocator. This makes double-free and physical
use-after-free impossible in the current runtime. Runtime reclamation may be
added later only if it preserves these language rules.

## Aliases

`as_variable` creates another name for the same local storage slot. Aliases are
non-owning references and cannot cross a function boundary. Writes through any
name are visible through every alias.

Removing one name leaves the slot alive while another alias exists. The
`__remove__` hook runs only when the last name for that slot is removed. A
removed name cannot be read, written, removed again, or used to create another
reference. Scalar aliases may be disconnected to create an independent copy.

### Alias identity and synchronization

Each local binding has its own name and points to one storage slot. A normal
variable is a root binding. `source.as_variable` creates an alias whose parent
is the storage node used by `source` at that moment and whose root is that
node's root. An alias chain does not create extra runtime copies while it is
synchronized. Detaching or redirecting the source name later does not silently
retarget aliases that were previously created from it.

The alias API has these guarantees:

* `get_parent()` and `alias_of()` read the current value in the direct parent's
  slot. Calling either method on a root is a compile error.
* `get_root()` reads the current value in the root slot. On a root it reads the
  variable itself.
* `is_alias()` is true for synchronized and temporarily detached aliases. It is
  false for roots and permanently disconnected values.
* `is_synced()` is true only while writes use the same slot as the parent.
* `alias_count()` returns the number of other names currently synchronized to
  the same slot.
* `detach()` and `desync()` copy the current value into private storage while
  retaining the parent relationship. Writes then affect only that alias.
* `change_only(value)` detaches a synchronized alias and assigns `value`. On an
  already detached alias it changes the existing private value.
* `sync()` discards the private value and reconnects to the remembered parent.
* `reattach()` is equivalent to `sync()`. `reattach(target)` redirects the alias
  to `target` and adopts the target's root.
* `disconnect()` and the compatibility spelling `disconect` copy the current
  value and permanently remove all parent and root relationships.
* `changename(new_name)` moves the binding to an unused identifier. It preserves
  the slot, value, type, mutability, parent, root, and synchronization state.

Removing one synchronized name does not remove shared storage. A detached alias
keeps its remembered parent slot alive so it can synchronize again. Removing
the last reachable name ends the source-level lifetime. Redirecting or
reattaching an alias does not mutate either the old or new target.

Synchronization management applies to scalar value aliases. Function, enum,
and class symbol aliases can be removed or redirected to another symbol, but
cannot be detached or disconnected as runtime values. `changename` applies to
value bindings.

## Offsets

An offset is a typed, non-owning reference to a local variable's storage slot.
It is created with `.offset` or `.get_offset()` and read with `.by_offset` or
`.value_by_offset`. Reading creates a value copy and preserves the target type.

Offsets cannot appear in function signatures, class fields, Lists, or printed
output. They therefore cannot leave the stack frame containing their target.
The compiler tracks the target slot and rejects dereferencing it after the last
name for that slot has been removed. Raw numeric addresses are never exposed.

## Objects and lifecycle hooks

Class values use runtime-managed storage that remains valid until shutdown.
`__new__` runs after field initialization, `__change__` runs after a successful
field write, and `__remove__` runs once when the final name is removed.

The compiler rejects direct field mutation from `__change__` and recursive
removal from `__remove__`. This prevents immediate recursive lifecycle calls.
Lifecycle methods cannot manually free storage or access raw addresses.

## Diagnostics

Memory-safety violations are compile errors with source locations. These
include access through removed names, invalid alias disconnection, escaped or
printed offsets, offsets stored in containers, dereferencing removed targets,
recursive lifecycle operations, and invalid field or method access.
