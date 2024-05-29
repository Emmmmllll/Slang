// this is a comment
/* this also */

/// documentation comment
/** another documentation **/



// entry point of the binary
// this is done implicitly if the name is `main`
// + the function is in the moudle root of a binary
// + no other function is declared as entry point
#[entry]
fn main() {
    // calls `another function` with its default parameters
    // def!() is a special macro, which expands to an tuple with the default parameters
    // it can only be directly used as an parameter of an function
    // parameters without default value have to be specified manually
    // in this case def!() would internally expand to `(1, explicit_param!(), 0 )`
    another_function("Test123", ..def!());
    // this function could also be called by
    another_function(1, "Test123", 0);
    // or by
    another_function(..(1, "Test123", 0));
    // note that when expanding parameters with `..` all `explicit_param!()`
    // must be set expicitly
    // if optional characters should be changed, `$ident: $expr` must be used
    another_function("Test123", offset: 0, ..def!())
}


// function:
// `$function_signature` $( `;` | `$block` )
//
// funtion_signature:
// `fn` `$ident` `(` $( `$param` )`,`* $(`,`)? `)` `$ret`
//
// param:
// `$ident` `:` `$ty` $( `=` `$expr` )?
//
// e.g.                 arr: &[u8]
// or with default:     arr: &[u8] = &[1, 2, 3]
//
// ret:
// $( `$ty` $( `?` | `!` $( `$ty` )? )`?`  |  `!` $( `$ty` ) )?
//
// e.g.                     String          => String
// or as an option:         String!         => Option<String>
// or as an result:         String!Error    => Result<String, Error>
// or returns nothing:                      => ()
// or returns empty result: !Error          => Result<(), Error>
// or never returns:        !               => !

// this function multiplies the lenght of the string by a factor, which is `1` by default
fn another_function(factor: usize = 1, string: &str, offset: usize = 0) usize {
    // implicitly returns this expr without `;`
    string.len() * factor
}

fn basic_operations() usize {
    (4 + 2 * 8 - 7 << 2) / 4
    // should be processed like this:
    // (((4 + (2 * 8)) - 7) << 2) / 4
    // (((4 + 16) - 7) << 2) / 4
    // ((20 - 7) << 2) / 4
    // ( 13 << 2) / 4
    // 52 / 4
    // 13
}