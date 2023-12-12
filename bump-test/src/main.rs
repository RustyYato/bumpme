use std::hint::black_box;

fn main() {
    throughput(bumpme::Bump::new);
    throughput(bumpalo::Bump::new);
    latency(bumpme::Bump::new);
    latency(bumpalo::Bump::new);
}

pub trait BumpAlloc {
    fn reset(&mut self);

    #[allow(clippy::mut_from_ref)]
    fn alloc_str(&self, s: &str) -> &mut str;
}

impl BumpAlloc for bumpme::Bump {
    fn reset(&mut self) {
        self.reset()
    }

    fn alloc_str(&self, s: &str) -> &mut str {
        self.alloc_str(s)
    }
}

impl BumpAlloc for bumpalo::Bump {
    fn reset(&mut self) {
        self.reset()
    }

    fn alloc_str(&self, s: &str) -> &mut str {
        self.alloc_str(s)
    }
}

fn throughput<F: Fn() -> A, A: BumpAlloc>(mk_bump: F) {
    let start = std::time::Instant::now();

    let mut bump = mk_bump();
    for i in 0..10_000_000_000u64 {
        if i % 10_000 == 0 {
            bump.reset();
        }
        bump.alloc_str(black_box("hello world"));
    }
    drop(bump);
    dbg!(start.elapsed());
}

fn latency<F: Fn() -> A, A: BumpAlloc>(mk_bump: F) {
    let mut timings = [std::time::Duration::ZERO; 4];

    let mut bump = mk_bump();
    for i in 0..1_000_000_000u64 {
        if i % 10_000 == 0 {
            bump.reset();
        }

        let start = std::time::Instant::now();
        bump.alloc_str(black_box("hello world"));
        let time = start.elapsed();
        for prev in timings.iter_mut() {
            if time > *prev {
                *prev = time;
                break;
            }
        }
    }
    drop(bump);
    dbg!(timings);
}
