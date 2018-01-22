use std::rc::Rc;
use std::collections::VecDeque;
use std::cell::{RefCell, UnsafeCell};
use bytes::{BufMut, BytesMut};

use body::Binary;


/// Internal use only! unsafe
#[derive(Debug)]
pub(crate) struct SharedBytesPool(RefCell<VecDeque<Rc<UnsafeCell<BytesMut>>>>);

impl SharedBytesPool {
    pub fn new() -> SharedBytesPool {
        SharedBytesPool(RefCell::new(VecDeque::with_capacity(128)))
    }

    pub fn get_bytes(&self) -> Rc<UnsafeCell<BytesMut>> {
        if let Some(bytes) = self.0.borrow_mut().pop_front() {
            bytes
        } else {
            Rc::new(UnsafeCell::new(BytesMut::new()))
        }
    }

    pub fn release_bytes(&self, bytes: Rc<UnsafeCell<BytesMut>>) {
        let v = &mut self.0.borrow_mut();
        if v.len() < 128 {
            unsafe { &mut *bytes.get() }.take();
            v.push_front(bytes);
        }
    }
}

#[derive(Debug)]
pub(crate) struct SharedBytes(
    Option<Rc<UnsafeCell<BytesMut>>>, Option<Rc<SharedBytesPool>>);

impl Drop for SharedBytes {
    fn drop(&mut self) {
        if let Some(ref pool) = self.1 {
            if let Some(bytes) = self.0.take() {
                if Rc::strong_count(&bytes) == 1 {
                    pool.release_bytes(bytes);
                }
            }
        }
    }
}

impl SharedBytes {

    pub fn empty() -> Self {
        SharedBytes(None, None)
    }

    pub fn new(bytes: Rc<UnsafeCell<BytesMut>>, pool: Rc<SharedBytesPool>) -> SharedBytes {
        SharedBytes(Some(bytes), Some(pool))
    }

    #[inline(always)]
    #[cfg_attr(feature = "cargo-clippy", allow(inline_always))]
    pub fn as_mut(&mut self) -> &mut BytesMut {
        unsafe{ &mut *self.0.as_ref().unwrap().as_ref().get() }
    }

    #[inline]
    pub fn len(&self) -> usize {
        unsafe { &*self.0.as_ref().unwrap().get() }.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        unsafe { &*self.0.as_ref().unwrap().get() }.is_empty()
    }

    #[inline]
    pub fn as_ref(&self) -> &[u8] {
        unsafe { &*self.0.as_ref().unwrap().get() }.as_ref()
    }

    pub fn split_to(&self, n: usize) -> BytesMut {
        unsafe{ &mut *self.0.as_ref().unwrap().as_ref().get() }.split_to(n)
    }

    pub fn take(&self) -> BytesMut {
        unsafe{ &mut *self.0.as_ref().unwrap().as_ref().get() }.take()
    }

    #[inline]
    pub fn reserve(&self, cnt: usize) {
        unsafe{ &mut *self.0.as_ref().unwrap().as_ref().get() }.reserve(cnt)
    }

    #[inline]
    #[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
    pub fn extend(&self, src: Binary) {
        unsafe{ &mut *self.0.as_ref().unwrap().as_ref().get() }.extend_from_slice(src.as_ref());
    }

    #[inline]
    pub fn extend_from_slice(&self, src: &[u8]) {
        unsafe{ &mut *self.0.as_ref().unwrap().as_ref().get() }.extend_from_slice(src);
    }

    #[inline]
    pub fn put_slice(&mut self, src: &[u8]) {
        self.as_mut().put_slice(src);
    }
}

impl Default for SharedBytes {
    fn default() -> Self {
        SharedBytes(Some(Rc::new(UnsafeCell::new(BytesMut::new()))), None)
    }
}

impl Clone for SharedBytes {
    fn clone(&self) -> SharedBytes {
        SharedBytes(self.0.clone(), self.1.clone())
    }
}
