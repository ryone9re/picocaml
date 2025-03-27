# Specification

## 言語仕様

### 文法

```bnf
<program> ::= <expression>

<expression> ::= <integer_literal>
         | <bool_literal>
         | <identifier>
         | <expression> <op> <expression>
         | "if" <expression> "then" <expression> "else" <expression>
         | "let" <identifier> "=" <expression> "in" <expression>
         | "fun" <identifier> "->" <expression>
         | <expression> <expression>
         | "let" "rec" <identifier> "=" "fun" <identifier> "->" <expression> "in" <expression>
         | <nil>
         | <expression> "::" <expression>
         | "match" <expression> "with" <nil> "->" <expression> "|" <identifier> "::" <identifier> "->" <expression>
         | "(" <expression> ")"

<integer_literal> ::= ["-"] <digit> {<digit>}
<digit> ::= "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9"

<bool_literal> ::= "true" | "false"

<identifier> ::= <lowercase> {<letter> | <digit> | "_"}
<letter> ::= <lowercase> | <uppercase>
<lowercase> ::= "a" | "b" | ... | "z"
<uppercase> ::= "A" | "B" | ... | "Z"

<op> ::= "+" | "-" | "*" | "<"

<nil> ::= "[]"
```

### 型

- 型は以下のBNFによって示される
  - b = 基底型, t ∈ Typesとする

```bnf
τ ::= b | t | t -> t | t list
```

- データ構造はすべてCons Listや！それ以外ありまへん
  - `getName = car cons`, `getAddress car (cdr cons)` で取れるはずや

### 構造

環境、型環境の組を構造とする

- 環境
  - 宣言されたやつを `<変数名> = 値` という感じで持つ
- 型環境
  - `<変数名> : 型` という感じで持つ
  - 環境で持ってるものと現在の文脈にある型情報を持つ
  - 型スキーマで使用されている多相型の一覧とそれの対応付けも持つ

### 簡約

- 最外最左簡約（call-by-name）を行う

## REPL仕様

### 初期状態

- REPL実行時の状態をglobal環境として持つ
  - 型環境も合わせて持つ

### 評価

- 評価前に型の判定を行い、型判定でエラーが出たら評価を行わずエラーを表示する
- AST舐めながら適宜環境から引っ張ってきて評価する
