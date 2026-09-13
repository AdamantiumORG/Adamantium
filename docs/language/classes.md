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
