/// A cheaply cloneable and sliceable chunk of contiguous memory.
///
/// `Bytes` is an efficient container for storing and operating on contiguous
/// slices of memory. It is intended for use primarily in networking code, but
/// could have applications elsewhere as well.
///
/// `Bytes` values facilitate zero-copy network programming by allowing multiple
/// `Bytes` objects to point to the same underlying memory.
///
/// `Bytes` does not have a single implementation. It is an interface, whose
/// exact behavior is implemented through dynamic dispatch in several underlying
/// implementations of `Bytes`.
///
/// All `Bytes` implementations must fulfill the following requirements:
/// - They are cheaply cloneable and thereby shareable between an unlimited amount
///   of components, for example by modifying a reference count.
/// - Instances can be sliced to refer to a subset of the original buffer.
///
/// ```
/// use bytes::Bytes;
///
/// let mut mem = Bytes::from("Hello world");
/// let a = mem.slice(0..5);
///
/// assert_eq!(a, "Hello");
///
/// let b = mem.split_to(6);
///
/// assert_eq!(mem, "world");
/// assert_eq!(b, "Hello ");
/// ```
///
/// # Memory layout
///
/// The `Bytes` struct itself is fairly small, limited to 4 `usize` fields used
/// to track information about which segment of the underlying memory the
/// `Bytes` handle has access to.
///
/// `Bytes` keeps both a pointer to the shared state containing the full memory
/// slice and a pointer to the start of the region visible by the handle.
/// `Bytes` also tracks the length of its view into the memory.
///
/// # Sharing
///
/// `Bytes` contains a vtable, which allows implementations of `Bytes` to define
/// how sharing/cloning is implemented in detail.
/// When `Bytes::clone()` is called, `Bytes` will call the vtable function for
/// cloning the backing storage in order to share it behind multiple `Bytes`
/// instances.
///
/// For `Bytes` implementations which refer to constant memory (e.g. created
/// via `Bytes::from_static()`) the cloning implementation will be a no-op.
///
/// For `Bytes` implementations which point to a reference counted shared storage
/// (e.g. an `Arc<[u8]>`), sharing will be implemented by increasing the
/// reference count.
///
/// Due to this mechanism, multiple `Bytes` instances may point to the same
/// shared memory region.
/// Each `Bytes` instance can point to different sections within that
/// memory region, and `Bytes` instances may or may not have overlapping views
/// into the memory.
///
/// The following diagram visualizes a scenario where 2 `Bytes` instances make
/// use of an `Arc`-based backing storage, and provide access to different views:
///
/// ```text
///
///    Arc ptrs                   ┌─────────┐
///    ________________________ / │ Bytes 2 │
///   /                           └─────────┘
///  /          ┌───────────┐     |         |
/// |_________/ │  Bytes 1  │     |         |
/// |           └───────────┘     |         |
/// |           |           | ___/ data     | tail
/// |      data |      tail |/              |
/// v           v           v               v
/// ┌─────┬─────┬───────────┬───────────────┬─────┐
/// │ Arc │     │           │               │     │
/// └─────┴─────┴───────────┴───────────────┴─────┘
/// ```
pub struct Bytes {
    ptr: *const u8,
    len: usize,
    // inlined "trait object"
    data: AtomicPtr<()>,
    vtable: &'static Vtable,
    #[cfg(all(creusot, bytes_original_freeze_gate))]
    original_frozen: Option<OriginalFrozenProof>,
    #[cfg(all(creusot, bytes_original_shared_gate))]
    original_shared: creusot_std::prelude::Ghost<OriginalSharedProof>,
}
