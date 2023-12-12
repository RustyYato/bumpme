use core::{
    alloc::Layout,
    marker::PhantomData,
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

use crate::Bump;

pub struct Box<'a, T: ?Sized> {
    ptr: NonNull<T>,
    bump: PhantomData<&'a Bump>,
}

impl<T: ?Sized> Drop for Box<'_, T> {
    fn drop(&mut self) {
        unsafe { self.ptr.as_ptr().drop_in_place() }
    }
}

pub trait NoDropGlue {}

impl<T: Copy> NoDropGlue for T {}
impl<T: NoDropGlue> NoDropGlue for [T] {}
impl NoDropGlue for str {}

impl<'a, T: ?Sized> Box<'a, T> {
    #[inline]
    pub unsafe fn from_raw(ptr: *mut T) -> Self {
        Self {
            ptr: NonNull::new_unchecked(ptr),
            bump: PhantomData,
        }
    }

    #[inline]
    pub fn leak(this: Self) -> &'a mut T {
        let this = ManuallyDrop::new(this);
        unsafe { &mut *this.ptr.as_ptr() }
    }

    #[inline]
    pub fn into_mut_ref(this: Self) -> &'a mut T
    where
        T: NoDropGlue,
    {
        Self::leak(this)
    }

    #[inline]
    pub fn into_ref(this: Self) -> &'a T
    where
        T: NoDropGlue,
    {
        Self::into_mut_ref(this)
    }

    #[inline]
    pub fn drop_in_place(this: Self) -> super::Allocation<'a> {
        super::Allocation {
            ptr: this.ptr.cast(),
            layout: Layout::for_value(unsafe { this.ptr.as_ref() }),
            bump: PhantomData,
        }
    }
}

impl<'a> Box<'a, [u8]> {
    #[inline]
    pub unsafe fn from_utf8_unchecked(this: Self) -> Box<'a, str> {
        Box {
            ptr: NonNull::new_unchecked(this.ptr.as_ptr() as *mut str),
            bump: PhantomData,
        }
    }
}

impl<T: ?Sized> Deref for Box<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.ptr.as_ref() }
    }
}

impl<T: ?Sized> DerefMut for Box<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.ptr.as_mut() }
    }
}
