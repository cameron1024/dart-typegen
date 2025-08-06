# `dart-typegen`

This program allows generating "plain-old-data" (POD) types in Dart based on a
config file. 

## Installation

Install Rust if you need to at <https://rust-lang.org/tools/install>. Then:

```shell
cargo install dart-typegen
```

### Building from source on MacOS

Recent versions of Xcode ship with a broken `ld` which will fail to link this
binary. You can work around this by:
- installing `lld` (for example, via `nix shell nixpkgs#lld`)
- editing `.cargo/config.toml` and adding the following:
```toml
[target.aarch64-apple-darwin]
rustflags = [
  "-C", "link-arg=-fuse-ld=lld"
]
```
- running `cargo install dart-typegen` again

Once you have installed it, you can uninstall `lld` and remove the change to
`.cargo/config.toml`. But you don't have to, and you may enjoy having a faster
and less broken linker 🤷‍♂️.

### Nix

This repository is packaged as a Nix flake.

To use it, add it to your flake inputs:
```nix
{
  inputs = {
    # ...
    dart-typegen.url = "github:cameron1024/dart-typegen";
  };
}
```
Then, later in your config, use `inputs.dart-typegen.packages.${pkgs.system}.default`.

## Motivation

I created it out of frustration with the existing `build_runner`-based
solutions. Some particular issues with existing solutions that bothered me are:
- generating non-idiomatic or stylized code (e.g. `built_value`) - I want users
  of this tool to not notice they are using generated code
- performance - generating a handful of classes should not take 20+ seconds on
  a powerful laptop
- reliability - `build_runner` seems to get stuck a lot, and often requires
  cleaning, especially if switching Flutter versions often

`dart-typegen` solves these problems for me. 

