#![cfg_attr(not(test), no_std)]

use core::mem::MaybeUninit;

pub struct ConstArray<T, const N: usize> {
    buf: [MaybeUninit<T>; N],
    len: usize,
}

impl<T, const N: usize> ConstArray<T, N> {
    #[must_use]
    pub const fn uninit() -> Self {
        Self {
            buf: [const { MaybeUninit::uninit() }; N],
            len: 0,
        }
    }

    /// Creates a new array from any size of passed array
    /// 
    /// # Example
    /// ```rust
    /// # use const_array::ConstArray;
    /// let arr = ConstArray::<u32, 16>::new([1, 2, 3]);
    /// 
    /// assert_eq!(arr.capacity(), 16);
    /// assert_eq!(arr.len(), 3);
    /// assert_eq!(arr.as_slice(), &[1, 2, 3]);
    /// ```
    pub const fn new<const W: usize>(array: [T; W]) -> Self {
        let mut buf = [const { MaybeUninit::uninit() }; N];
        let mut i = 0;

        // We already checked that W < N
        while i < W {
            // SAFETY: We are manually dropping the passed array, and ensuring that it is small enough to fit
            let value = unsafe { core::ptr::read(&raw const array[i]) };

            buf[i] = MaybeUninit::new(value);
            i += 1;
        }

        // We must forget the passed array, or else the values may be dropped multiple times
        core::mem::forget(array);

        Self {
            buf,
            len: W,
        }
    }

    /// Creates a new const array from the passed array.
    /// 
    /// # Example
    /// ```rust
    /// # use const_array::ConstArray;
    /// let arr = ConstArray::from_array([1u32, 2, 3, 4]);
    /// 
    /// assert_eq!(arr.capacity(), 4);
    /// assert_eq!(arr.len(), 4);
    /// assert_eq!(arr.as_slice(), &[1, 2, 3, 4]);
    /// ```
    pub const fn from_array(array: [T; N]) -> Self {
        Self::new(array)
    }

    /// Attemps to push an item to the front of this array.
    ///
    /// # Errors
    /// If the array is full, this function will Err and return the passed value
    ///
    /// # Example
    /// ```rust
    /// # use const_array::ConstArray;
    /// let mut arr = ConstArray::<u32, 16>::uninit();
    /// 
    /// arr.push_front(20);
    /// arr.push_front(10);
    /// 
    /// assert_eq!(arr.as_slice(), &[10, 20]);
    /// ```
    pub const fn push_front(&mut self, item: T) -> Result<(), T> {
        if self.is_full() {
            return Err(item);
        }

        // SAFETY: We are only moving known objects one to the right, so long as this this array is not full.
        unsafe {
            let ptr = self.buf.as_mut_ptr();
            core::ptr::copy(ptr, ptr.add(1), self.len());
        }

        self.buf[0] = MaybeUninit::new(item);
        self.len += 1;

        Ok(())
    }

    /// Attemps to push an item to the back of this array.
    ///
    /// # Errors
    /// If the array is full, this function will Err and return the passed value
    ///
    /// # Example
    /// ```rust
    /// # use const_array::ConstArray;
    /// let mut arr = ConstArray::<u32, 16>::uninit();
    ///
    /// arr.push_back(10);
    /// arr.push_back(20);
    ///
    /// assert_eq!(arr.as_slice(), &[10, 20]);
    /// ```
    pub const fn push_back(&mut self, item: T) -> Result<(), T> {
        if self.is_full() {
            return Err(item);
        }

        self.buf[self.len()] = MaybeUninit::new(item);
        self.len += 1;

        Ok(())
    }

    /// Attemps to pop the front item from this array.
    /// 
    /// # Example
    /// ```rust
    /// # use const_array::ConstArray;
    /// let mut arr = ConstArray::from_array([1u32, 2]);
    /// 
    /// assert_eq!(arr.pop_front(), Some(1));
    /// assert_eq!(arr.pop_front(), Some(2));
    /// assert_eq!(arr.pop_front(), None);
    /// # assert_eq!(arr.len(), 0);
    /// # assert_eq!(arr.capacity(), 2);
    /// # assert_eq!(arr.as_slice(), &[]);
    /// ```
    pub const fn pop_front(&mut self) -> Option<T> {
        match self.len() {
            0 => None,
            len => {
                // SAFETY: We know that this element is valid as the length is greater than 0
                let obj = unsafe { self.buf[0].assume_init_read() };
                self.len -= 1;

                // SAFETY: We simply shift all elements to the left by one
                unsafe {
                    let ptr = self.buf.as_mut_ptr();
                    core::ptr::copy(ptr.add(1), ptr, len - 1);
                }
                
                Some(obj)
            }
        }
    }

    ///  Attempts to pop the last item from this array.
    /// 
    /// # Example
    /// ```rust
    /// # use const_array::ConstArray;
    /// let mut arr = ConstArray::from_array([1u32, 2]);
    /// 
    /// assert_eq!(arr.pop_back(), Some(2));
    /// assert_eq!(arr.pop_back(), Some(1));
    /// assert_eq!(arr.pop_back(), None);
    /// # assert_eq!(arr.len(), 0);
    /// # assert_eq!(arr.capacity(), 2);
    /// # assert_eq!(arr.as_slice(), &[]);
    /// ```
    pub const fn pop_back(&mut self) -> Option<T> {
        match self.len() {
            0 => None,
            index => {
                // SAFETY: We move the memory back to the caller, and the decrement the length.
                let obj = unsafe { self.buf[index - 1].assume_init_read() };
                self.len -= 1;

                Some(obj)
            }
        }
    }

