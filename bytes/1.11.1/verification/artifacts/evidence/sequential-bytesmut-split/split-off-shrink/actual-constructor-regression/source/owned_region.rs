//! Pure resource-algebra model for disjoint owned byte regions.
//!
//! A value of [`OwnedRegion`] carries only a Creusot resource ledger. This
//! module does not connect that ledger to an allocation, pointer, or byte
//! access. The model-only constructor is available to proof code so the
//! algebra can be checked in isolation; physical binding must be established
//! by a separate reviewed bridge.

use creusot_std::{
    ghost::resource::Resource,
    logic::{FMap, Id, Int, ra::{RA, excl::Excl}},
    prelude::*,
};

/// Resource algebra shared by the owned-region and recovery ledgers.
///
/// The first component reserves the unique allocation-recovery marker. An
/// owned region contains the unit (`None`) there and owns only its interval in
/// the map component.
pub(crate) type KernelRA = (Option<Excl<()>>, FMap<Int, Excl<Option<u8>>>);

/// Slot-map component of [`KernelRA`].
pub(crate) type SlotMap = FMap<Int, Excl<Option<u8>>>;

/// Lift a pointwise compatible composition into equality of finite maps.
///
/// The pointwise premise is the key condition for `FMap::total_op`; extensional
/// equality then identifies its result with the expected whole map.
#[check(ghost)]
#[requires(forall<index: Int> (*left).get(index).op((*right).get(index)) == Some((*whole).get(index)))]
#[ensures((*left).op(*right) == Some(*whole))]
pub(crate) fn map_compose_eq(
    left: Snapshot<SlotMap>,
    right: Snapshot<SlotMap>,
    whole: Snapshot<SlotMap>,
) {
    proof_assert!(forall<index: Int> (*left).get(index).op((*right).get(index)) != None);
    let merged: Snapshot<SlotMap> = snapshot! { (*left).total_op(*right) };
    proof_assert!((*merged).ext_eq(*whole));
}

/// Recover pointwise lookup facts from a successful finite-map composition.
#[check(ghost)]
#[requires((*left).op(*right) == Some(*whole))]
#[ensures(forall<index: Int>
    (*left).get(index).op((*right).get(index)) == Some((*whole).get(index)))]
pub(crate) fn map_op_get(
    left: Snapshot<SlotMap>,
    right: Snapshot<SlotMap>,
    whole: Snapshot<SlotMap>,
) {
    let merged: Snapshot<SlotMap> = snapshot! { (*left).total_op(*right) };
    proof_assert!(forall<index: Int> (*left).get(index).op((*right).get(index)) != None);
    proof_assert!(forall<index: Int>
        Some((*merged).get(index)) == (*left).get(index).op((*right).get(index))
    );
    proof_assert!((*merged).ext_eq(*whole));
    proof_assert!(forall<index: Int> (*merged).get(index) == (*whole).get(index));
}

/// A resource-ledger fragment for one half-open interval of byte slots.
///
/// `None` in a slot means its byte value is not known by this model. It does
/// not claim that the physical byte is uninitialized.
#[allow(missing_debug_implementations)]
pub(crate) struct OwnedRegion {
    lo: Int,
    hi: Int,
    resource: Resource<KernelRA>,
}

impl View for OwnedRegion {
    type ViewTy = (Int, Int, Id, KernelRA);

    #[logic]
    fn view(self) -> Self::ViewTy {
        pearlite! { (self.lo, self.hi, self.resource.id(), self.resource@) }
    }
}

impl Invariant for OwnedRegion {
    #[logic(prophetic)]
    fn invariant(self) -> bool {
        pearlite! {
            self@.0 <= self@.1 &&
            self@.3.0 == None &&
            forall<index: Int> {
                self@.3.1.contains(index) ==
                    (self@.0 <= index && index < self@.1)
            }
        }
    }
}

impl OwnedRegion {
    /// Wrap a model resource as an interval ledger.
    ///
    /// This constructor establishes only the pure resource-algebra invariant.
    /// It does not bind the resource to an allocation or grant access to bytes.
    #[check(ghost)]
    #[requires(lo <= hi)]
    #[requires(resource@.0 == None)]
    #[requires(forall<index: Int> resource@.1.contains(index) == (lo <= index && index < hi))]
    #[ensures(result.lo() == lo && result.hi() == hi)]
    #[ensures(result@.3 == resource@)]
    #[ensures(result.resource_id() == resource.id())]
    #[ensures(result.invariant())]
    pub(crate) fn from_model_ledger(lo: Int, hi: Int, resource: Resource<KernelRA>) -> Self {
        Self { lo, hi, resource }
    }

