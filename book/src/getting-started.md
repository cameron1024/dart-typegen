# Getting Started

`dart-typegen` is a tool that generates Dart types.

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

## A simple class definition

Here's a very simple class definition in a file called `user.kdl`:

```kdl
class "User" {
  field "id" type="String"
  field "name" type="String"
}
```

Now, run `dart-typegen generate --input user.kdl`, and you should see the
following printed in your terminal.

```dart
final class User {
  final String id;
  final String name;

  const User({required this.id, required this.name});
  
  // equals, hashCode, toJson, fromJson

  UserBuilder toBuilder() => UserBuilder(id: id, name: name);
}

final class UserBuilder {
  String id;
  String name;

  UserBuilder({required this.id, required this.name});

  User build() => User(id: id, name: name);
}
```

Alternatively, if you want it written to a file, you can run `dart-typegen
generate --input user.kdl --output user.dart`, which will write the output to
`user.dart`.
