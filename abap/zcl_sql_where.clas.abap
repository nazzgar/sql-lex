" Lexer, recursive-descent parser, and AST for a small SQL WHERE grammar.
" Install zcx_sql_where before this global class.
CLASS zcl_sql_where DEFINITION PUBLIC FINAL CREATE PUBLIC.
  PUBLIC SECTION.
    TYPES tt_strings TYPE STANDARD TABLE OF string WITH EMPTY KEY.

    " AST fields. node_type is COMPARISON, AND, or OR.
    DATA node_type           TYPE string READ-ONLY.
    DATA table_name          TYPE string READ-ONLY.
    DATA column_name         TYPE string READ-ONLY.
    DATA comparison_operator TYPE string READ-ONLY.
    DATA value_type          TYPE string READ-ONLY. " STRING, INTEGER, COLUMN, LIST
    DATA value               TYPE string READ-ONLY.
    DATA value_table         TYPE string READ-ONLY.
    DATA value_column        TYPE string READ-ONLY.
    DATA values              TYPE tt_strings READ-ONLY.
    DATA left                TYPE REF TO zcl_sql_where READ-ONLY.
    DATA right               TYPE REF TO zcl_sql_where READ-ONLY.

    METHODS constructor
      IMPORTING
        node_type           TYPE string OPTIONAL
        table_name          TYPE string OPTIONAL
        column_name         TYPE string OPTIONAL
        comparison_operator TYPE string OPTIONAL
        value_type          TYPE string OPTIONAL
        value               TYPE string OPTIONAL
        value_table         TYPE string OPTIONAL
        value_column        TYPE string OPTIONAL
        values              TYPE tt_strings OPTIONAL
        left                TYPE REF TO zcl_sql_where OPTIONAL
        right               TYPE REF TO zcl_sql_where OPTIONAL.

    CLASS-METHODS parse
      IMPORTING where_clause TYPE string
      RETURNING VALUE(result) TYPE REF TO zcl_sql_where
      RAISING zcx_sql_where.

  PRIVATE SECTION.
    TYPES: BEGIN OF ty_token,
             kind     TYPE string,
             text     TYPE string,
             position TYPE i,
           END OF ty_token,
           tt_tokens TYPE STANDARD TABLE OF ty_token WITH EMPTY KEY,
           BEGIN OF ty_value,
             kind        TYPE string,
             text        TYPE string,
             table_name  TYPE string,
             column_name TYPE string,
             values      TYPE tt_strings,
           END OF ty_value.

    DATA tokens TYPE tt_tokens.
    DATA index TYPE i.

    METHODS lex
      IMPORTING input TYPE string
      RETURNING VALUE(result) TYPE tt_tokens
      RAISING zcx_sql_where.
    METHODS parse_expression RETURNING VALUE(result) TYPE REF TO zcl_sql_where RAISING zcx_sql_where.
    METHODS parse_or RETURNING VALUE(result) TYPE REF TO zcl_sql_where RAISING zcx_sql_where.
    METHODS parse_and RETURNING VALUE(result) TYPE REF TO zcl_sql_where RAISING zcx_sql_where.
    METHODS parse_primary RETURNING VALUE(result) TYPE REF TO zcl_sql_where RAISING zcx_sql_where.
    METHODS parse_comparison RETURNING VALUE(result) TYPE REF TO zcl_sql_where RAISING zcx_sql_where.
    METHODS parse_value RETURNING VALUE(result) TYPE ty_value RAISING zcx_sql_where.
    METHODS parse_in_list RETURNING VALUE(result) TYPE ty_value RAISING zcx_sql_where.
    METHODS parse_operator RETURNING VALUE(result) TYPE string RAISING zcx_sql_where.
    METHODS parse_identifier RETURNING VALUE(result) TYPE string RAISING zcx_sql_where.
    METHODS expect IMPORTING kind TYPE string message TYPE string RAISING zcx_sql_where.
    METHODS matches IMPORTING kind TYPE string RETURNING VALUE(result) TYPE abap_bool.
    METHODS peek RETURNING VALUE(result) TYPE ty_token.
    METHODS advance RETURNING VALUE(result) TYPE ty_token.
    METHODS is_identifier_start IMPORTING character TYPE c LENGTH 1 RETURNING VALUE(result) TYPE abap_bool.
    METHODS is_identifier_continue IMPORTING character TYPE c LENGTH 1 RETURNING VALUE(result) TYPE abap_bool.
ENDCLASS.

