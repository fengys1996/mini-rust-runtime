use std::{
    mem,
    sync::Arc,
    task::{RawWaker, RawWakerVTable, Waker},
};

pub trait ArcWake: Send + Sync {
    fn wake(self: Arc<Self>) {
        Self::wake_by_ref(&self)
    }

    fn wake_by_ref(arc_self: &Arc<Self>);
}

pub fn waker<W>(wake: Arc<W>) -> Waker
where
    W: ArcWake + 'static,
{
    // Q: cast cost high?
    let ptr = Arc::into_raw(wake).cast::<()>();

    unsafe { Waker::from_raw(RawWaker::new(ptr, waker_vtable::<W>())) }
}

fn waker_vtable<W: ArcWake>() -> &'static RawWakerVTable {
    todo!()
}

// TODO(fys): remove dead_code
#[allow(dead_code)]
unsafe fn clone_arc_raw<T: ArcWake>(data: *const ()) -> RawWaker {
    increase_refcount::<T>(data);
    RawWaker::new(data, waker_vtable::<T>())
}

unsafe fn increase_refcount<T: ArcWake>(data: *const ()) {
    // Retain Arc, but don't touch refcount by wrapping in ManuallyDrop
    let arc = mem::ManuallyDrop::new(Arc::<T>::from_raw(data.cast::<T>()));
    // Now increase refcount, but don't drop new refcount either
    let _arc_clone: mem::ManuallyDrop<_> = arc.clone();
}

// TODO(fys): remove unused test
#[cfg(test)]
mod tests {
    use std::{mem::ManuallyDrop, sync::Arc};

    #[test]
    fn learn_forget() {
        let s = String::new();
        std::mem::forget(s);

        let mut v = vec![65, 122];
        // Build a `String` using the contents of `v`
        let s = unsafe { String::from_raw_parts(v.as_mut_ptr(), v.len(), v.capacity()) };
        // leak `v` because its memory is now managed by `s`
        std::mem::forget(v); // ERROR - v is invalid and must not be passed to a function
        assert_eq!(s, "Az");
        // `s` is implicitly dropped and its memory deallocated.
        // Use ManuallyDrop solve it.
    }

    #[test]
    fn learn_manually_drop() {
        let v = vec![65, 122];

        // Before we disassemble `v` into its raw parts, make sure it
        // does not get dropped!
        let mut v = ManuallyDrop::new(v);

        // Now disassemble `v`. These operations cannot panic, so there cannot be a leak.
        let ptr = v.as_mut_ptr();
        let len = v.len();
        let cap = v.capacity();

        // Finally, build a `String`.
        let s = unsafe { String::from_raw_parts(ptr, len, cap) };
        assert_eq!(s, "Az");

        // unsafe {
        // // remove //, it will panic, since free twice
        //     ManuallyDrop::drop(&mut v);
        // }
        // drop(v);
        // `s` is implicitly dropped and its memory deallocated.
    }

    #[test]
    fn learn_arc_into_raw1() {
        let a = Arc::new(1);
        let a_clone = a.clone();

        let a_p = Arc::into_raw(a);
        unsafe {
            let ptr = &*a_p;
            Arc::decrement_strong_count(ptr);
        };
        assert_eq!(1, *a_clone);
    }

    #[test]
    fn learn_arc_into_raw2() {
        let a = Arc::new(1);
        let a_clone = a.clone();

        assert_eq!(2, Arc::strong_count(&a));

        let a_p = Arc::into_raw(a);

        assert_eq!(2, Arc::strong_count(&a_clone));

        let ptr = unsafe {
            // Q: what happen with bind
            // let ptr = &*a_p;
            // ptr
            &*a_p
        };
        drop(a_clone);

        assert_eq!(1, *ptr);
    }
}
