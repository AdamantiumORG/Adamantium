# Decorators

Decorators execute reusable functions when another function or method starts.
The decorator function is type checked like an ordinary call.

```adamantium
fun log() result:None {
    print.newline("called");
}

#[log]
fun work() result:None {
    print.newline("working");
}
```

`work()` prints `called` before `working`. Multiple decorators execute from top
to bottom:

```adamantium
#[authenticate]
#[log]
fun update() result:None {}
```

## Function arguments

A decorator argument names a zero-argument function. Adamantium evaluates that
function when the decorated function starts and passes its result to the
decorator:

```adamantium
fun current_user() result:int { result=7; }
fun audit(user:int) result:None { print.newline(user); }

#[audit(current_user)]
fun save() result:None {}
```

Normal call checking validates the callback result against the decorator
parameter. Unknown functions, incorrect arity, and incompatible types are
compile-time errors.

## Classes and exclusions

A class decorator applies to its constructor and every method. A method can
exclude an inherited decorator with `#[!name]`:

```adamantium
#[log]
class Worker() {
    fun __new__() {}
    pub fun run() result:None {}

    #[!log]
    pub fun quiet() result:None {}
}
```

Exclusions are valid only on methods and cannot have arguments. A method cannot
add a decorator it already inherits from its class.

Decorators do not replace or wrap return values. They execute before the
decorated body. Lifecycle hooks follow the same rule as other class methods.
