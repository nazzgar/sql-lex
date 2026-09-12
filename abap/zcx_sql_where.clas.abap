CLASS zcx_sql_where DEFINITION PUBLIC INHERITING FROM cx_static_check CREATE PUBLIC.
  PUBLIC SECTION.
    DATA position TYPE i READ-ONLY.
    DATA message TYPE string READ-ONLY.
    METHODS constructor
      IMPORTING
        position TYPE i
        message  TYPE string.
ENDCLASS.

CLASS zcx_sql_where IMPLEMENTATION.
  METHOD constructor.
    super->constructor( ).
    me->position = position.
    me->message = message.
  ENDMETHOD.
ENDCLASS.
