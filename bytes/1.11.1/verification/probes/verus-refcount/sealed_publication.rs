#![cfg_attr(verus_keep_ghost, verifier::exec_allows_no_decreases_clause)]
use vstd::prelude::*;
#[path = "release_sequence.rs"] mod model;

mod publication {
use super::*;
use vstd::invariant::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use verus_state_machines_macros::tokenized_state_machine;
verus! {
// Native primitives and their #[atomic] attributes are PRIVATE. Clients cannot
// use them to open an invariant containing arbitrary physical permissions.
#[verifier::external_body]
struct Native { value: AtomicUsize }
#[verifier::external_body]
tracked struct Permission { no_copy: NoCopy }
impl Permission {
    uninterp spec fn id(self) -> int;
    uninterp spec fn value(self) -> usize;
    uninterp spec fn history(self) -> Seq<model::Modification>;
}
tracked struct ReleaseStamp { ghost cell: int, ghost event: nat }
pub tracked struct AcquireView { ghost cell: int, ghost seen: Set<nat> }
pub tracked struct Released<R> { tracked resource: R, ghost cell: int, ghost event: nat }
impl Native {
    uninterp spec fn id(&self) -> int;
    #[verifier::external_body]
    fn new(n: usize) -> (r:(Self,Tracked<Permission>))
        ensures r.1@.id()==r.0.id(), r.1@.value()==n, r.1@.history()==Seq::empty(),
    { (Self{value:AtomicUsize::new(n)},Tracked::assume_new()) }
    #[verifier::atomic]
    #[verifier::external_body]
    fn release(&self, Tracked(p):Tracked<&mut Permission>) -> (r:(usize,Tracked<ReleaseStamp>))
        requires old(p).id()==self.id(), old(p).value()>0,
        ensures r.0==old(p).value(), final(p).id()==self.id(), final(p).value()+1==r.0,
            final(p).history()==old(p).history().push(model::Modification{
                rmw:true,release:true,before:r.0 as nat,after:(r.0-1) as nat,
                published:set![old(p).history().len()],
            }),
            r.1@.cell==self.id(),r.1@.event==old(p).history().len(),
        opens_invariants none
        no_unwind
    { (self.value.fetch_sub(1,Ordering::Release),Tracked::assume_new()) }
    // Generic final-RMW/read coherence rule. The stamp was produced by this
    // thread's preceding Release operation; the final-zero/history premises
    // prohibit a later modification. This contract adds memory-semantics TCB.
    #[verifier::atomic]
    #[verifier::external_body]
    fn final_acquire(&self, Tracked(p):Tracked<&Permission>, Tracked(stamp):Tracked<&ReleaseStamp>) -> (r:(usize,Tracked<AcquireView>))
        requires p.id()==self.id(),stamp.cell==self.id(),p.value()==0,
            stamp.event+1==p.history().len(),model::refcount_trace(p.history()),
            p.history()[stamp.event as int].after==0,
        ensures r.0==0,r.1@.cell==self.id(),r.1@.seen==model::carried(p.history(),p.history().len()),
        opens_invariants none
        no_unwind
    { (self.value.load(Ordering::Acquire),Tracked::assume_new()) }
}

impl<R> Released<R> {
    pub closed spec fn visible(self, view:AcquireView)->bool { self.cell==view.cell && view.seen.contains(self.event) }
    pub proof fn unseal(tracked self, tracked view:&AcquireView)->(tracked r:R)
        requires self.visible(*view),
    { self.resource }
}

tokenized_state_machine!(Tickets {
    fields {
        #[sharding(variable)] pub status:(nat,Set<nat>),
        #[sharding(set)] pub pending:Set<nat>,
        #[sharding(persistent_set)] pub completed:Set<nat>,
    }
    #[invariant] pub fn partition(self)->bool {
        self.status.0==self.pending.len()
        && self.pending.disjoint(self.status.1)
        && self.pending.union(self.status.1)==set![0nat,1nat]
        && (self.completed.contains(0) ==> self.status.0==0)
    }
    init! { initialize() {
        init status=(2,Set::empty());
        init pending=Set::empty().insert(0nat).insert(1nat);
        init completed=Set::empty();
    } }
    transition! { retire_first(id:nat) {
        require(pre.status.0==2);
        remove pending -= set {id};
        assert(pre.status.0>0);
        assert(!pre.status.1.contains(id));
        update status=((pre.status.0-1) as nat,pre.status.1.insert(id));

    } }
    transition! { retire_last(id:nat) {
        require(pre.status.0==1);
        remove pending -= set {id};
        assert(!pre.status.1.contains(id));
        update status=(0,pre.status.1.insert(id));
        add completed (union)= set {0nat};
    } }
    property! { live(id:nat) {
        have pending >= set {id};
        assert(pre.status.0>0);
        assert(!pre.status.1.contains(id));
    } }
    property! { finished() {
        have completed >= set {0nat};
        assert(pre.status.0==0);
    } }
    #[inductive(initialize)] fn initialize_inductive(post:Self) {}
    #[inductive(retire_first)] fn retire_first_inductive(pre:Self,post:Self,id:nat) {
        assert(post.pending.union(post.status.1) =~= set![0nat,1nat]);
    }
    #[inductive(retire_last)] fn retire_last_inductive(pre:Self,post:Self,id:nat) {
        assert(post.pending.union(post.status.1) =~= set![0nat,1nat]);
    }
});