CLASS zcl_sql_where IMPLEMENTATION.
  METHOD constructor.
    me->node_type = node_type.
    me->table_name = table_name.
    me->column_name = column_name.
    me->comparison_operator = comparison_operator.
    me->value_type = value_type.
    me->value = value.
    me->value_table = value_table.
    me->value_column = value_column.
    me->values = values.
    me->left = left.
    me->right = right.
  ENDMETHOD.

  METHOD parse.
    DATA(parser) = NEW zcl_sql_where( ).
    parser->tokens = parser->lex( where_clause ).
    parser->index = 1.
    result = parser->parse_expression( ).
    IF parser->peek( )-kind <> `EOF`.
      RAISE EXCEPTION TYPE zcx_sql_where
        EXPORTING position = parser->peek( )-position message = `Unexpected trailing input`.
    ENDIF.
  ENDMETHOD.

  METHOD lex.
    DATA offset TYPE i VALUE 0.
    DATA length TYPE i VALUE strlen( input ).
    WHILE offset < length.
      DATA(character) = input+offset(1).
      IF character = space OR character = cl_abap_char_utilities=>horizontal_tab OR
         character = cl_abap_char_utilities=>newline.
        offset += 1.
        CONTINUE.
      ENDIF.
      CASE character.
        WHEN `.`. APPEND VALUE #( kind = `DOT` position = offset ) TO result. offset += 1.
        WHEN `=`. APPEND VALUE #( kind = `EQ` position = offset ) TO result. offset += 1.
        WHEN `,`. APPEND VALUE #( kind = `COMMA` position = offset ) TO result. offset += 1.
        WHEN `(`. APPEND VALUE #( kind = `LEFT_PAREN` position = offset ) TO result. offset += 1.
        WHEN `)`. APPEND VALUE #( kind = `RIGHT_PAREN` position = offset ) TO result. offset += 1.
        WHEN `<`.
          DATA(next_offset_lt) = offset + 1.
          DATA(next_lt) = COND string( WHEN next_offset_lt < length THEN input+next_offset_lt(1) ELSE `` ).
          IF next_lt = `=`. APPEND VALUE #( kind = `LTE` position = offset ) TO result. offset += 2.
          ELSEIF next_lt = `>`. APPEND VALUE #( kind = `NE` position = offset ) TO result. offset += 2.
          ELSE. APPEND VALUE #( kind = `LT` position = offset ) TO result. offset += 1. ENDIF.
        WHEN `>`.
          DATA(next_offset_gt) = offset + 1.
          DATA(next_gt) = COND string( WHEN next_offset_gt < length THEN input+next_offset_gt(1) ELSE `` ).
          IF next_gt = `=`. APPEND VALUE #( kind = `GTE` position = offset ) TO result. offset += 2.
          ELSE. APPEND VALUE #( kind = `GT` position = offset ) TO result. offset += 1. ENDIF.
        WHEN `'`.
          DATA(start) = offset.
          DATA(text) = ``.
          DATA(terminated) = abap_false.
          offset += 1.
          WHILE offset < length.
            character = input+offset(1).
            IF character <> `'`.
              text = |{ text }{ character }|. offset += 1.
            ELSE.
              DATA(next_quote_offset) = offset + 1.
              IF next_quote_offset < length AND input+next_quote_offset(1) = `'`.
              text = |{ text }'|. offset += 2.
              ELSE.
                offset += 1. terminated = abap_true. EXIT.
              ENDIF.
            ENDIF.
          ENDWHILE.
          IF terminated = abap_false.
            RAISE EXCEPTION TYPE zcx_sql_where EXPORTING position = start message = `Unterminated string literal`.
          ENDIF.
          APPEND VALUE #( kind = `STRING` text = text position = start ) TO result.
        WHEN OTHERS.
          IF me->is_identifier_start( character ) = abap_true.
            start = offset. text = character. offset += 1.
            WHILE offset < length AND me->is_identifier_continue( input+offset(1) ) = abap_true.
              text = |{ text }{ input+offset(1) }|. offset += 1.
            ENDWHILE.
            DATA(upper) = to_upper( text ).
            DATA(kind) = COND string( WHEN upper = `AND` THEN `AND` WHEN upper = `OR` THEN `OR`
                                      WHEN upper = `IN` THEN `IN` ELSE `IDENTIFIER` ).
            APPEND VALUE #( kind = kind text = text position = start ) TO result.
          ELSEIF character CO `0123456789`.
            start = offset. text = character. offset += 1.
            WHILE offset < length AND input+offset(1) CO `0123456789`.
              text = |{ text }{ input+offset(1) }|. offset += 1.
            ENDWHILE.
            APPEND VALUE #( kind = `INTEGER` text = text position = start ) TO result.
          ELSE.
            RAISE EXCEPTION TYPE zcx_sql_where EXPORTING position = offset message = |Unexpected character { character }|.
          ENDIF.
      ENDCASE.
    ENDWHILE.
    APPEND VALUE #( kind = `EOF` position = length ) TO result.
  ENDMETHOD.

  METHOD parse_expression. result = parse_or( ). ENDMETHOD.
  METHOD parse_or.
    result = parse_and( ).
    WHILE matches( `OR` ) = abap_true.
      result = NEW zcl_sql_where( node_type = `OR` left = result right = parse_and( ) ).
    ENDWHILE.
  ENDMETHOD.
  METHOD parse_and.
    result = parse_primary( ).
    WHILE matches( `AND` ) = abap_true.
      result = NEW zcl_sql_where( node_type = `AND` left = result right = parse_primary( ) ).
    ENDWHILE.
  ENDMETHOD.
  METHOD parse_primary.
    IF matches( `LEFT_PAREN` ) = abap_true.
      result = parse_expression( ).
      expect( kind = `RIGHT_PAREN` message = `Expected ')' after expression` ).
    ELSE.
      result = parse_comparison( ).
    ENDIF.
  ENDMETHOD.

  METHOD parse_comparison.
    DATA(table_name) = parse_identifier( ).
    expect( kind = `DOT` message = `Expected '.' in column reference` ).
    DATA(column_name) = parse_identifier( ).
    DATA(operator) = parse_operator( ).
    DATA parsed_value TYPE ty_value.
    IF operator = `IN`.
      parsed_value = parse_in_list( ).
    ELSE.
      parsed_value = parse_value( ).
    ENDIF.
    result = NEW zcl_sql_where(
      node_type = `COMPARISON` table_name = table_name column_name = column_name
      comparison_operator = operator value_type = parsed_value-kind value = parsed_value-text
      value_table = parsed_value-table_name value_column = parsed_value-column_name values = parsed_value-values ).
  ENDMETHOD.

  METHOD parse_operator.
    CASE peek( )-kind.
      WHEN `EQ`. result = `=`.
      WHEN `NE`. result = `<>`.
      WHEN `LT`. result = `<`.
      WHEN `LTE`. result = `<=`.
      WHEN `GT`. result = `>`.
      WHEN `GTE`. result = `>=`.
      WHEN `IN`. result = `IN`.
      WHEN OTHERS. RAISE EXCEPTION TYPE zcx_sql_where EXPORTING position = peek( )-position message = `Expected comparison operator`.
    ENDCASE.
    advance( ).
  ENDMETHOD.

  METHOD parse_value.
    CASE peek( )-kind.
      WHEN `STRING`. result = VALUE #( kind = `STRING` text = advance( )-text ).
      WHEN `INTEGER`. result = VALUE #( kind = `INTEGER` text = advance( )-text ).
      WHEN `IDENTIFIER`.
        result-kind = `COLUMN`. result-table_name = parse_identifier( ).
        expect( kind = `DOT` message = `Expected '.' in value column reference` ).
        result-column_name = parse_identifier( ).
      WHEN OTHERS. RAISE EXCEPTION TYPE zcx_sql_where EXPORTING position = peek( )-position message = `Expected a value`.
    ENDCASE.
  ENDMETHOD.

  METHOD parse_in_list.
    expect( kind = `LEFT_PAREN` message = `Expected '(' after IN` ).
    result-kind = `LIST`.
    WHILE abap_true = abap_true.
      IF peek( )-kind <> `STRING`.
        RAISE EXCEPTION TYPE zcx_sql_where EXPORTING position = peek( )-position message = `IN lists support string literals only`.
      ENDIF.
      APPEND advance( )-text TO result-values.
      IF matches( `COMMA` ) = abap_false. EXIT. ENDIF.
    ENDWHILE.
    expect( kind = `RIGHT_PAREN` message = `Expected ')' after IN list` ).
  ENDMETHOD.

  METHOD parse_identifier.
    IF peek( )-kind <> `IDENTIFIER`.
      RAISE EXCEPTION TYPE zcx_sql_where EXPORTING position = peek( )-position message = `Expected identifier`.
    ENDIF.
    result = advance( )-text.
  ENDMETHOD.
  METHOD expect.
    IF matches( kind ) = abap_false.
      RAISE EXCEPTION TYPE zcx_sql_where EXPORTING position = peek( )-position message = message.
    ENDIF.
  ENDMETHOD.
  METHOD matches.
    result = xsdbool( peek( )-kind = kind ).
    IF result = abap_true. advance( ). ENDIF.
  ENDMETHOD.
  METHOD peek. READ TABLE tokens INDEX index INTO result. ENDMETHOD.
  METHOD advance.
    result = peek( ).
    IF result-kind <> `EOF`. index += 1. ENDIF.
  ENDMETHOD.
  METHOD is_identifier_start. result = xsdbool( character CO `ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz_` ). ENDMETHOD.
  METHOD is_identifier_continue. result = xsdbool( character CO `ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz_0123456789` ). ENDMETHOD.
ENDCLASS.
