#![no_std]

use core::{alloc::Layout, marker::PhantomData, ptr::NonNull};

extern crate alloc;

pub mod boxed;
pub mod raw;

#[derive(Default)]
#[repr(transparent)]
pub struct Bump {
    pub raw: raw::Bump,
}

pub struct Allocation<'a> {
    ptr: NonNull<u8>,
    layout: Layout,
    bump: PhantomData<&'a Bump>,
}

impl Bump {
    pub fn new() -> Self {
        Self {
            raw: raw::Bump::new(),
        }
    }

    #[inline]
    pub fn try_alloc_layout(&self, layout: Layout) -> Option<Allocation<'_>> {
        self.raw.try_alloc_layout(layout).map(|ptr| Allocation {
            ptr,
            layout,
            bump: PhantomData,
        })
    }

    #[inline]
    pub fn alloc_layout(&self, layout: Layout) -> Allocation<'_> {
        Allocation {
            ptr: self.raw.alloc_layout(layout),
            layout,
            bump: PhantomData,
        }
    }

    pub fn alloc<T>(&self, value: T) -> boxed::Box<'_, T> {
        self.alloc_layout(Layout::new::<T>()).write(value)
    }

    pub fn alloc_copy_slice<T: Copy>(&self, value: &[T]) -> &mut [T] {
        boxed::Box::into_mut_ref(
            self.alloc_layout(Layout::for_value(value))
                .copy_from_slice(value),
        )
    }

    #[inline]
    pub fn alloc_str(&self, value: &str) -> &mut str {
        boxed::Box::into_mut_ref(
            self.alloc_layout(Layout::for_value(value))
                .copy_from_str(value),
        )
    }
}

impl<'a> Allocation<'a> {
    #[inline]
    pub fn by_ref(&mut self) -> Allocation<'_> {
        Self {
            ptr: self.ptr,
            layout: self.layout,
            bump: PhantomData,
        }
    }

    #[inline]
    pub fn write<T>(self, value: T) -> boxed::Box<'a, T> {
        assert!(self.layout.align() >= core::mem::align_of::<T>());
        assert!(self.layout.size() >= core::mem::size_of::<T>());

        let ptr: *mut T = self.ptr.as_ptr().cast();
        unsafe { ptr.write(value) }
        unsafe { boxed::Box::from_raw(ptr) }
    }

    #[inline]
    pub fn write_slice<T>(self, mut mk_value: impl FnMut() -> T) -> boxed::Box<'a, [T]> {
        assert!(self.layout.align() >= core::mem::align_of::<T>());
        let len = self.layout.size() / core::mem::size_of::<T>();
        let ptr: *mut T = self.ptr.as_ptr().cast();
        let mut current = ptr;
        for _ in 0..len {
            unsafe { current.write(mk_value()) }
            unsafe { current = current.add(1) }
        }
        unsafe { boxed::Box::from_raw(core::ptr::slice_from_raw_parts_mut(ptr, len)) }
    }

    #[inline]
    pub fn copy_from_slice<T>(self, slice: &[T]) -> boxed::Box<'a, [T]>
    where
        T: Copy,
    {
        assert!(self.layout.align() >= core::mem::align_of::<T>());
        if !slice.is_empty() {
            assert!(self.layout.size() / slice.len() >= core::mem::size_of::<T>());
        }

        let ptr: *mut T = self.ptr.as_ptr().cast();
        unsafe { ptr.copy_from_nonoverlapping(slice.as_ptr(), slice.len()) }
        unsafe { boxed::Box::from_raw(core::ptr::slice_from_raw_parts_mut(ptr, slice.len())) }
    }

    #[inline]
    pub fn copy_from_str(self, slice: &str) -> boxed::Box<'a, str> {
        unsafe { boxed::Box::from_utf8_unchecked(self.copy_from_slice(slice.as_bytes())) }
    }
}
