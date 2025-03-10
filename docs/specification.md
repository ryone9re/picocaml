# Specification

## 言語仕様

### 文法

```bnf
<program> ::= <expr>

<expr> ::= <int_literal>
         | <bool_literal>
         | <identifier>
         | <expr> <op> <expr>
         | "if" <expr> "then" <expr> "else" <expr>
         | "let" <identifier> "=" <expr> "in" <expr>
         | "fun" <identifier> "->" <expr>
         | <expr> <expr>
         | "let" "rec" <identifier> "=" "fun" <identifier> "->" <expr> "in" <expr>
         | <nil>
         | <expr> "::" <expr>
         | "match" <expr> "with" <nil> "->" <expr> ["|" <identifier> "::" <identifier> "->" <expr>]
         | "(" <expr> ")"

<int_literal> ::= ["-"] <digit> {<digit>}
<digit> ::= "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9"

<bool_literal> ::= "true" | "false"

<identifier> ::= <lowercase> {<letter> | <digit> | "_"}
<letter> ::= <lowercase> | <uppercase>
<lowercase> ::= "a" | "b" | ... | "z"
<uppercase> ::= "A" | "B" | ... | "Z"

<op> ::= "+" | "-" | "*" | "<"

<nil> ::= "[]"
```

### 構造

環境、型環境の組を構造とする

- 環境
  - 宣言されたやつを `<変数名> = AST` という感じで持つ
- 型環境
  - `<変数名> : 型` という感じで持つ
  - 環境で持ってるものと現在の文脈にある型情報を持つ
    - `enum t | t -> t | t list` t ∈ Types
  - 型スキーマで使用されている多相型の一覧とそれの対応付けも持つ

### データ型

- データ構造はすべてCons Listや！それ以外ありまへん
  - `getName = car cons`, `getAddress car (cdr cons)` で取れるはずや

## REPL仕様

### 初期状態

- REPL実行時の状態をglobal環境として持つ
  - 型環境も合わせて持つ

### 評価

- 評価前に型の判定を行い、型環境でエラーが出たら評価をキャンセルしてエラーを表示する
- AST舐めながら適宜環境から引っ張ってきて評価する
