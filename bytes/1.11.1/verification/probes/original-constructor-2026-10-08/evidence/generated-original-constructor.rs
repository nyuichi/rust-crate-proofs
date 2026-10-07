// Generated from the guarded production snapshot.
pub struct BytesMut {
    ptr: NonNull<u8>,
    len: usize,
    cap: usize,
    data: *mut Shared,
}
struct Shared {
    vec: Vec<u8>,
    original_capacity_repr: usize,
    ref_count: AtomicUsize,
}
const _: [(); 0 - mem::align_of::<Shared>() % 2] = []; // Assert that the alignment of `Shared` is divisible by 2.
const KIND_ARC: usize = 0b0;
const KIND_VEC: usize = 0b1;
const KIND_MASK: usize = 0b1;
const MAX_ORIGINAL_CAPACITY_WIDTH: usize = 17;
const MIN_ORIGINAL_CAPACITY_WIDTH: usize = 10;
const ORIGINAL_CAPACITY_MASK: usize = 0b11100;
const ORIGINAL_CAPACITY_OFFSET: usize = 2;
const VEC_POS_OFFSET: usize = 5;
const MAX_VEC_POS: usize = usize::MAX >> VEC_POS_OFFSET;
const NOT_VEC_POS_MASK: usize = 0b11111;
#[cfg(target_pointer_width = "64")]
const PTR_WIDTH: usize = 64;

#[cfg(target_pointer_width = "32")]
const PTR_WIDTH: usize = 32;


impl BytesMut {
    #[inline]
    pub fn with_capacity(capacity: usize) -> BytesMut {
        BytesMut::from_vec(Vec::with_capacity(capacity))
    }
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }
    #[inline]
    pub fn capacity(&self) -> usize {
        self.cap
    }
    #[inline]
    pub(crate) fn from_vec(vec: Vec<u8>) -> BytesMut {
        let mut vec = ManuallyDrop::new(vec);
        let ptr = vptr(vec.as_mut_ptr());
        let len = vec.len();
        let cap = vec.capacity();

        let original_capacity_repr = original_capacity_to_repr(cap);
        let data = (original_capacity_repr << ORIGINAL_CAPACITY_OFFSET) | KIND_VEC;

        BytesMut {
            ptr,
            len,
            cap,
            data: invalid_ptr(data),
        }
    }
}
#[inline]
fn original_capacity_to_repr(cap: usize) -> usize {
    let width = PTR_WIDTH - ((cap >> MIN_ORIGINAL_CAPACITY_WIDTH).leading_zeros() as usize);
    cmp::min(
        width,
        MAX_ORIGINAL_CAPACITY_WIDTH - MIN_ORIGINAL_CAPACITY_WIDTH,
    )
}
#[inline]
fn vptr(ptr: *mut u8) -> NonNull<u8> {
    if cfg!(debug_assertions) {
        NonNull::new(ptr).expect("Vec pointer should be non-null")
    } else {
        unsafe { NonNull::new_unchecked(ptr) }
    }
}
#[inline]
fn invalid_ptr<T>(addr: usize) -> *mut T {
    let ptr = core::ptr::null_mut::<u8>().wrapping_add(addr);
    debug_assert_eq!(ptr as usize, addr);
    ptr.cast::<T>()
}
