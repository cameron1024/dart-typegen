# Usage

The following sections explain how to use `dart-typegen`.

The basics are quite straightforward. Imagine you have the following directory
structure:
```
types:
- foo.kdl
- bar.kdl
```
Running `dart-typegen generate ./types` will, by default, result in the
following output:
```
types:
- foo.kdl
- foo.dart
- bar.kdl
- bar.dart
```
If you want `foo.dart` to be generated in a different location (with a
different filename perhaps), you can add the following section to `foo.kdl`:
```kdl
output {
  path "../relative/or/absolute/path/to/output.dart"
}
```

