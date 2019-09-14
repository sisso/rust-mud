#[macro_export]
macro_rules! unpack {
    // `()` indicates that the macro takes no argument.
    ($name:expr) => (
        // The macro will expand into the contents of this block.
        match $name {
            Some(v) => v,
            None => return,
        }
    )
}
