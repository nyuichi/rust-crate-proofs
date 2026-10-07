//! Two-reader weak retirement with body-proved completion receipts.
//! T02 objective metadata/AtView protocol plus authoritative receipt conservation.
//! The native atomic primitive is unchanged; no protocol theorem is trusted.
use super::*;
use creusot_std::std::sync::view::AcquireSyncView;
use super::primitive::fence_acquire;
use creusot_std::{ghost::{invariant::{AtomicInvariant,Protocol,Tokens,declare_namespace},resource::{Resource,Authority,Fragment}},logic::{Id,ra::{excl::Excl,agree::Ag}}};
/// Implementations define a projection; no law or protocol fact is assumed.
pub trait Payload {
    type Metadata;
    #[logic] fn metadata(self)->Self::Metadata;
    #[logic(prophetic)] fn wellformed(self)->bool;
}
declare_namespace! { PUBLICATION }
type Tickets=(Option<Excl<()>>,Option<Excl<()>>);
pub struct Ticket { resource:Resource<Tickets>, left:bool }
impl Ticket {
    #[logic]
    pub(crate) fn is_left(self)->bool { self.left }
}
pub(crate) struct Receipt { evidence:Fragment<Option<Ag<bool>>>, left:bool }
impl Receipt {
    #[logic] pub(crate) fn is_left(self)->bool { self.left }
}
struct State<T:Payload> {
    own:Perm<ModelAtomic>, latest:Int, returned:Resource<Tickets>, expected:Snapshot<(T::Metadata,T::Metadata)>,
    left:Option<AtView<T>>,right:Option<AtView<T>>,withdrawn:bool,
    completed_left:Authority<Option<Ag<bool>>>, completed_right:Authority<Option<Ag<bool>>>,
}
#[logic]
fn count(r:Tickets)->usize { if r.0 == None {if r.1 == None {2usize} else {1usize}} else {if r.1 == None {1usize} else {0usize}} }
impl<T:Payload> Protocol for State<T> {
    type Public=(ModelAtomic,Id,(T::Metadata,T::Metadata),Id,Id);
    #[logic] fn public(self)->Self::Public { (*self.own.ward(),self.returned.id(),*self.expected,self.completed_left.id(),self.completed_right.id()) }
    #[logic(prophetic)] fn protocol(self)->bool { pearlite! {
        self.own.val().get(self.latest) != None &&
        self.own.val().get(self.latest).unwrap_logic().0 == count(self.returned@) &&
        (forall<t:Int> self.own.val().get(t) != None ==> t <= self.latest) &&
        (self.withdrawn == (count(self.returned@) == 0usize)) &&
        (self.completed_left@ == None) == (self.returned@.0 == None) &&
        (self.completed_right@ == None) == (self.returned@.1 == None) &&
        (match (self.completed_left@,self.completed_right@) {
            (Some(l),Some(r)) => l.0 != r.0,
            (Some(l),None) => !l.0,
            (None,Some(r)) => !r.0,
            (None,None) => true
        }) &&
        (if self.withdrawn {self.left == None && self.right == None} else {
            (self.left == None) == (self.returned@.0 == None) &&
            (self.right == None) == (self.returned@.1 == None) &&
            (self.left != None ==> self.left.unwrap_logic().val().wellformed() && self.left.unwrap_logic().val().metadata() == self.expected.0) &&
            (self.right != None ==> self.right.unwrap_logic().val().wellformed() && self.right.unwrap_logic().val().metadata() == self.expected.1) &&
            (self.left != None ==> self.left.unwrap_logic().view() <= self.own.val().get(self.latest).unwrap_logic().1) &&
            (self.right != None ==> self.right.unwrap_logic().view() <= self.own.val().get(self.latest).unwrap_logic().1)
        })
    } }
}
pub struct SharedRetirement<T:Payload> { atomic:NativeAtomic, invariant:Ghost<AtomicInvariant<State<T>>> }
impl<T:Payload> SharedRetirement<T> {
    #[logic] pub(crate) fn valid(self)->bool { self.invariant.public().0 == self.atomic.model() && self.invariant.namespace() == PUBLICATION() }
    #[logic] pub(crate) fn accepts(self,t:Ticket)->bool { pearlite! {
        t.resource.id() == self.invariant.public().1 &&
        t.resource@ == if t.left {(Some(Excl(())),None)} else {(None,Some(Excl(())))}
    } }

    #[logic] pub(crate) fn expected(self)->(T::Metadata,T::Metadata) { self.invariant.public().2 }
    #[logic(open(crate), prophetic)] pub(crate) fn accepts_payload(self,t:Ticket,payload:T)->bool {
        pearlite! { self.accepts(t) && payload.wellformed() && payload.metadata() == if t.is_left() {self.expected().0} else {self.expected().1} }
    }

    #[logic]
    pub(crate) fn accepts_receipt(self,receipt:Receipt,last:bool)->bool {
        pearlite! { receipt.evidence.id() == if receipt.is_left() {self.invariant.public().3} else {self.invariant.public().4} &&
            receipt.evidence@ == Some(Ag(last)) }
    }

    #[ensures(result.0.valid() && result.0.accepts(result.1.inner_logic()) && result.0.accepts(result.2.inner_logic()))]
    #[ensures(result.0.expected() == *expected)]
    #[ensures(result.1.inner_logic().is_left() && !result.2.inner_logic().is_left())]
    pub fn new(expected:Snapshot<(T::Metadata,T::Metadata)>)->(Self,Ghost<Ticket>,Ghost<Ticket>) {
        let mut view=ghost! {SyncView::new().into_inner()};
        let (atomic,own)=NativeAtomic::new(2,view.borrow_mut());
        let tickets=ghost! {
            let all=Resource::alloc(snapshot!((Some(Excl(())),Some(Excl(()))))).into_inner();
            let (left,right)=all.split(snapshot!((Some(Excl(())),None)),snapshot!((None,Some(Excl(())))));
            let (left,returned)=left.split(snapshot!((Some(Excl(())),None)),snapshot!((None,None)));
            (Ticket{resource:left,left:true},Ticket{resource:right,left:false},returned)
        };
        let (left,rest)=ghost! {let (l,r,u)=tickets.into_inner(); (l,(r,u))}.split();
        let (right,returned)=rest.split();
        let invariant=ghost! {
            let latest:Snapshot<Int> = snapshot!(atomic.model().get_timestamp(*view));
            let state=State{own:own.into_inner(),latest:latest.into_ghost().into_inner(),
                returned:returned.into_inner(),expected,left:None,right:None,withdrawn:false,
                completed_left:Authority::alloc().into_inner(),completed_right:Authority::alloc().into_inner()};
            AtomicInvariant::new(Ghost::new(state),snapshot!(PUBLICATION())).into_inner()
        };
        (Self{atomic,invariant},left,right)
    }

    #[requires(self.valid() && self.accepts_payload(ticket.inner_logic(),resource.inner_logic()))]
    #[requires(tokens.contains(PUBLICATION()))]
    #[ensures(result.0 == (result.1.inner_logic() != None))]
    #[ensures(result.0 ==> result.1.inner_logic().unwrap_logic().0.wellformed() && result.1.inner_logic().unwrap_logic().1.wellformed())]
    #[ensures(result.0 ==> (result.1.inner_logic().unwrap_logic().0.metadata(),result.1.inner_logic().unwrap_logic().1.metadata()) == self.expected())]
    #[ensures(self.accepts_receipt(result.2.inner_logic(),result.0))]
    #[ensures(result.2.inner_logic().is_left() == ticket.inner_logic().is_left())]
    pub fn retire(&self,ticket:Ghost<Ticket>,resource:Ghost<T>,mut tokens:Ghost<Tokens>)->(bool,Ghost<Option<(T,T)>>,Ghost<Receipt>) {
        let (mut current,sealed)=AtView::new(resource).split();
        let mut collected:Ghost<Option<(T,AtView<T>,bool)>>=ghost! {None};
        let mut pending:Ghost<Option<AcquireSyncView>>=ghost! {None};
        let mut receipt:Ghost<Option<Receipt>>=ghost! {None};
        let old=self.atomic.decrement(ghost! {|c:&mut Committer<ModelAtomic,usize,Relaxed,Release>| {
            self.invariant.open(tokens.reborrow(), |state:&mut State<T>| {
                let ticket=ticket.into_inner();
                let before=snapshot!(state.returned@);
                state.returned.join_in(ticket.resource);
                proof_assert!(count(*before)>0usize);
                // A relaxed RMW read carries the predecessor's release view
                // as a deferred acquire witness; it does not acquire it yet.
                let previous=c.shoot_load(&state.own,&mut current);
                let published=primitive::release_rmw(c,&mut state.own,&mut current);
                proof_assert!(c.timestamp() == state.latest);
                proof_assert!(c.val_load() == count(*before));
                proof_assert!(!state.withdrawn);
                let next:Snapshot<Int> = snapshot!(c.timestamp()+1);
                state.latest=next.into_ghost().into_inner();
                if ticket.left {state.left=Some(sealed.into_inner());} else {state.right=Some(sealed.into_inner());}
                let last:Snapshot<bool> = snapshot!(c.val_load() == 1usize);
                let evidence=if ticket.left {
                    state.completed_left.add_fragment(snapshot!(Some(Ag(*last))))
                } else {
                    state.completed_right.add_fragment(snapshot!(Some(Ag(*last))))
                };
                *receipt=Some(Receipt{evidence,left:ticket.left});
                if last.into_ghost().into_inner() {
                    let left=state.left.take().unwrap();
                    let right=state.right.take().unwrap();
                    if ticket.left {
                        proof_assert!(right.view() <= previous@);
                        *collected=Some((left.sync(*current),right,true));
                    } else {
                        proof_assert!(left.view() <= previous@);
                        *collected=Some((right.sync(*current),left,false));
                    }
                    *pending=Some(previous);
                    state.withdrawn=true;
                }
            });
        }});
        if old == 1 {
            #[cfg(not(feature="negative_no_acquire"))]
            let observed=fence_acquire(ghost! {pending.into_inner().unwrap()});
            #[cfg(feature="negative_no_acquire")]
            let observed=current;
            (true,ghost! {
                let (local,other,left)=collected.into_inner().unwrap();
                let other=other.sync(*observed);
                Some(if left {(local,other)} else {(other,local)})
            },ghost! {receipt.into_inner().unwrap()})
        } else { (false,ghost! {None},ghost! {receipt.into_inner().unwrap()}) }
    }

    #[requires(self.valid())]
    #[requires(self.accepts_receipt(first.inner_logic(),first_last))]
    #[requires(self.accepts_receipt(second.inner_logic(),second_last))]
    #[requires(first.inner_logic().is_left() != second.inner_logic().is_left())]
    #[ensures(first_last != second_last)]
    pub(crate) fn finish(self,first_last:bool,first:Ghost<Receipt>,second_last:bool,second:Ghost<Receipt>) {
        ghost! {
            let state=self.invariant.into_inner().into_inner();
            let first=first.into_inner();let second=second.into_inner();
            if first.left {
                state.completed_left.frag_lemma(&first.evidence);
                state.completed_right.frag_lemma(&second.evidence);
            } else {
                state.completed_right.frag_lemma(&first.evidence);
                state.completed_left.frag_lemma(&second.evidence);
            }
            proof_assert!(state.withdrawn);
            proof_assert!(first_last != second_last);
        };
    }

}
