use elc_proc::*;

/// Just try using elc
#[elc]
fn _foo(n: i32) -> i32 {
    'requires: {
        n > 2
    }
    'ensures: {
        ret > 8
    }
    n * n
}
