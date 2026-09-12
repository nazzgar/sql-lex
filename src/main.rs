use sql_lex::{lex, parse_where};

fn main() {
    for where_clause in [
        "users.name = 'Ada'",
        "users.role = 'admin' AND users.active <> 'yes' and users.some_value > 123 and users.some_value2 < 123 and users.some_value2 <= 123 and users.some_value2 >= 123",
        "users.role IN ('aba', 'ddsadas')",
    ] {
        println!("WHERE {where_clause}");
        println!("tokens: {:#?}", lex(where_clause));
        println!("AST: {:#?}\n", parse_where(where_clause));
    }
}
