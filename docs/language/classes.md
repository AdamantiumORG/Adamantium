# Classes

Classes declare typed fields and methods. `__new__` runs after construction:

```adamantium
class Counter(pub value:int) {
    fun __new__() {}
    pub fun current() result:int { result=self.value; }
}

fun main() {
    var counter=Counter(value=1);
    print.newline(counter.current());
}
```

Fields and methods are private unless marked `pub`. Lifecycle hooks include `__new__`, `__change__`, and `__remove__`.

## Copying nested class values

Assignment copies ordinary class values, but complete recursive deep-copy
semantics for fields that themselves contain class values are not yet part of
the stable language contract. Code must not rely on nested class graphs being
recursively duplicated. This restriction avoids silently choosing identity,
cycle, alias, and lifecycle-hook behavior before those rules are specified.
Use scalar fields or construct an explicit independent nested object when copy
isolation matters. The compiler will gain deep copying only together with
defined cycle handling and `__new__`, `__change__`, and `__remove__` ordering.
