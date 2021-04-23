/*
    Macro to reduce code duplication: 
    each macro in this file is designed to 
    factorize all the syntactic noise present
    in certain definition or declaration
*/

/**
 * Convert the input expression (usually just variable names)
 * into a complex Error definition block
 */
#[macro_export]
macro_rules! new_name_error {
    ($name:expr, $orig_cls:expr, $ridef_cls:expr, $orig_loc:expr, $ridef_loc:expr) => {{
        let err = NameRidefinitionError {
            name: $name,
            orig_loc: $orig_loc,
            ridef_loc: $ridef_loc,
            orig_class: $orig_cls,
            ridef_class: $ridef_cls,
        };
        let name_error = NameError::NameRidefinitionError(err);
        Err(name_error)
    }};
}

/**
 * Add the implementation for From<T> for NameError
 */
#[macro_export]
macro_rules! into_name_error {
    ($name:ident ) => {
        impl<'a> From<$name<'a>> for NameError<'a> {
            fn from(err: $name) -> NameError {
                NameError::$name(err)
            }
        }
    };
}