It also generates builders instead of `.copyWith()`. See [below](#why-builders)
for the reason why.

It also generates to/from JSON functions, because I dream of a
`build_runner`-free life, and we already have all the information, so why not
generate it ¯\\_(ツ)_/¯.

## Usage

Create a config file `foo.kdl` (the name doesn't matter), for example:
```kdl
preamble "// ignore_for_file: some_lint"

class "Foo" {
  field "name" type="String"
  field "age" type="int" { defaults-to 123; }

  extra_dart "void printMe() => print((name, age));"
}
```
Then, run the command:
```shell
dart-typegen generate -i foo.kdl -o foo.dart
```
This will generate the following Dart in `foo.dart`:
```dart
// ignore_for_file: some_lint
import "package:equatable/equatable.dart";

final class Foo with EquatableMixin {
  final String name;
  final int age;

  const Foo({required this.name, this.age = 123});

  @override
  List<Object?> get props => [name, age];

  FooBuilder toBuilder() => FooBuilder(name: name, age: age);

  Map<String, dynamic> toJson() => {"name": name, "age": age};
  factory Foo.fromJson(Map<String, dynamic> json) =>
      Foo(name: json["name"] as String, age: json["age"] as int);

  void printMe() => print((name, age));
}

final class FooBuilder {
  String name;
  int age;

  FooBuilder({required this.name, required this.age});

  Foo build() => Foo(name: name, age: age);
}
```

### What's that config file format?

It's [KDL][kdl]. It's pretty neat.

Most syntax is supported except multi-line strings. But raw strings work, so
it's not a big deal:

```kdl
// instead of:
docs """
  My docs that are very
  long and need multiple
  lines
"""

// write:
docs r"
  My docs that are very
  long and need multiple
  lines
"

/- btw {
  isnt "this"
  comment "syntax"
  pretty "neat"
}
```

### Unions (a.k.a. enums, sum types, sealed classes, etc.)

Dart doesn't have a convenient way to express that a type may be one of many
possible variants (comparable to Rust's `enum`s or TypeScript's union types).

The closest approximation we have are abstract/sealed classes with a subclass
per variant.

This is not ideal for a few reasons:
- `json_serializable`, the de-facto standard JSON library for Dart, still has
  not implemented support for sealed classes, despite a [two year old
  issue][json sealed]. This requires manually stitching together the parts that
  `json_serializable` *can* generate
- it's a pretty large amount of boilerplate

So let's generate them.

To generate "unions", declare them in your config file:
```kdl
// It wouldn't be an OO example without some animals...
union "Animal" {
  class "Dog" {
    field "breed" type="String";
  }

  class "Cat" {
    field "color" type="int";
  }
}
```
This will generate:
- an abstract class `Animal`
- subclasses `Dog` and `Cat` with value equality, builders, and JSON conversions
- specialized JSON code in `Animal` which encodes the type in a `"type"` field
  in the resulting JSON

#### Why not a sealed class?

In a library, `sealed` classes can pose a semver hazard. When you publish a
sealed class, users may write code that fails to compile if new variants are
added. However, often, library authors want to add new variants to a union
after publishing, without bumping the major version.

If you are confident you will never need to add a new variant (without a
breaking change), you can opt into using sealed classes with:
```kdl
union "MyUnion" sealed=true {
  // ...
}
```

### Default values

Fields can be given default values:
```kdl
class "Foo" {
  field "bar" type="String" {
    defaults-to "stuff"
  }
}
```
The generated constructor will now contain `this.bar = "stuff"` instead of
`required this.bar`. Fields without defaults are always `required`. 
If you have a nullable field that you would like to not be `required`, simply
assign it `defaults-to null`.

The value provided is interpreted as a KDL scalar value and converted directly
to Dart. However, this is not able to express certain Dart values (such as
collection literals, identifiers, etc.). For these cases, the
`defaults-to-dart` argument can be used instead. It takes a single string which
is interpreted as Dart code:
```kdl
class "Foo" {
  field "bar" type="List<int>" {
    defaults-to-dart "const [1, 2, 3]"
  }
}
```
It is an error to have both `defaults-to` and `defaults-to-dart` on the same
field.

### Docs

Most entities have a `docs` property. This will be converted to a standard Dart
`///` doc comment.

## Why builders?

When creating immutable classes, it's important to have some way to create a
new object with some fields changed. A common approach is to have a
`.copyWith()` method that takes optional parameters for each field. For
example: 

```dart
class Dog {
  final String name;
  final int age;

  const Dog({required this.name, required this.age});

  Dog copyWith({String name?, int? age}) => Dog(
    name: name ?? this.name,
    age: age ?? this.age,
  );
}
```

This allows users to call `dog.copyWith(name: "new name")`, and the old age
will be preserved in the new object.

This is simple and convenient, but it has two major issues, both of which are
solved by using builders.

### Nullable fields

This pattern does not allow setting a field to `null` if it was already set to
a non-null value. For example, consider a slight modification to the previous
example so that `name` is now nullable:

```dart
class Dog {
  final String? name;
  final int age;

  const Dog({required this.name, required this.age});

  Dog copyWith({String name?, int? age}) => Dog(
    name: name ?? this.name,
    age: age ?? this.age,
  );
}
```
Now consider the following code:
```dart
final dog = Dog(name: "frank", age: 12);
final dogWithoutName = dog.copyWith(name: null);

print(dogWithoutName.name);  // prints "frank"
```
This is because Dart doesn't have a way to distinguish between an optional
argument that is omitted, and one which is explicitly provided the default
value.

You can work around this in some cases if you can extend the type of the
parameter, but many common Dart types cannot be subtyped (`int`, `String`,
etc.) so this solution isn't applicable in the general case.

### Deeply nested fields

If you have a complex object hierarchy and you want to update a deeply nested
field, `.copyWith()` makes this tedious:
```dart
final newFoo = foo.copyWith(
  bar: foo.bar.copyWith(
    baz: foo.bar.baz.copyWith(
      qux: "new value",
    ),
  ),
);
```
If we were dealing with mutable fields, we could just write:
```dart
foo.bar.baz.qux = "new value";
```
We're giving up a lot of ergonomics in exchange for immutability.

### Builders to the rescue!

With builders, this gets reduced to:
```dart
final builder = foo.builder()
  ..bar.baz.qux = "new value";
final newFoo = builder.build();
```
Not perfect, but it's much better.

Importantly, the boilerplate doesn't scale with the depth of the nesting.

## Should I use this?

It depends. It has several advantages over more standard tooling (performance,
reliability, etc.), but also some downsides:
- it's much less configurable - I've added support for features I personally
  need, and very little else
- it doesn't have access to type information - if you need to be able to
  customize code generation based on the types of fields, use `build_runner`
- less community support

This tool is very much influenced by problems I encountered at work, where I
work on a library, not an application. This means that many of the tradeoffs I
have chosen are oriented towards library development, rather than application
development. For example, I cannot require users of my code to be familiar with
`freezed`, `built_value`, or any other library. Keep this in mind when
evaluating this tool.

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

[kdl]: https://kdl.dev
[json sealed]: https://github.com/google/json_serializable.dart/issues/1342
