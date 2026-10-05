use vstd::prelude::*;

verus! {
// R is an affine Verus resource, e.g. vstd::cell::PointsTo<T>. Ghost expected
// values do not own any R; only the tracked retired map can return actual R.
tracked struct Escrow<R> {
    ghost expected: Map<nat, R>,
    ghost pending: Set<nat>,
    tracked retired: Map<nat, R>,
}

impl<R> Escrow<R> {
    spec fn wf(self) -> bool {
        &&& self.pending.subset_of(self.expected.dom())
        &&& self.retired.dom() == self.expected.dom().difference(self.pending)
        &&& forall|id: nat| self.retired.dom().contains(id) ==> self.retired[id] == self.expected[id]
    }

    proof fn new(expected: Map<nat, R>) -> (tracked e: Self)
        ensures e.wf(), e.expected == expected, e.pending == expected.dom(),
    {
        let tracked e = Self { expected, pending: expected.dom(), retired: Map::tracked_empty() };
        assert(e.retired.dom() =~= expected.dom().difference(e.pending));
        e
    }

    proof fn retire(tracked &mut self, id: nat, tracked resource: R)
        requires old(self).wf(), old(self).pending.contains(id), resource == old(self).expected[id],
        ensures final(self).wf(), final(self).expected == old(self).expected,
            final(self).pending == old(self).pending.remove(id),
            final(self).pending.len() + 1 == old(self).pending.len(),
            final(self).retired == old(self).retired.insert(id, resource),
    {
        self.pending = self.pending.remove(id);
        self.retired.tracked_insert(id, resource);
        assert(self.retired.dom() =~= self.expected.dom().difference(self.pending));
    }

    proof fn finish(tracked self) -> (tracked resources: Map<nat, R>)
        requires self.wf(), self.pending.len() == 0,
        ensures resources == self.expected,
    {
        assert(self.pending =~= Set::<nat>::empty());
        assert(self.retired =~= self.expected);
        self.retired
    }
}

// Instantiate R with an actual PCell permission, not a model-only byte label.
fn two_physical_cells() {
    let (a, Tracked(pa)) = vstd::cell::pcell::PCell::new(10u64);
    let (b, Tracked(pb)) = vstd::cell::pcell::PCell::new(20u64);
    let tracked mut e = Escrow::new(map![0nat => pa, 1nat => pb]);
    proof {
        e.retire(0, pa);
        assert(e.pending.len() == 1);
        e.retire(1, pb);
    }
    let tracked mut all = e.finish();
    let tracked pa = all.tracked_remove(0);
    let tracked pb = all.tracked_remove(1);
    let x = a.into_inner(Tracked(pa));
    let y = b.into_inner(Tracked(pb));
    assert(x == 10 && y == 20);
}

#[cfg(negative_missing_ticket)]
fn cannot_finish_with_empty_ticket() {
    let (a, Tracked(pa)) = vstd::cell::pcell::PCell::new(());
    let (empty, Tracked(pe)) = vstd::cell::pcell::PCell::new(());
    // A unit resource still represents a live ticket even with zero payload bytes.
    let tracked mut e = Escrow::new(map![0nat => pa, 1nat => pe]);
    proof { e.retire(0, pa); }
    let tracked all = e.finish();
}
}
fn main() { two_physical_cells(); }
