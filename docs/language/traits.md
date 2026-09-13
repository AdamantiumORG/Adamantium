# Traits

Traits describe required public class methods:

```adamantium
trait Named { fun name() result:string; }

class Item(pub text:string) implements Named {
    fun __new__() {}
    pub fun name() result:string { result=self.text; }
}
```

The compiler checks that every implementation provides compatible method parameters and results.
