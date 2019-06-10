#[macro_export]
macro_rules! constify {
    (
        if #[cfg($cfg:meta)]
        $visibility:vis fn $fnname:ident ($($argname:ident: $argtype:ty),*) -> $ret:ty $implementation:block
    ) => {
        #[cfg($cfg)]
        $visibility const fn $fnname($($argname: $argtype),*) -> $ret $implementation
        
        #[cfg(not($cfg))]
        $visibility fn $fnname($($argname: $argtype),*) -> $ret $implementation
    }
}