tracked struct State<R> {
    tracked permission:Permission,
    tracked status:Tickets::status,
    tracked pool:Map<nat,Released<R>>,
}
struct Pred<R> { r: R }
impl<R> InvariantPredicate<(int,Tickets::Instance),State<R>> for Pred<R> {
    closed spec fn inv(k:(int,Tickets::Instance),s:State<R>)->bool {
        &&& s.permission.id()==k.0
        &&& s.status.instance_id()==k.1.id()
        &&& s.permission.value() as nat==s.status.value().0
        &&& s.permission.history().len()+s.status.value().0==2
        &&& s.pool.dom()==s.status.value().1
        &&& model::refcount_trace(s.permission.history())
        &&& forall|i:int| 0<=i<s.permission.history().len() ==>
            s.permission.history()[i].release && s.permission.history()[i].published==set![i as nat]
            && s.permission.history()[i].before==2-i
            && s.permission.history()[i].after==1-i
        &&& forall|id:nat| s.pool.dom().contains(id) ==>
            s.pool[id].cell==k.0 && s.pool[id].event<s.permission.history().len()
    }
}

pub struct Counter<R> {
    atomic: Native,
    inv: Tracked<AtomicInvariant<(int,Tickets::Instance),State<R>,Pred<R>>>,
}
pub tracked struct Ticket { tracked inner:Tickets::pending }
impl<R> Counter<R> {
    pub closed spec fn wf(&self)->bool { self.inv@.constant().0==self.atomic.id() }
    pub closed spec fn accepts(&self,t:Ticket)->bool { t.inner.instance_id()==self.inv@.constant().1.id() }
    pub fn new()->(r:(Self,Tracked<Ticket>,Tracked<Ticket>))
        ensures r.0.wf(),r.0.accepts(r.1@),r.0.accepts(r.2@),
    {
        let (atomic,Tracked(permission))=Native::new(2);
        let tracked (Tracked(inst),Tracked(status),Tracked(mut pending),_) = Tickets::Instance::initialize();
        let tracked first=Ticket{inner:pending.remove(0)};
        let tracked second=Ticket{inner:pending.remove(1)};
        let tracked state=State{permission,status,pool:Map::tracked_empty()};
        let tracked inv=AtomicInvariant::new((atomic.id(),inst),state,0);
        (Self{atomic,inv:Tracked(inv)},Tracked(first),Tracked(second))
    }
}
}
}
fn main() {}
