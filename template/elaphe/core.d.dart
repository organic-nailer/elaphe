/// create slice object
external dynamic sl([int? start, int? end, int? step]);

// built-in functions in python 3.9
external abs(x);
external bool all(iterable);
external bool any(iterable);
external String ascii(object);
external String bin(int x);
external bool callable(object);
external String chr(int i);
external compile(source, filename, mode, {flags, dont_inherit, optimize});
external delattr(object, String name);
external List<String> dir(object);
external divmod(a, b);
external enumerate(iterable, {int start});
external eval(String expression, [globals, locals]);
external exec(object, [globals, locals]);
external filter(Function function, iterable);
external format(value, [format_spec]);
external getattr(object, String name);
external globals();
external bool hasattr(object, String name);
external int hash(object);
external help([object]);
external String hex(x);
external int id(object);
external input([prompt]);
external isinstance(object, classinfo);
external issubclass(cls, classinfo);
external iter(object, [sentinel]);
external int len(s);
external locals();
external map(Function function, iterable);
external max(iterable, [key, def]);
external min(iterable, [key, def]);
external next(iterator, [def]);
external String oct(x);
external open(file, {mode, buffering, encoding, errors, newline, closefd, opener});
external int ord(String c);
external pow(base, exp, [mod]);
external print(object, {sep, end, file, flush});
external String repr(object);
external reversed(seq);
external round(number, [ndigits]);
external setattr(object, String name, value);
external sorted(iterable, {key, reverse});
external sum(iterable, {int start});
// external super([type, object_or_type]);
external vars([object]);
external zip(iter1, [iter2, iter3, iter4, iter5]);
