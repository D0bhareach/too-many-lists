// simple example demonstrating that order of operation is matter in unsafe code.
#[test]
fn test_ptr_one() {
    unsafe {
        let mut data = 10;
        let ref1 = &mut data;
        let ptr1 = ref1 as *mut _;
        // try switch pointers and run
        // MIRIFLAGS="-Zmiri-tag-raw-pointers" cargo +nightly-2022-01-21 miri test
        *ptr1 += 2;
        *ref1 += 1;
        assert_eq!(data, 13);
    }
}

#[test]
fn test_ptr_two() {
    unsafe {
        let mut data = 10;
        // borrow  mutable in the scop it shall be the only one, but because we get raw pointers
        let ref1 = &mut data;

        // raw pointer of borrowed is like reborrowing again.
        let ptr2 = ref1 as *mut _;

        // we dereference ptr2 so we get data it means re3 and ref1 are the same.
        let ref3 = &mut *ptr2;

        // take pointer to borrowed pointer.
        let ptr4 = ref3 as *mut _;

        // Access the first raw pointer first if uncomment miri will complain because of
        // opereation orders read about stacked borrows in the book.
        // *ptr2 += 2;

        // Then access things in "borrow stack" order
        *ptr4 += 4;

        *ref3 += 3; // ptr4 is poped of the stack borrows
        *ptr2 += 2;
        *ref1 += 1;

        assert_eq!(data, 20);
    }
}

#[test]
fn test_arr_1() {
    unsafe {
        let mut data = [0; 4];
        let ref1_at_0 = &mut data[0]; // Reference to 0th element single because it's mut
        let ptr2_at_0 = ref1_at_0 as *mut i32; // Ptr to 0th element raw pointer

        // let ptr3_at_1 = ptr2_at_0.add(1); // Ptr to 1st element raw pointer

        // uncomment next line to see miri's error.
        // *ptr3_at_1 += 3;
        *ptr2_at_0 += 2;
        *ref1_at_0 += 1;

        // Should be [3, 3, 0, ...]
        // miri will complain, because it sees array as one chunk of memory if change ptrs to
        // point to one address like zero index it will pass.
        assert_eq!(data, [3, 0, 0, 0]);
    }
}

#[test]
fn test_arr_2() {
    unsafe {
        let mut data = [0; 4];
        let ref1_at_0 = &mut data[0]; // Reference to 0th element
        let ptr2_at_0 = ref1_at_0 as *mut i32; // Ptr to 0th element
        let ptr3_at_0 = ptr2_at_0; // Ptr to 0th element
        let ptr4_at_0 = ptr2_at_0.add(0); // Ptr to 0th element
        let ptr5_at_0 = ptr3_at_0.add(1).sub(1); // Ptr to 0th element

        // An absolute jumbled hash of ptr usages, order is not important
        // because they all are the same some of them are just simple equality and
        // some are result of computation, but pointers are integers so they all are the same.
        *ptr3_at_0 += 3;
        *ptr2_at_0 += 2;
        *ptr4_at_0 += 4;
        *ptr5_at_0 += 5;
        *ptr3_at_0 += 3;
        *ptr2_at_0 += 2;
        *ref1_at_0 += 1;

        // Should be [20, 0, 0, ...]
        assert_eq!(data, [20, 0, 0, 0]);
    }
}

// this test is mending the problem introduced in test_arr_1
// by splitting slice in two miri can distinguish between two parts of a slice.
// simply because it's two different locations now.
#[test]
fn test_arr_3() {
    unsafe {
        let mut data = [0; 4];

        let slice1 = &mut data[..];
        let (slice2_at_0, slice3_at_1) = slice1.split_at_mut(1);

        let ref4_at_0 = &mut slice2_at_0[0]; // Reference to 0th element
        let ref5_at_1 = &mut slice3_at_1[0]; // Reference to 1th element
        let ptr6_at_0 = ref4_at_0 as *mut i32; // Ptr to 0th element
        let ptr7_at_1 = ref5_at_1 as *mut i32; // Ptr to 1th element

        *ptr7_at_1 += 7;
        *ptr6_at_0 += 6;
        *ref5_at_1 += 5;
        *ref4_at_0 += 4;

        // Should be [10, 12, 0, ...]
        println!("{:?}", &data[..]);
    }
}

