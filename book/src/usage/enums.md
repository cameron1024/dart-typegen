# Generating Enums

Compared to classes, enums are quite straightforward. An enum can be defined as follows:
```kdl
enum "Role" {
  variant "admin"
  variant "owner"
  variant "user"
}
```

There's not many customization options for enums since they're so simple.

Like classes, they can have `extra-dart` and `annotations`. 

