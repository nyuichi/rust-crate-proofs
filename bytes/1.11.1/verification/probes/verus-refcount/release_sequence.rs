use vstd::prelude::*;

verus! {
// One refcount modification in modification order. `published` describes
// writes sequenced before a Release operation; it is visibility, not ownership.
pub struct Modification {
    pub rmw: bool,
    pub release: bool,
    pub before: nat,
    pub after: nat,
    pub published: Set<nat>,
}

pub open spec fn release_sequence(m: Seq<Modification>, head: int, tail: int) -> bool {
    0 <= head <= tail < m.len()
    && m[head].release
    && forall|j: int| head < j <= tail ==> m[j].rmw
}

// Operational accumulation of writes attached to still-live release heads.
// A Relaxed RMW preserves this set, but does NOT acquire it into its own view.
pub open spec fn carried(m: Seq<Modification>, end: nat) -> Set<nat>
    recommends end <= m.len(),
    decreases end,
{
    if end == 0 { Set::empty() }
    else {
        let e = m[end as int - 1];
        let previous = if e.rmw { carried(m, (end - 1) as nat) } else { Set::empty() };
        if e.release { previous.union(e.published) } else { previous }
    }
}

pub open spec fn acquired(m: Seq<Modification>, reads_from: int, acquire: bool) -> Set<nat> {
    if acquire && 0 <= reads_from < m.len() {
        carried(m, (reads_from + 1) as nat)
    } else { Set::empty() }
}

// The operational carry cannot gain unrelated visibility: exactly one of the
// release heads in the uninterrupted RMW suffix must have published a write.
pub proof fn carried_iff_release_sequence(m: Seq<Modification>, end: nat, write: nat)
    requires end <= m.len(),
    ensures carried(m, end).contains(write) <==>
        exists|h: int| release_sequence(m, h, end as int - 1) && m[h].published.contains(write),
    decreases end,
{
    if end > 0 {
        carried_iff_release_sequence(m, (end - 1) as nat, write);
        let last = end as int - 1;
        if carried(m, end).contains(write) {
            if m[last].release && m[last].published.contains(write) {
                assert(release_sequence(m, last, last));
            } else {
                let h = choose|h: int| release_sequence(m, h, last - 1) && m[h].published.contains(write);
                assert(release_sequence(m, h, last));
            }
        }
        if exists|h: int| release_sequence(m, h, last) && m[h].published.contains(write) {
            let h = choose|h: int| release_sequence(m, h, last) && m[h].published.contains(write);
            if h < last {
                assert(m[last].rmw);
                assert(release_sequence(m, h, last - 1));
            }
        }
    }
}

pub proof fn acquire_observes_release(m: Seq<Modification>, h: int, rf: int, write: nat)
    requires release_sequence(m, h, rf), m[h].published.contains(write),
    ensures acquired(m, rf, true).contains(write),
{
    carried_iff_release_sequence(m, (rf + 1) as nat, write);
}

pub open spec fn abc() -> Seq<Modification> {
    seq![
        Modification { rmw: true, release: true, before: 2, after: 1, published: set![10nat] },
        Modification { rmw: true, release: false, before: 1, after: 2, published: Set::empty() },
        Modification { rmw: true, release: true, before: 2, after: 1, published: set![20nat] },
        Modification { rmw: true, release: true, before: 1, after: 0, published: set![30nat] },
    ]
}

// A live handle authorizes every RMW, so no RMW starts at zero. Adjacent
// modifications obey the immediate-predecessor read rule for RMWs.
pub open spec fn refcount_trace(m: Seq<Modification>) -> bool {
    forall|j: int| 0 <= j < m.len() ==>
        m[j].rmw && m[j].before > 0
        && (m[j].after + 1 == m[j].before || m[j].after == m[j].before + 1)
        && (j > 0 ==> m[j].before == m[j - 1].after)
}

// Write-read coherence forces a load sequenced after the final RMW to read
// that modification or a later one. The no-resurrection premise rules out later
// modifications. These are model premises, not newly trusted atomic contracts.
pub proof fn final_acquire_reads_zero(m: Seq<Modification>, last: int, rf: int)
    requires refcount_trace(m), 0 <= last < m.len(), m[last].after == 0,
        last <= rf < m.len(),
    ensures last == m.len() - 1, rf == last, m[rf].after == 0,
{
    if last + 1 < m.len() {
        assert(m[last + 1].before == m[last].after);
        assert(m[last + 1].before > 0);
    }
}

pub proof fn abc_final_acquire()
    ensures
        acquired(abc(), 3, true).contains(10),
        acquired(abc(), 3, true).contains(20),
        acquired(abc(), 3, true).contains(30),
        !acquired(abc(), 3, false).contains(10),
{
    let m = abc();
    assert(refcount_trace(m));
    final_acquire_reads_zero(m, 3, 3);
    assert(release_sequence(m, 0, 3));
    assert(release_sequence(m, 2, 3));
    acquire_observes_release(m, 0, 3, 10);
    acquire_observes_release(m, 2, 3, 20);
    acquire_observes_release(m, 3, 3, 30);
}

#[cfg(negative_no_acquire)]
pub proof fn cannot_recover_without_acquire() {
    assert(acquired(abc(), 3, false).contains(10));
}

#[cfg(negative_broken_sequence)]
pub proof fn cannot_cross_plain_relaxed_store() {
    let m = abc().update(1, Modification {
        rmw: false, release: false, before: 1, after: 2, published: Set::empty(),
    });
    assert(acquired(m, 3, true).contains(10));
}
}

fn main() {}
