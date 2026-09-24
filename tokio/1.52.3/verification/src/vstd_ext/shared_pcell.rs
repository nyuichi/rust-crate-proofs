use vstd::cell::pcell::{PCell, PointsTo};
use vstd::cell::CellId;
use vstd::prelude::*;

verus! {

/// A vstd-style representation of an `UnsafeCell<T>` whose physical cell and
/// linear permission are stored separately.  Unlike `PublishedCell`, this
/// permits a Tokio object to retain only `&self` while its protocol transfers
/// the permission between endpoints or an atomic invariant.
///
/// The implementation is entirely body-proved from vstd's `PCell`; it adds no
/// axiom beyond the already frozen UnsafeCell/physical-memory foundation.
pub struct SharedPCell<T> {
    cell: PCell<T>,
}

pub tracked struct SharedPCellPermission<T> {
    points_to: PointsTo<T>,
}

impl<T> SharedPCellPermission<T> {
    pub closed spec fn id(&self) -> CellId { self.points_to.id() }
    pub closed spec fn value(&self) -> T { *self.points_to.value() }
}

impl<T> SharedPCell<T> {
    pub closed spec fn id(&self) -> CellId { self.cell.id() }

    pub fn new(value: T) -> (result: (Self, Tracked<SharedPCellPermission<T>>))
        ensures
            result.1@.id() == result.0.id(),
            result.1@.value() == value,
        no_unwind
    {
        let (cell, Tracked(points_to)) = PCell::new(value);
        (SharedPCell { cell }, Tracked(SharedPCellPermission { points_to }))
    }

    pub fn replace(
        &self,
        Tracked(permission): Tracked<&mut SharedPCellPermission<T>>,
        value: T,
    ) -> (previous: T)
        requires old(permission).id() == self.id(),
        ensures
            previous == old(permission).value(),
            final(permission).id() == self.id(),
            final(permission).value() == value,
        no_unwind
    {
        self.cell.replace(Tracked(&mut permission.points_to), value)
    }

    pub fn borrow<'a>(
        &'a self,
        Tracked(permission): Tracked<&'a SharedPCellPermission<T>>,
    ) -> (value: &'a T)
        requires permission.id() == self.id(),
        ensures *value == permission.value(),
        no_unwind
    {
        self.cell.borrow(Tracked(&permission.points_to))
    }
}

pub fn verify_shared_reference_roundtrip(value: u64)
{
    let (cell, Tracked(mut permission)) = SharedPCell::new(None::<u64>);
    let previous = cell.replace(Tracked(&mut permission), Some(value));
    assert(previous.is_none());
    let observed = cell.borrow(Tracked(&permission));
    assert(*observed == Some(value));
    let taken = cell.replace(Tracked(&mut permission), None);
    assert(taken == Some(value));
    assert(permission.value().is_none());
}

} // verus!
