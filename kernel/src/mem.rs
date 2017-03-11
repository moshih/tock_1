use AppId;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
use core::ptr::Unique;
use core::slice;
use process;

#[derive(Debug)]
pub struct Private;
#[derive(Debug)]
pub struct Shared;

pub struct AppPtr<L, T> {
    ptr: Unique<T>,
    process: AppId,
    _phantom: PhantomData<L>,
}

impl<L, T> AppPtr<L, T> {
    pub unsafe fn new(ptr: *mut T, appid: AppId) -> AppPtr<L, T> {
        AppPtr {
            ptr: Unique::new(ptr),
            process: appid,
            _phantom: PhantomData,
        }
    }
}

impl<L, T> Deref for AppPtr<L, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { self.ptr.get() }
    }
}

impl<L, T> DerefMut for AppPtr<L, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { self.ptr.get_mut() }
    }
}

impl<L, T> Drop for AppPtr<L, T> {
    fn drop(&mut self) {
        unsafe {
            let ps = &mut process::PROCS;
            if ps.len() > self.process.idx() {
                ps[self.process.idx()].as_mut().map(|process| process.free(self.ptr.get_mut()));
            }
        }
    }
}

pub struct AppSlice<L, T> {
    ptr: AppPtr<L, T>,
    len: usize,
}

impl<L, T> AppSlice<L, T> {
    pub unsafe fn new(ptr: *mut T, len: usize, appid: AppId) -> AppSlice<L, T> {
        AppSlice {
            ptr: AppPtr::new(ptr, appid),
            len: len,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub unsafe fn ptr(&self) -> *const T {
        self.ptr.ptr.get() as *const T
    }

    pub unsafe fn expose_to(&self, appid: AppId) -> bool {
        let ps = &mut process::PROCS;
        if appid.idx() != self.ptr.process.idx() && ps.len() > appid.idx() {
            ps[appid.idx()]
                .as_ref()
                .map(|process| process.add_mpu_region(self.ptr() as *const u8, self.len()))
                .unwrap_or(false)
        } else {
            false
        }
    }

    pub fn iter(&self) -> slice::Iter<T> {
        self.as_ref().iter()
    }

    pub fn iter_mut(&mut self) -> slice::IterMut<T> {
        self.as_mut().iter_mut()
    }
}

impl<L, T> AsRef<[T]> for AppSlice<L, T> {
    fn as_ref(&self) -> &[T] {
        unsafe { slice::from_raw_parts(self.ptr.ptr.get(), self.len) }
    }
}

impl<L, T> AsMut<[T]> for AppSlice<L, T> {
    fn as_mut(&mut self) -> &mut [T] {
        unsafe { slice::from_raw_parts_mut(self.ptr.ptr.get_mut(), self.len) }
    }
}
