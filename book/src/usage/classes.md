# Generating Classes

Generating classes is one of the core features of `dart-typegen`

We'll start with a simple example, and build up to something more complex.

## A simple `User` type

```kdl
// user.kdl
class "User" {
  field "id" type="int"
  field "email" type="String"
  field "name" type="String?"
}
```

This will generate a `User` class that can be used as follows:

```dart
final alice = User(id: 1, email: "alice@example.com", name: null);
final bob = User.fromJson({"id": 2, "email": "bob@example.com", "name": "bob"});

print(alice);  // prints "User(id: 1, email: alice@example.com: name: null)"

final aliceWithName = (alice.toBuilder()..name = "alice").build();
```

## Defaults

This is great, but it's annoying that we need to write `name: null` when
creating a `User` with no name.

In general, `dart-typegen` assumes fields will be `required` in the constructor
unless a default value is given.

In the case of nullable fields, `null` is often a sensible default, but we're
free to use anything, so long as the types match. Let's start with `null`:

```kdl
class "User" {
  field "id" type="int"
  field "email" type="String"
  field "name" type="String?" {
    defaults-to null
  }
}
```

If you want, we can keep that block on a single line, but you'll need a trailing semicolon if you do:

```kdl
class "User" {
  field "id" type="int"
  field "email" type="String"
  field "name" type="String?" { defaults-to null; }
}
```

Now we can write: 
```dart
final alice = User(id: 1, email: "alice@example.com");
```
Fantastic.

## Using a default defined in another file

We just remembered - there's a file in our codebase that contains all the
default values, it's called `constants.dart` and it contains a bunch of default
values that we'd like to use:

```dart
// constants.dart

const _defaultFirstName = "John";
const _defaultSecondName = "Dart";
const defaultName = "$_defaultFirstName $_defaultSecondName";
```

Let's add that to our generated file.

First, we'll need to make sure the generated file imports `constants.dart`.
To do this, we use the `preamble` section:

```kdl
preamble r#"
  import "constants.dart";
"#
```
A couple of things to note:
- the `r#" ... "#` is a KDL "raw string". It can span over multiple lines.
- the `import` line is indented. This is optional, but I find it helps keep the
  visual structure of the KDL intact. All generated files will be `dart
  format`ted before being written.

Now, we can change the default:

```kdl
class "User" {
  field "id" type="int"
  field "email" type="String"
  field "name" type="String?" { 
    defaults-to "defaultName"; 
  }
}
```

> [!WARNING]
> Uh oh! This doesn't work!

`defaults-to` will use the value provided verbatim as the default. We don't
want to use the string `"defaultName"`, we want to use the Dart identifier
`defaultName`. In other words, instead of interpreting the string as a string,
we want it interpreted as **Dart code**.

This is what `defaults-to-dart` is for:

```kdl
class "User" {
  field "id" type="int"
  field "email" type="String"
  field "name" type="String?" { 
    defaults-to-dart "defaultName"; 
  }
}
```

Fantastic!

```dart
final alice = User(id: 1, email: "alice@example.com");
print(alice.name);  // prints "John Dart"
```

To summarize the differences between `defaults-to` and `defaults-to-dart`:

| `defaults-to`                            | `defaults-to-dart`                    |
| -                                        | -                                     |
| Can be any KDL type                      | Must be a String                      |
| Is converted to an equivalent Dart value | Injected directly into generated code |
| Simple values only                       | Arbitrary Dart expressions            |


It is, perhaps unsurprisingly, an error to have both `defaults-to` and
`defaults-to-dart` on the same field.

## Adding custom code to a class

In our app, all users have a `displayName`, which is how they are represented
in a UI. Their display name is:
- their `name` if not null
- their `email` if `name` is `null`

We could create a function in a separate file like this:
```dart
String userDisplayName(User user) {
  return user.name ?? user.email;
}
```

We could even make it an extension:
```dart
extension DisplayNameExtension on User {
  String get displayName => name ?? email;
}
```
But what if we wanted it on the class itself?

That's simple! Just add an `extra-dart` directive:

```kdl
class "User" {
  field "id" type="int"
  field "email" type="String"
  field "name" type="String?" { 
    defaults-to-dart "defaultName"; 
  }

  extra-dart r#"
    String get displayName => name ?? email;
  "#
}
```
Now our generated class will have this extra getter.

Code in an `extra-dart` directive is not handled specially - it is simply
pasted into the end of the class body. This means it can contain any valid Dart
code, and it also doesn't impact things like equality or JSON serialization.

## What about custom annotations?

Sometimes you need to add annotations to your class. Maybe it's important that
this class isn't exposed publicly as part of your package's API, so you might
want to add `@internal` from `package:meta`. You can't use `extra-dart` for
this, since that adds code to the *inside* of the class.

For this, there is a dedicated `annotations` directive:

```kdl
class "User" {
  annotations "@internal"
  
  field "id" type="int"
  field "email" type="String"
  field "name" type="String?" { 
    defaults-to-dart "defaultName"; 
  }

  extra-dart r#"
    String get displayName => name ?? email;
  "#
}
```

Make sure to add the corresponding import to the `preamble`:
```kdl
preamble r#"
  import "constants.dart";
  import "package:meta/meta.dart";
"#
```

## Docs

A good library author documents their code. You are documenting your code, right?...

Most objects in `dart-typegen` have a `docs` directive. In our case, we're
going to add some docs to the class itself, as well as to a field:


```kdl
class "User" {
  annotations "@internal"
  docs r#"
Here are some docs

They can take multiple lines
  "#
  
  field "id" type="int" {
    docs "The id used in the database"
  }
  field "email" type="String"
  field "name" type="String?" { 
    defaults-to-dart "defaultName"; 
  }

  extra-dart r#"
    String get displayName => name ?? email;
  "#
}
```

That's enough to get started writing basic classes.

