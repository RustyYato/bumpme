use std::hint::black_box;

fn main() {
    let start = std::time::Instant::now();

    let mut bump = bumpme::Bump::new();
    for i in 0..10_000_000_000u64 {
        if i % 100_000 == 0 {
            bump.reset();
        }
        bump.alloc_str(black_box("hello world"));
    }
    drop(bump);
    dbg!(start.elapsed());

    let start = std::time::Instant::now();

    let mut bump = bumpalo::Bump::new();
    for i in 0..10_000_000_000u64 {
        if i % 100_000 == 0 {
            bump.reset();
        }
        bump.alloc_str(black_box("hello world"));
    }
    drop(bump);
    dbg!(start.elapsed());
}
