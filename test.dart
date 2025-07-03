/// Example docs for the class Foo
///
/// Here are some docs
final class Foo with EquatableMixin {
  final String foo;
  final Bar bar;

  const Foo({required this.foo, required this.bar});

  @override
  List<Object?> get props => [foo, bar];

  FooBuilder toBuilder() => FooBuilder(foo: foo, bar: bar.toBuilder());
}

final class FooBuilder {
  String foo;
  BarBuilder bar;

  FooBuilder({required this.foo, required this.bar});

  Foo build() => Foo(foo: foo, bar: bar.build());
}

final class Bar with EquatableMixin {
  final int hello;

  const Bar({required this.hello});

  @override
  List<Object?> get props => [hello];

  BarBuilder toBuilder() => BarBuilder(hello: hello);
}

final class BarBuilder {
  int hello;

  BarBuilder({required this.hello});

  Bar build() => Bar(hello: hello);
}
