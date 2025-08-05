# `dart-typegen`

This program allows generating "plain-old-data" (POD) types in Dart based on a
config file. 

## Motivation

I created it out of frustration with the existing
`build_runner`-based solutions. Some particular issues with existing solutions
that bothered me are:
- generating non-idiomatic or stylized code (e.g. `built_value`) - I want users
  of this tool to not notice they are using generated code
- performance - generating a handful of classes should not take 20+ seconds on
  a powerful laptop
- reliability - `build_runner` seems to get stuck a lot, and often requires
  cleaning, especially if switching Flutter versions often

`dart-typegen` solves these problems for me. 

It also generates builders instead of `.copyWith()`. See [below](#why-builders)
for the reason why.

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

### Unions ()


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