    /// Consume this model ledger into its interval and underlying resource.
    ///
    /// This is a model-only extractor. It does not create or strengthen any
    /// physical binding; callers that need such a binding must already own a
    /// separately sealed physical capability.
    #[check(ghost)]
    #[requires(self.invariant())]
    #[ensures(result.0 == self.lo() && result.1 == self.hi())]
    #[ensures(result.2@ == self@.3)]
    #[ensures(result.2.id() == self.resource_id())]
    #[ensures(result.2@.0 == None)]
    #[ensures(forall<index: Int> result.2@.1.contains(index) ==
        (result.0 <= index && index < result.1))]
    pub(crate) fn into_parts(self) -> (Int, Int, Resource<KernelRA>) {
        (self.lo, self.hi, self.resource)
    }

    /// Lower endpoint of the interval.
    #[logic(open)]
    pub(crate) fn lo(self) -> Int {
        pearlite! { self@.0 }
    }

    /// Upper endpoint of the interval.
    #[logic(open)]
    pub(crate) fn hi(self) -> Int {
        pearlite! { self@.1 }
    }

    /// Resource identity shared by regions split from one ledger.
    #[logic(open)]
    pub(crate) fn resource_id(self) -> Id {
        pearlite! { self@.2 }
    }

    /// Read-only slot map used by the separate physical bridge.
    #[logic(open)]
    pub(crate) fn slots(self) -> FMap<Int, Excl<Option<u8>>> {
        pearlite! { self@.3.1 }
    }

    /// Slot model at an index. The outer option records whether the index is
    /// in this region; the inner option records whether a value is known.
    #[logic(open)]
    pub(crate) fn slot(self, index: Int) -> Option<Option<u8>> {
        pearlite! {
            match self@.3.1.get(index) {
                None => None,
                Some(Excl(value)) => Some(value),
            }
        }
    }

    /// Restrict a slot map to one half-open interval.
    #[logic(open)]
    fn restrict(
        map: FMap<Int, Excl<Option<u8>>>,
        lo: Int,
        hi: Int,
    ) -> FMap<Int, Excl<Option<u8>>> {
        map.filter(|(index, _)| lo <= index && index < hi)
    }

    /// Split the resource at an interior or endpoint boundary.
    ///
    /// The two returned resources retain the original resource identity. Their
    /// domains partition the original interval, and their slot values are
    /// unchanged.
    #[check(ghost)]
    #[requires(self.invariant())]
    #[requires(self.lo() <= at && at <= self.hi())]
    #[ensures(result.0.invariant() && result.1.invariant())]
    #[ensures(result.0.lo() == self.lo() && result.0.hi() == at)]
    #[ensures(result.1.lo() == at && result.1.hi() == self.hi())]
    #[ensures(result.0.resource_id() == self.resource_id())]
    #[ensures(result.1.resource_id() == self.resource_id())]
    #[ensures(forall<index: Int> result.0.slot(index) ==
        if self.lo() <= index && index < at { self.slot(index) } else { None })]
    #[ensures(forall<index: Int> result.1.slot(index) ==
        if at <= index && index < self.hi() { self.slot(index) } else { None })]
    pub(crate) fn split_at(self, at: Int) -> (Self, Self) {
        let left_slots: Snapshot<SlotMap> = snapshot! {
            Self::restrict(self.resource@.1, self.lo, at)
        };
        let right_slots: Snapshot<SlotMap> = snapshot! {
            Self::restrict(self.resource@.1, at, self.hi)
        };
        let whole_slots: Snapshot<SlotMap> = snapshot!(self.resource@.1);
        ghost! {
            proof_assert!(forall<index: Int>
                (*left_slots).get(index).op((*right_slots).get(index)) ==
                    Some((*whole_slots).get(index))
            );
            map_compose_eq(left_slots, right_slots, whole_slots);
        };
        let left_value: Snapshot<KernelRA> = snapshot! {
            (None, *left_slots)
        };
        let right_value: Snapshot<KernelRA> = snapshot! {
            (None, *right_slots)
        };

        proof_assert!((*left_value).op(*right_value) == Some(self.resource@));
        proof_assert!(KernelRA::incl_eq_op(*left_value, *right_value, self.resource@));

        let (left_resource, right_resource) = self.resource.split(left_value, right_value);
        (
            Self { lo: self.lo, hi: at, resource: left_resource },
            Self { lo: at, hi: self.hi, resource: right_resource },
        )
    }

    /// Rejoin contiguous regions with the same resource identity.
    ///
    /// This operation preserves the complete map, including `Unknown` slots.
    /// It performs no slot update and does not establish physical byte access.
    #[check(ghost)]
    #[requires(self.invariant() && other.invariant())]
    #[requires(self.resource_id() == other.resource_id())]
    #[requires(self.hi() == other.lo())]
    #[ensures(result.invariant())]
    #[ensures(result.lo() == self.lo() && result.hi() == other.hi())]
    #[ensures(result.resource_id() == self.resource_id())]
    #[ensures(forall<index: Int> result.slot(index) ==
        if self.lo() <= index && index < self.hi() {
            self.slot(index)
        } else if other.lo() <= index && index < other.hi() {
            other.slot(index)
        } else {
            None
        })]
    pub(crate) fn join(self, other: Self) -> Self {
        let lo = self.lo;
        let hi = other.hi;
        let resource = self.resource.join(other.resource);
        Self { lo, hi, resource }
    }
}