// usage of pointers and mut references to mutate array
// main question for me are we accessing and mutating array?
#[test]
fn test_slice_1() {
    unsafe {
        let mut data = [0; 6];

        let slice1_all = &mut data[..]; // Slice for the entire array
        let ptr2_all = slice1_all.as_mut_ptr(); // Pointer for the entire array

        let ptr3_at_0 = ptr2_all; // Pointer to 0th elem (the same)
        let ptr4_at_1 = ptr2_all.add(1); // Pointer to 1th elem
        let ref5_at_0 = &mut *ptr3_at_0; // Reference to 0th elem
        let ref6_at_1 = &mut *ptr4_at_1; // Reference to 1th elem

        *ref5_at_0 += 5;
        *ref5_at_0 += 5; // ok to use
        *ref6_at_1 += 6; // ok to use, because apparently it's different stacked borrow.
        *ptr4_at_1 += 4;
        // *ref6_at_1 += 6; // can not use it here because it pops off precedence is still matters!
        *ptr3_at_0 += 3;
        *ptr3_at_0 += 3; // ok to use

        // *ref5_at_0 += 5; // not ok to use
        // *ptr3_at_0 += 3; // not ok to use

        // Just for fun, modify all the elements in a loop
        // (Could use any of the raw pointers for this, they share a borrow!)
        for idx in 0..6 {
            *ptr2_all.add(idx) += idx;
        }

        // Safe version of this same code for fun
        for (idx, elem_ref) in slice1_all.iter_mut().enumerate() {
            *elem_ref += idx;
        }

        // Should be [8, 12, 4, 6, 8, 10, 12, 14, 16, 18]
        assert_eq!(&[16, 12, 4, 6, 8, 10,], &data[..]);
    }
}

// this is just to escape println macros optimization.
fn opaque_read(val: &i32) {
    println!("{}", val);
}
#[test]
fn test_shared_read() {
    unsafe {
        let mut data = 10;
        let mref1 = &mut data;
        let sref2 = &mref1;
        let sref3 = sref2;
        let sref4 = &*sref2;

        // Random hash of shared reference reads
        opaque_read(sref3);
        opaque_read(sref2);
        opaque_read(sref4);
        opaque_read(sref2);
        opaque_read(sref3);

        *mref1 += 1;

        opaque_read(&data);
        assert_eq!(data, 11);
    }
}

#[test]
fn test_shared_read_mut() {
    unsafe {
        let mut data = 10;
        let mref1 = &mut data;
        let ptr2 = mref1 as *mut i32;
        let sref3 = &mref1;
        // let ptr4 = sref3 as *mut i32;

        // *ptr4 += 4;
        opaque_read(sref3);
        *ptr2 += 2;
        *mref1 += 1;

        opaque_read(&data);
        assert_eq!(13, data);
    }
}

// Test inner mutability

#[test]
fn test_cell() {
    use std::cell::Cell;

    unsafe {
        let mut data = Cell::new(10);
        let mref1 = &mut data;
        let ptr2 = mref1 as *mut Cell<i32>;
        let sref3 = &*mref1;

        sref3.set(sref3.get() + 3);
        (*ptr2).set((*ptr2).get() + 2);
        mref1.set(mref1.get() + 1);

        assert_eq!(16, data.get());
    }
}

#[test]
fn test_unsafe_cell() {
    use std::cell::UnsafeCell;

    unsafe {
        let mut data = UnsafeCell::new(10);
        /* miri complains about reborrowing of mutable
        let mref1 = data.get_mut(); // Get a mutable ref to the contents
        let ptr2 = mref1 as *mut i32;
        let sref3 = &*ptr2;

        *ptr2 += 2;
        opaque_read(sref3);
        *mref1 += 1;

        */

        // this is working:
        let mref1 = &mut data; // Mutable ref to the *outside*
        let ptr2 = mref1.get(); // Get a raw pointer to the insides
        let sref3 = &*mref1; // Get a shared ref to the *outside*

        *ptr2 += 2; // Mutate with the raw pointer
        opaque_read(&*sref3.get()); // Read from the shared ref
        *sref3.get() += 3; // Write through the shared ref
        *mref1.get() += 1; // Mutate with the mutable ref

        assert_eq!(16, *data.get());

        // reorder pointers just to comply with stack borrows
        let mut data = UnsafeCell::new(10);
        let mref1 = &mut data;
        // These two are swapped so the borrows are *definitely* totally stacked
        let sref2 = &*mref1;
        // Derive the ptr from the shared ref to be super safe!
        let ptr3 = sref2.get();

        *ptr3 += 3;
        opaque_read(&*sref2.get());
        *sref2.get() += 2;
        *mref1.get() += 1;
        assert_eq!(16, *data.get());
    }
}

// this one is here just to show how get a pointer to a Box
#[test]
fn test_box() {
    unsafe {
        let mut data = Box::new(10);
        let ptr1 = (&mut *data) as *mut i32;

        // use Box from pointer and from normal usage
        *ptr1 += 1;
        *data += 10;

        assert_eq!(21, *data);
    }
}
