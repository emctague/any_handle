pub use std::any::Any;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

/// A thread-safe shared pointer to a value of any [Any] type, allowing for downcasting.
///
/// Internally, this uses [RwLock], allowing for multiple concurrent readers
/// or a single writer.
///
/// # Example
/// ```
/// use any_handle::{AnyHandle, Any};
///
/// struct SomeStruct (i32);
///
/// impl SomeStruct {
///     fn do_things_with(&self) {}
///     fn do_mut_things_with(&mut self) {}
/// }
///
/// fn demo() -> Option<()> {
///     // Initialize a handle with an unknown type.
///     // If you want to pass in a Box<dyn SomeOtherTrait>, instead of a concrete
///     // type, you will have to use `#![feature(trait_upcasting)]`, unfortunately.
///     let handle : AnyHandle<dyn Any> = AnyHandle::new(Box::new(SomeStruct(12)));
///     // Now we can put it in some sort of generic container...
///
///     // ...and when we retrieve it later:
///     let mut handle : AnyHandle<SomeStruct> = handle.downcast().ok()?;
///     handle.write().do_mut_things_with();
///     handle.read().do_things_with();
///     Some(())
/// }
///
/// fn main() { demo().unwrap() }
/// ```
pub struct AnyHandle<T: ?Sized>(Arc<RwLock<Box<dyn Any>>>, PhantomData<T>);

impl AnyHandle<dyn Any> {
    /// Initialize an AnyHandle from a [Box]<dyn [Any]>.
    pub fn new(inner: Box<dyn Any>) -> Self {
        Self(Arc::new(RwLock::new(inner)), PhantomData)
    }

    /// Downcast this handle from `dyn Any` to a specific type.
    /// If the stored data can be downcast to type Y, succeeds and
    /// returns Ok(the cast AnyHandle).
    /// If the data cannot be downcast, errors and returns Error(self).
    ///
    /// You may also downcast using `Option<AnyHandle<T>>::from`.
    pub fn downcast<Y: 'static>(self) -> Result<AnyHandle<Y>, Self> {
        if self.0.read().unwrap().is::<Y>() {
            Ok(AnyHandle::<Y>(self.0, PhantomData))
        } else {
            Err(self)
        }
    }
}

impl<T: ?Sized> AnyHandle<T> {

    /// Get a 'read guard' that allows for reading from the object.
    /// Any number of read guards can exist at a given time, but
    /// not at the same time as any write guards, so this may block
    /// or result in deadlocks if used improperly.
    #[inline(always)]
    pub fn read(&self) -> AnyHandleReadGuard<'_, T> {
        AnyHandleReadGuard(self.0.read().unwrap(), PhantomData)
    }

    /// Get a 'write guard' that allows for writing to the object.
    /// Only one write guard can exist at a given time for an object,
    /// and not at the same time as any read guards, so this may
    /// block or result in deadlocks if used improperly.
    #[inline(always)]
    pub fn write(&mut self) -> AnyHandleWriteGuard<'_, T> {
        AnyHandleWriteGuard(self.0.write().unwrap(), PhantomData)
    }

    /// Get a count of the number of living references to this object.
    #[inline(always)]
    pub fn reference_count(&self) -> usize {
        Arc::strong_count(&self.0)
    }
}

impl<T: Sized + 'static> From<AnyHandle<dyn Any>> for Option<AnyHandle<T>> {
    /// Downcast an AnyHandle<dyn [Any]> to an AnyHandle<T>.
    fn from(item: AnyHandle<dyn Any>) -> Option<AnyHandle<T>> {
        item.downcast().ok()
    }
}


impl<T: ?Sized> Clone for AnyHandle<T> {
    /// Make a new copy of this handle.
    /// This will not copy the object within, and will increase the
    /// reference count.
    #[inline(always)]
    fn clone(&self) -> Self {
        Self(self.0.clone(), PhantomData)
    }
}


pub struct AnyHandleReadGuard<'a, T: ?Sized + 'a>(RwLockReadGuard<'a, Box<dyn Any>>, PhantomData<T>);
pub struct AnyHandleWriteGuard<'a, T: ?Sized + 'a>(RwLockWriteGuard<'a, Box<dyn Any>>, PhantomData<T>);

impl<'a, T: 'a + 'static> Deref for AnyHandleReadGuard<'a, T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*(self.0.deref().deref() as *const dyn Any as *const T) }
    }
}

impl<'a, T: 'a + 'static> Deref for AnyHandleWriteGuard<'a, T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*(self.0.deref().deref() as *const dyn Any as *const T) }
    }
}

impl<'a, T: 'a + 'static> DerefMut for AnyHandleWriteGuard<'a, T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *(self.0.deref_mut().deref_mut() as *mut dyn Any as *mut T) }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    struct SomeStruct { value: i32 }

    #[test]
    fn basic_reading_writing() {
        let handle : AnyHandle<dyn Any> = AnyHandle::new(Box::new(SomeStruct {
            value: 12
        }));

        let handle : Option<AnyHandle<SomeStruct>> = handle.into();
        let mut handle = handle.unwrap();

        {
            let handle_two = handle.clone();

            assert_eq!(handle.read().value, 12);
            assert_eq!(handle_two.read().value, 12);
            assert_eq!(handle.reference_count(), 2);
            handle.write().value = 24;
            assert_eq!(handle.read().value, 24);
            assert_eq!(handle_two.read().value, 24);
        }
        assert_eq!(handle.reference_count(), 1);
    }

    #[test]
    fn type_safety() {
        let handle = AnyHandle::new(Box::new(SomeStruct { value: 12 }));
        Into::<Option<AnyHandle<SomeStruct>>>::into(handle).unwrap();
    }
}
