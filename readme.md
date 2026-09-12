expression := or_expression

or_expression := and_expression ("OR" and_expression)\*

and_expression := primary ("AND" primary)\*

primary := comparison
| "(" expression ")"

comparison := column comparison_operator comparison_value
| column "IN" in_list

comparison_operator := "="
| "<>"
| "<"
| "<="
| ">"
| ">="

comparison_value := string
| integer
| column

in_list := "(" string ("," string)\* ")"

column := identifier "." identifier

identifier := [A-Za-z\_][A-Za-z0-9_]\*

integer := [0-9]+

string := "'" characters "'"
characters := character\*
