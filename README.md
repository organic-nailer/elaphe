# Elaphe 

![](assets/logo_wide.png)

ElapheはDartのプログラムをPython VM上で動くバイトコードに変換するコンパイラです。Dartの文法や静的な型チェックを使いながらPythonの豊富なライブラリを利用することができます。

![](assets/overview.png)

# Install

現状Linuxのみサポートしています。GitHubからソースコードをダウンロードし、手元でビルドしてください。

- `git clone https://github.com/organic-nailer/elaphe.git`
- `cd ./elaphe`
- `cargo build --release`
- `mkdir -p ~/.elaphe/bin && cp -r ./target/release/elaphe ~/.elaphe/bin && cp -r ./target/release/template ~/.elaphe/bin && cp -r ./target/release/script ~/.elaphe/bin`
- `echo 'export PATH="$PATH:$HOME/.elaphe/bin"' >> ~/.bashrc`
- `source ~/.bashrc`

# Getting Started

Python3.9のみに対応しています。

```
$ elaphe init foo
$ cd foo
foo$ python -V
Python 3.9.15
foo$ elaphe run main.dart
```

# Example

[Anaconda](https://www.anaconda.com/products/distribution)/[Miniconda](https://docs.conda.io/en/latest/miniconda.html)でPython3.9の環境を作ります。`conda`コマンドは事前に使えるようにしてください。

```
$ conda create -n elaphe_env python=3.9
$ conda activate elaphe_env
$ python -V
Python 3.9.15
$ conda install numpy matplotlib
```

`elaphe init`でプロジェクトを作成し、利用するライブラリを`elaphe add`で追加します。

```
$ elaphe init elaphe_example
$ cd elaphe_example
elaphe_example$ elaphe add numpy
elaphe_example$ elaphe add matplotlib
```

`main.dart`を以下のように書き換えます。

```dart
import 'elaphe/numpy.d.dart' as np;
import 'elaphe/matplotlib/pyplot.d.dart' as plt;

void main() {
  final x = np.arange(0, 10, 0.1);
  final y = np.sin(x);
  plt.plot(x, y);
  plt.show();
}
```

`elaphe run`でコンパイル、実行します。

```
elaphe_example$ elaphe run main.dart
```

![](assets/example.png)

# Commands

## elaphe init

```
elaphe init <directory>
```

指定されたディレクトリにプロジェクトを作成します。

## elaphe add

```
elaphe add <python module>
```

指定したPythonモジュールの型定義ファイルをプロジェクトに生成します。Pythonモジュールは現在のPython環境に存在する必要があります。

## elaphe build

```
elaphe build <target dart file>
```

指定したDartファイルをコンパイルし、`main.pyc`ファイルを生成します。このファイルは`python main.pyc`で実行できます。

## elaphe run

```
elaphe run <target dart file>

elaphe run -c <dart code>
```

指定したDartファイルをコンパイルし、実行します。オプション`-c`が指定された場合、後の文字列をDartプログラムと解釈し実行します。

# elaphe/core

## sl()

```dart
external dynamic sl([int? start, int? end, int? step]);
```

sliceをDart上で実現するため、sl()関数を用意しています。

# Limitation

## 対応するPython VM

生成するバイトコードの関係でPython3.9のみ対応しています。他のバージョンについては今後対応予定です。

## コンパイル対象

単一ファイルのみ(main.dart)のコンパイルに対応しています。複数ファイルのコンパイル、他のdartファイルのimportは非対応です。

## Dartライブラリ

Dartライブラリには対応していません。Flutterなどだけではなく、dart:coreやdart:mathなどの標準ライブラリも使うことができません。代わりにPythonのライブラリが利用できます。

## Dart文法

ElapheではDartの限られたサブセットのみをサポートしており、一部の文法が利用できません。順次対応文法の予定をしています。

形式的な文法の対応状況は以下のドキュメントを参照してください。赤字部分が対応済です。

https://docs.google.com/document/d/1c956nDwu3t9qNN0C4HBvl9U6WSvCUY3umqpzohqlrKs/edit?usp=sharing

- [x] Variables
- [ ] Functions
    - [ ] async keyword
    - [ ] sync keyword
    - [ ] generator
    - [ ] generics
    - [ ] covariant keyword
    - [ ] this keyword
- [ ] Classes
    - [ ] abstract
    - [ ] generics
    - [ ] superclass
    - [ ] mixin
    - [x] simple constructor
    - [ ] constructor with initializers
    - [ ] factory constructor
    - [x] method declaration
    - [ ] static keyword
    - [ ] getter/setter
    - [ ] operator
    - [x] late keyword
    - [x] instance variable declaration
    - [ ] covariant keyword
    - [ ] const keyword
    - [ ] constructor redirection
- [ ] Extensions
- [ ] Enums
- [ ] Generics
- [ ] Metadata
- [ ] Expressions
    - [x] Assignment Expression
    - [x] Expression List
    - [ ] Primary
        - [x] this
        - [ ] function
        - [x] null
        - [x] bool
        - [x] numeric
        - [ ] String
            - [x] Single Quote
            - [x] Double Quote
            - [ ] Multiline
            - [ ] format
            - [ ] Escape
        - [ ] List
            - [x] Normal
            - [ ] Spread
        - [ ] Set/Map
            - [x] Normal
            - [ ] if
            - [ ] for
    - [x] Throw
    - [ ] new keyword
    - [ ] const keyword
    - [ ] Cascade
    - [x] Conditional
    - [x] IfNull
    - [x] Logical operators
    - [x] Equality operators
    - [x] Relational operators
    - [x] Bitwise operators
    - [x] Shift operators
        - Not Support `>>>` and `>>>=`
    - [x] Additive/Multiplicative operators
    - [x] Unary operators
    - [ ] await keyword
    - [x] Increment/Decrement
    - [ ] Selector
        - [ ] !
        - [ ] ?.xxx
        - [ ] ?[]
        - [x] .xxx
        - [x] .[]
        - [x] Arguments
    - [x] Type Cast
    - [x] Type Test
- [ ] Statements
    - [x] Label
    - [x] Block
    - [x] Local Variable Declaration
    - [ ] Local Function Declaration
    - [x] for
    - [ ] await for
    - [ ] for in
    - [x] while
    - [x] do
    - [ ] switch
        - [x] case
        - [x] default
        - [ ] label
    - [x] if
    - [x] rethrow
    - [x] try
    - [x] break
    - [x] continue
    - [x] return
    - [ ] yield
    - [ ] yield*
    - [ ] Expression Statement
    - [ ] assert
- [ ] Libraries and Scripts
    - [ ] part keyword
    - [ ] export keyword
    - [ ] import keyword
        - [x] Normal
        - [x] as
        - [ ] show/hide
- [ ] Static Types
    - [x] void
    - [ ] function type
    - [x] Identifier
    - [ ] Identifier.Identifier
- [ ] Other

# Supplementary Information

> Elaphe logo is designed by [Bing Image Creator](https://www.bing.com/create)
> 
> The font used in Elaphe's logo is [Confortaa](https://fonts.google.com/specimen/Comfortaa/)
