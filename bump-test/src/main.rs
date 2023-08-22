fn main() {
    let start = std::time::Instant::now();

    let mut bump = bumpme::Bump::new();
    for i in 0..1_000_000_000 {
        if i % 10_000 == 0 {
            bump = bumpme::Bump::new();
        }
        bump.alloc_str("hello world");
    }
    drop(bump);
    dbg!(start.elapsed());

    let start = std::time::Instant::now();

    let mut bump = bumpalo::Bump::new();
    for i in 0..1_000_000_000 {
        if i % 10_000 == 0 {
            bump = bumpalo::Bump::new();
        }
        bump.alloc_str("hello world");
    }
    drop(bump);
    dbg!(start.elapsed());
}
