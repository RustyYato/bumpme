pub fn alloc_me<'a>(bump: &'a bumpme::Bump) -> &'a mut str {
    bump.alloc_str("hello world")
}

pub fn alloc_alo<'a>(bump: &'a bumpalo::Bump) -> &'a mut str {
    bump.alloc_str("hello world")
}
