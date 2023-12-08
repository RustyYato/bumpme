use alloc::alloc::{alloc, dealloc, handle_alloc_error as oom};
use core::{alloc::Layout, cell::Cell, ptr::NonNull};

pub struct Bump {
    chunk: Cell<NonNull<Chunk>>,
}

unsafe impl Send for Bump {}

impl Drop for Bump {
    #[inline]
    fn drop(&mut self) {
        free_chunk_list(self.chunk.get())
    }
}

fn free_chunk_list(mut chunk: NonNull<Chunk>) {
    loop {
        let next = unsafe { chunk.as_ref().next };
        let layout = unsafe { chunk.as_ref().layout };
        unsafe { dealloc(chunk.as_ptr().cast(), layout) }

        if let Some(next) = next {
            chunk = next;
        } else {
            break;
        }
    }
}

struct Chunk {
    end: Cell<*mut u8>,
    start: *mut u8,
    layout: Layout,
    next: Option<NonNull<Chunk>>,
}

impl Chunk {
    #[inline]
    fn calculate_alloc_ptr(&self, layout: Layout) -> *mut u8 {
        let end = self.end.get();
        let end_addr = addr(end);
        let new_addr_unaligned = end_addr.saturating_sub(layout.size());
        let new_addr = new_addr_unaligned & !layout.align().wrapping_sub(1);
        let offset = new_addr as isize - end_addr as isize;
        end.wrapping_offset(offset)
    }

    #[inline]
    unsafe fn alloc_layout_unchecked(&self, layout: Layout) -> NonNull<u8> {
        let new = self.calculate_alloc_ptr(layout);

        NonNull::new_unchecked(new)
    }

    #[inline]
    fn alloc_layout(&self, layout: Layout) -> Option<NonNull<u8>> {
        let new = self.calculate_alloc_ptr(layout);

        if new >= self.start {
            self.end.set(new);

            Some(unsafe { NonNull::new_unchecked(new) })
        } else {
            None
        }
    }
}

impl Bump {
    #[inline]
    pub fn new() -> Self {
        Self::with_capacity(1 << 11)
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self::try_with_capacity(capacity)
            .unwrap_or_else(|| oom(Layout::from_size_align(capacity, 1).unwrap()))
    }

    #[inline]
    pub fn try_with_capacity(capacity: usize) -> Option<Self> {
        let bump = Self {
            chunk: Cell::new(Self::create_chunk(
                Layout::from_size_align(capacity, core::mem::align_of::<usize>())
                    .ok()?
                    .pad_to_align(),
                None,
            )?),
        };

        Some(bump)
    }

    #[inline]
    pub fn reset(&mut self) {
        let chunk_ptr = self.chunk.get();
        let chunk = unsafe { self.chunk.get().as_mut() };

        if let Some(chunk) = chunk.next.take() {
            free_chunk_list(chunk);
        }

        chunk.end = Cell::new(unsafe { chunk_ptr.cast::<u8>().as_ptr().add(chunk.layout.size()) });
    }

    fn create_chunk(layout: Layout, next: Option<NonNull<Chunk>>) -> Option<NonNull<Chunk>> {
        let (layout, _) = Layout::new::<Chunk>().extend(layout).unwrap();

        let ptr_bytes = unsafe { alloc(layout) };
        let ptr = NonNull::new(ptr_bytes)?;
        let ptr = ptr.cast::<Chunk>();

        unsafe {
            ptr.as_ptr().write(Chunk {
                layout,
                next,
                start: ptr_bytes.add(core::mem::size_of::<Chunk>()),
                end: Cell::new(ptr_bytes.add(layout.size())),
            });
        }

        Some(ptr)
    }

    #[cold]
    #[inline(never)]
    fn try_new_chunk(&self, layout: Layout) -> Option<()> {
        let chunk_ptr = self.chunk.get();
        let chunk = unsafe { chunk_ptr.as_ref() };

        let layout = Layout::from_size_align(
            layout.size().max(chunk.layout.size().wrapping_mul(2)),
            layout.align().max(chunk.layout.align()),
        )
        .unwrap()
        .pad_to_align();

        let ptr = Self::create_chunk(layout, Some(chunk_ptr))?;
        self.chunk.set(ptr);

        Some(())
    }

    #[inline]
    pub fn try_alloc_layout(&self, layout: Layout) -> Option<NonNull<u8>> {
        self.try_alloc_layout_fast(layout)
            .or_else(|| self.try_alloc_layout_slow(layout))
    }

    #[inline]
    fn try_alloc_layout_fast(&self, layout: Layout) -> Option<NonNull<u8>> {
        unsafe { self.chunk.get().as_ref() }.alloc_layout(layout)
    }

    #[cold]
    #[inline(never)]
    fn try_alloc_layout_slow(&self, layout: Layout) -> Option<NonNull<u8>> {
        self.try_new_chunk(layout)?;

        let chunk = unsafe { self.chunk.get().as_ref() };

        Some(unsafe { chunk.alloc_layout_unchecked(layout) })
    }

    #[inline]
    pub fn alloc_layout(&self, layout: Layout) -> NonNull<u8> {
        self.try_alloc_layout_fast(layout)
            .unwrap_or_else(|| self.alloc_layout_slow(layout))
    }

    #[cold]
    #[inline(never)]
    fn alloc_layout_slow(&self, layout: Layout) -> NonNull<u8> {
        self.try_new_chunk(layout).unwrap_or_else(|| oom(layout));

        let chunk = unsafe { self.chunk.get().as_ref() };

        unsafe { chunk.alloc_layout_unchecked(layout) }
    }
}

impl Default for Bump {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(clippy::transmutes_expressible_as_ptr_casts)]
fn addr<T>(ptr: *mut T) -> usize {
    unsafe { core::mem::transmute(ptr) }
}

#[test]
fn test_extremely_large_layout() {
    let chunk = Bump::create_chunk(Layout::new::<()>(), None).unwrap();
    let chunk = unsafe { chunk.as_ref() };
    let ptr = chunk
        .calculate_alloc_ptr(Layout::from_size_align(chunk.end.get() as usize + 1, 1).unwrap());
    assert!(ptr.is_null())
}
