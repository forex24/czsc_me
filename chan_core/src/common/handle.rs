use std::ops::Deref;

/// Trait for types that can be indexed
pub trait Indexable {
    fn index(&self) -> usize;
}

/// Trait for types that have a parent relationship
pub trait HasParent {
    type Parent;
    fn parent(&self) -> Option<Handle<Self::Parent>>;
    fn set_parent(&mut self, parent: Handle<Self::Parent>);
}

/// Trait for converting types to handles
pub trait AsHandle {
    type Output;
    fn as_handle(&self) -> Self::Output;
}

/// Handle type for safe references to Vec elements
///
/// Vec扩容和收缩会导致Vec元素在内存中移动，从而直接引用元素会引发失效问题
/// Handle类型是为了解决Vec元素的引用问题
/// 通过Box可以保证Vec类型在堆上地址稳定的问题
/// 通过index索引可以确保元素引用稳定
///
/// 使用条件：
/// 1. 基于index的Vec必须是Append-only的Vec
/// 2. 一旦BoxedVec被drop，所有的handle都将失效，要确保Handle的生命周期小于BoxedVec生命周期
/// TODO: 今后考虑用Pin
#[derive(Debug, PartialEq, Eq)]
pub struct Handle<T> {
    ptr: *const Vec<T>, // * const 没有所有权/借用/生命周期,因此不会被Drop
    index: usize,
}

impl<T> Handle<T> {
    /// Creates a new Handle from a boxed vector and index
    /// 必须是boxed vec, 不能是vec,因为vec的地址可能被移动
    #[allow(clippy::borrowed_box)]
    pub fn new(boxed_vec: &Box<Vec<T>>, index: usize) -> Self {
        Self {
            ptr: &**boxed_vec,
            index,
        }
    }

    /// Updates the index of the handle
    pub fn update_index(&mut self, index: usize) {
        self.index = index;
    }

    /// Gets a reference to the underlying Vec
    #[inline(always)]
    fn get_vec_ref(&self) -> &Vec<T> {
        unsafe { &*self.ptr }
    }

    /// Gets a mutable reference to the underlying Vec
    #[allow(clippy::mut_from_ref)]
    #[inline(always)]
    fn get_vec_mut(&self) -> &mut Vec<T> {
        unsafe { &mut *self.ptr.cast_mut() }
    }

    /// Returns the current index
    #[inline(always)]
    pub fn index(&self) -> usize {
        self.index
    }

    /// Gets an immutable reference to the element
    #[inline(always)]
    pub fn to_ref(&self) -> &T {
        &self.get_vec_ref()[self.index]
    }

    /// Gets a mutable reference to the element
    #[inline(always)]
    pub fn as_mut(&self) -> &mut T {
        &mut self.get_vec_mut()[self.index]
    }

    /// Gets the next element's handle
    pub fn next(&self) -> Option<Handle<T>> {
        self.next_step_by(1)
    }

    /// Gets the previous element's handle
    pub fn prev(&self) -> Option<Handle<T>> {
        self.prev_step_by(1)
    }

    /// Gets a handle to an element n steps forward
    pub fn next_step_by(&self, step: usize) -> Option<Handle<T>> {
        let vec: &Vec<T> = self.get_vec_ref();
        if self.index + step >= vec.len() {
            None
        } else {
            Some(Self {
                ptr: self.ptr,
                index: self.index + step,
            })
        }
    }

    /// Gets a handle to an element n steps backward
    pub fn prev_step_by(&self, step: usize) -> Option<Handle<T>> {
        if step > self.index {
            None
        } else {
            Some(Self {
                ptr: self.ptr,
                index: self.index - step,
            })
        }
    }
}

// 如果T没有实现Clone,默认Handle也不会实现Clone
// 所以这里要强制实现Clone
#[allow(clippy::non_canonical_clone_impl)]
impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Self {
            ptr: self.ptr,
            index: self.index,
        }
    }
}

// 如果T没有实现Copy,默认Handle也不会实现Copy
// 所以这里要强制实现Copy
impl<T> Copy for Handle<T> {}

// 通过解引用使Handle变成智能指针
impl<T> Deref for Handle<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        let vec: &Vec<T> = self.get_vec_ref();
        &vec[self.index]
    }
}
