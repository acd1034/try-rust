```hs
-- https://github.com/aamine/cbc/blob/f339ad28bc826ddcaabfdafa3a84bc539a80f4d1/net/loveruby/cflat/parser/Parser.jj
-- clang-format off
compilation_unit = import_stmts top_defs <EOF>
import_stmts = import_stmt*
import_stmt = name ("." name)* ";"
name = <IDENT>
top_defs = top_def*
top_def = LOOKAHEAD(storage typeref <IDENT> "(") defun
        | LOOKAHEAD(3) defvars
        | defconst
        | defstruct
        | defunction
        | typedef
defvars = storage type name ("=" expr)? ("," name ("=" expr)?)* ";"
storage = <STATIC>?
defconst = <CONST> type name "=" expr ";"
defun = storage typeref name "(" params ")" block
params = LOOKAHEAD(<VOID> ")") <VOID>
       | fixedparams ("," "...")?
fixedparams = param (LOOKAHEAD(2) "," param)*
param = type name
block = "{" defvar_list stmts "}"
defstruct = <STRUCT> name member_list ";"
defunion = <UNION> name member_list ";"
member_list = "{" (slot ";")* "}"
slot = type name
typedef = <TYPEDEF> typeref <IDENT> ";"
type = typeref
typeref = typeref_base
        (LOOKAHEAD(2) "[" "]" -- 長さが指定されていない配列、`int[] x`
        | "[" <INTEGER> "]"   -- 長さが指定されている配列、`int[5] x`
        | "*"
        | "(" param_typerefs ")"
        )*
typeref_base = <VOID>
             | <CHAR>
             | <SHORT>
             | <INT>
             | <LONG>
             | LOOKAHEAD(2) <UNSIGNED> <CHAR>
             | LOOKAHEAD(2) <UNSIGNED> <SHORT>
             | LOOKAHEAD(2) <UNSIGNED> <INT>
             | <UNSIGNED> <LONG>
             | <STRUCT> <INDENT>
             | <UNION> <IDENT>
             | LOOKAHEAD({isType(getToken(1).image)}) <IDENT>

stmts = stmt*
stmt =
     ( ";"
     | LOOKAHEAD(2) labeled_stmt
     | expr ";"
     | block
     | if_stmt
     | while_stmt
     | dowhile_stmt
     | for_stmt
     | switch_stmt
     | break_stmt
     | continue_stmt
     | goto_stmt
     | return_stmt
     )
if_stmt = <IF> "(" expr ")" stmt (LOOKAHEAD(1) <ELSE> stmt)?
while_stmt = <WHILE> "(" expr ")" stmt
dowhile_stmt
for_stmt = <FOR> "(" expr? ";" expr? ";" expr? ")" stmt
switch_stmt
break_stmt = <BREAK> ";"
continue_stmt = <CONTINUE> ";"
goto_stmt
return_stmt = LOOKAHEAD(2) <RETURN> ";"
            | <RETURN> expr ";"

expr = LOOKAHEAD(term "=") term "=" expr
     | LOOKAHEAD(term opassign) term opassign expr
     | expr10
opassign =
         ( "+="
         | "-"
         | "*"
         | "/"
         | "%"
         | "&"
         | "|"
         | "^"
         | "<<="
         | ">>="
         )
expr10 = expr9 ("?" expr ":" expr10)
expr9 ||
expr8 &&
expr7 > < >= <= == !=
expr6 |
expr5 ^
expr4 &
expr3 >> <<
expr2 + -
expr1 * / %
term = LOOKAHEAD("(" type) "(" type ")" term
     | unary
unary = "++" unary
      | "--" unary
      | "+" term
      | "-" term
      | "!" term
      | "~" term
      | "*" term
      | "&" term
      | LOOKAHEAD(3) <SIZEOF> "(" type ")" -- sizeof(型)
      | <SIZEOF> unary                     -- sizeof 式
      | postfix
postfix = primary
        ( "++"
        | "--"
        | "[" expr "]"
        | "." name
        | "->" name
        | "(" args ")"
        )*
args = (expr ("," expr)*)?
primary = <INTEGER>
        | CHARACTER
        | STRING
        | INDENT
        | "(" expr ")"
```
