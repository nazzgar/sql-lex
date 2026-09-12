" ABAP Unit tests. Install zcx_sql_where and zcl_sql_where first.
CLASS ltc_sql_where DEFINITION FINAL FOR TESTING DURATION SHORT RISK LEVEL HARMLESS.
  PRIVATE SECTION.
    METHODS parses_all_comparison_operators FOR TESTING RAISING cx_static_check.
    METHODS parses_in_list FOR TESTING RAISING cx_static_check.
    METHODS parses_column_to_column_comparison FOR TESTING RAISING cx_static_check.
    METHODS respects_boolean_precedence_and_parentheses FOR TESTING RAISING cx_static_check.
ENDCLASS.

CLASS ltc_sql_where IMPLEMENTATION.
  METHOD parses_all_comparison_operators.
    DATA(ast) = zcl_sql_where=>parse(
      `users.role = 'admin' AND users.active <> 'yes' AND users.some_value > 123 AND users.some_value2 < 123 AND users.some_value2 <= 123 AND users.some_value2 >= 123` ).
    cl_abap_unit_assert=>assert_equals( act = ast->node_type exp = `AND` ).
    cl_abap_unit_assert=>assert_equals( act = ast->right->comparison_operator exp = `>=` ).
    cl_abap_unit_assert=>assert_equals( act = ast->right->value exp = `123` ).
  ENDMETHOD.

  METHOD parses_in_list.
    DATA(ast) = zcl_sql_where=>parse( `users.role IN ('aba', 'ddsadas')` ).
    cl_abap_unit_assert=>assert_equals( act = ast->node_type exp = `COMPARISON` ).
    cl_abap_unit_assert=>assert_equals( act = ast->comparison_operator exp = `IN` ).
    cl_abap_unit_assert=>assert_equals( act = lines( ast->values ) exp = 2 ).
    cl_abap_unit_assert=>assert_equals( act = ast->values[ 2 ] exp = `ddsadas` ).
  ENDMETHOD.

  METHOD parses_column_to_column_comparison.
    DATA(ast) = zcl_sql_where=>parse( `users.role = users.second_role` ).
    cl_abap_unit_assert=>assert_equals( act = ast->node_type exp = `COMPARISON` ).
    cl_abap_unit_assert=>assert_equals( act = ast->comparison_operator exp = `=` ).
    cl_abap_unit_assert=>assert_equals( act = ast->value_type exp = `COLUMN` ).
    cl_abap_unit_assert=>assert_equals( act = ast->value_table exp = `users` ).
    cl_abap_unit_assert=>assert_equals( act = ast->value_column exp = `second_role` ).
  ENDMETHOD.

  METHOD respects_boolean_precedence_and_parentheses.
    DATA(ast) = zcl_sql_where=>parse(
      `(users.role = 'admin' OR users.role = 'editor') AND users.active = 'yes'` ).
    cl_abap_unit_assert=>assert_equals( act = ast->node_type exp = `AND` ).
    cl_abap_unit_assert=>assert_equals( act = ast->left->node_type exp = `OR` ).
    cl_abap_unit_assert=>assert_equals( act = ast->left->left->value exp = `admin` ).
    cl_abap_unit_assert=>assert_equals( act = ast->left->right->value exp = `editor` ).
    cl_abap_unit_assert=>assert_equals( act = ast->right->column_name exp = `active` ).
  ENDMETHOD.
ENDCLASS.
