use core::cell::RefCell;
use core::future::Future;
use core::pin::Pin;

pub struct Task<'a> {
    pub name: &'a str,
    pub future: StackBoxFuture<'a>,
}

impl<'a> Task<'a> {
    pub fn new(name: &'a str, future: StackBoxFuture<'a>) -> Self {
        Self { name, future }
    }
}

pub struct StackBox<'a, T: ?Sized> {
    pub value: RefCell<Pin<&'a mut T>>,
}

impl<'a, T: ?Sized> StackBox<'a, T> {
    pub fn new(value: &'a mut T) -> Self {
        StackBox {
            value: RefCell::new(unsafe { Pin::new_unchecked(value) }),
        }
    }
}

pub type StackBoxFuture<'a> = StackBox<'a, dyn Future<Output = ()> + 'a>;
