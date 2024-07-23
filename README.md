# SModel

SModel (Semantic Modeling) for Rust provides a friendly way to describe semantic symbols of a language using dynamic dispatches and hierarchy definitions using an arena that allows for circular references.

## Definition order

Definition order is sensitive. Define subtypes after their inherited data types while using the `struct` keyword.

If you define `struct`s in any order, you may get a *not found* error that terminates the macro.

## Example

The basemost data type is the one that comes first. You may name it according to your tastes. You may usually call it *symbol* or *thingy* (according to a Microsoft Roslyn's engineer, *symbol* ought to be called *thingy*).

```rust
use hydroperfox_smodel::smodel;

smodel! {
    type Arena = Arena;

    struct Thingy {
        let x: f64 = 0.0;
        let ref y: String = "".into();

        pub fn Thingy() {
            super();
            println!("{}", self.m());
        }

        pub fn m(&self) -> String {
            "".into()
        }

        pub fn m1(&self) {
            println!("base");
        }
    }

    struct Foo: Thingy {
        pub fn Foo() {
            super();
        }

        pub override fn m(&self) -> String {
            "Foo".into()
        }

        pub override fn m1(&self) {
            if true {
                super.m1();
            }
        }
    }
}

fn main() {
    let arena = Arena::new();
    let thingy = Foo::new(&arena);
    println!("{}", thingy.m());
}
```

## Arena

The arena's name is defined as the right-hand side of the first `type Arena = ArenaName1;` directive.

## Fields

A field (a `let` declaration) has an optional `ref` modifier indicating whether to use `RefCell` or `Cell`. For all, types are either cloned or copied on read. Use `ref` for heap-allocated resources such as `String`.

Fields have a pair of a getter (`fieldname()`) and a setter (`set_fieldname(value)`).

For mutable hash maps or vectors, it is recommended to use a *shared container* (see below) that is cloned by reference and not by content.

Fields are always internal to the enclosing module, therefore there are no attributes; the field definition always starts with the `let` keyword, without a RustDoc comment.

It is recommended for fields to always start with either a underscore `_` or a private prefix such as `m_`, and consequently using accesses such as `_x()` and `set__x(v)`, or `m_x()` and `set_m_x()`, respectively.

Then, you would implement methods that may be overriden by subtypes in a base type, allowing for an *unified* data type that supports methods that operate on more than one variant.

## Shared containers

This crate provides two container data types that are cloned by reference, `SharedArray` and `SharedMap`, as well as `shared_array!` and `shared_map!` literals.

* `SharedArray` is a mutable vector managed by reference counting.
* `SharedMap` is a mutable hash map managed by reference counting.

Refer to the crate documentation for usage details.

## Constructor

The constructor is a method whose name matches the data type's name. The `arena` parameter is implicitly prepended to the formal parameter list.

The constructor is translated to a static `new` method.

The constructor contains a local `self` variable whose data type is the instance of the enclosing data type.

## Subtypes

* `symbol.is::<T>()` tests whether `symbol` is a `T` subtype.
* `symbol.to::<T>()` converts to the `T` subtype, returning `Ok(m)` or `Err`. It may be a contravariant conversion.
* `symbol.into()` is a covariant conversion.

## Super expression

The `super.f()` expression is supported by preprocessing the token sequence of a method and transforming it into another Rust code; therefore, it may be used anywhere within an instance method.

`super.f()` does a lookup in the method lists in the base data types in descending order.

## Inheriting documentation

Use the `#[inheritdoc]` attribute to inherit the RustDoc comment of an overriden method.

```rust
#[inheritdoc]
pub override fn m(&self) {
    // Action
}
```

## Method parameters

For now, the name of method parameters in overriden methods should match these of subtype methods, otherwise a macro hygiene error may occur indicating that the parameter does not exist in the subtype's method.

## License

Apache 2.0, copyright © Hydroper
