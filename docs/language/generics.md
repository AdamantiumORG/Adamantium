# Generics

Functions and classes may declare type parameters:

```adamantium
fun identity<T>(value:T) result:T { result=value; }
var number=identity<int>(10);
```

Trait constraints use `T:TraitName`. Generic instances are validated and specialized before type checking and code generation.