    /// Returns a slice that contains all initialized items
    ///
    /// The slice is guaranteed to be in order, from least to greatest
    ///
    /// # Example
    /// ```rust
    /// # use const_array::ConstArray;
    /// let mut arr = ConstArray::<u32, 16>::uninit();
    ///
    /// arr.push_back(5);
    /// arr.push_back(1);
    /// arr.push_back(3);
    ///
    /// assert_eq!(arr.as_slice(), &[5, 1, 3]);
    /// # assert_eq!(arr.len(), 3);
    /// ```
    #[must_use]
    pub const fn as_slice(&self) -> &[T] {
        // SAFETY: We uphold the invariant that `self.len` is initialized
        unsafe { core::slice::from_raw_parts(self.buf.as_ptr().cast::<T>(), self.len) }
    }

    /// Returns a mutable slice that contains all initialized items
    ///
    /// # Example
    /// ```rust
    /// # use const_array::ConstArray;
    /// let mut arr = ConstArray::from_array([0u32, 25]);
    ///
    /// arr.as_mut_slice()[0] = 100;
    ///
    /// assert_eq!(arr.as_slice(), &[100, 25]);
    /// # assert_eq!(arr.len(), 2);
    /// # assert_eq!(arr.capacity(), 2);
    /// ```
    #[must_use]
    pub const fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { core::slice::from_raw_parts_mut(self.buf.as_mut_ptr().cast::<T>(), self.len) }
    }

    /// Constructs a [`ConstArray`] from its raw parts.
    /// 
    /// # Safety
    /// It's required that the passed `len` is equivelant to the number of
    /// valid entries in the array.
    /// 
    /// # Example
    /// ```rust
    /// # use const_array::ConstArray;
    /// # use core::mem::MaybeUninit;
    /// let items = [
    ///     MaybeUninit::new(1u32),
    ///     MaybeUninit::new(2),
    ///     MaybeUninit::new(3),
    ///     MaybeUninit::uninit(),
    /// ];
    /// 
    /// let mut arr = unsafe { ConstArray::from_raw_parts(items, 3) };
    /// 
    /// assert_eq!(arr.as_slice(), &[1, 2, 3]);
    /// assert_eq!(arr.capacity(), items.len());
    /// # assert_eq!(arr.len(), 3);
    /// ```
    pub const unsafe fn from_raw_parts(array: [MaybeUninit<T>; N], len: usize) -> Self {
        Self {
            buf: array,
            len
        }
    }

    /// Decomposes the [`ConstArray`] into it's components (array, len)
    pub const fn into_raw_parts(self) -> ([MaybeUninit<T>; N], usize) {
        // SAFETY: To achieve this in a const-context, we must read the pointer without using ManuallyDrop
        let buf = unsafe { core::ptr::read(&raw const self.buf) };
        let len = self.len;

        // We forget self to prevent the calling of Drop
        core::mem::forget(self);

        (buf, len)
    }

    /// Converts the [`ConstArray`] to a standard [`[MaybeUninit<T>; N]`]
    pub const fn to_array(self) -> [MaybeUninit<T>; N] {
        // SAFETY: We invalidate the memory, and then forget about it immediately after.
        let buf = unsafe { core::ptr::read(&raw const self.buf) };

        // We drop self as we do not have a destructor for this method, only for the array
        core::mem::forget(self);

        buf
    }

    /// Returns the number of used entries in the array.
    ///
    /// For the capacity, see [`ConstArray::capacity()`]
    ///
    /// # Example
    /// ```rust
    /// # use const_array::ConstArray;
    /// let mut arr = ConstArray::<u32, 16>::uninit();
    ///
    /// arr.push_back(1);
    /// arr.push_back(2);
    /// arr.push_back(3);
    ///
    /// assert_eq!(arr.len(), 3);
    /// # assert_eq!(arr.as_slice(), &[1, 2, 3]);
    /// ```
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Returns the total capacity of the array.
    ///
    /// # Example
    /// ```rust
    /// # use const_array::ConstArray;
    /// let arr = ConstArray::<u32, 16>::uninit();
    ///
    /// assert_eq!(arr.capacity(), 16);
    /// ```
    pub const fn capacity(&self) -> usize {
        N
    }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub const fn is_full(&self) -> bool {
        self.len == N
    }

    pub const fn get(&self, index: usize) -> Option<&T> {
        if index <= self.len() {
            // SAFETY: Since index <= self.len(), only valid items can be retrieved
            Some(unsafe { self.buf[index].assume_init_ref() })
        } else {
            None
        }
    }
}

/// Const-drop semantics are currently to be considered.
impl<T, const N: usize> Drop for ConstArray<T, N> {
    fn drop(&mut self) {
        for i in 0..self.len() {
            unsafe { self.buf[i].assume_init_drop() }
        }
        
        self.len = 0;
    }
}

impl<T, const N: usize> Clone for ConstArray<T, N>
where T: Clone {
    fn clone(&self) -> Self {
        let mut new_arr = ConstArray::<T, N>::uninit();

        for i in 0..self.len() {
            let _ = new_arr.push_back(new_arr.get(i).unwrap().clone());
        }

        new_arr
    }
}

#[cfg(test)]
mod tests {}
