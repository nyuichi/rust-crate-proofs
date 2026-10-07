#![allow(unexpected_cfgs,unused_variables,dead_code)]
#![recursion_limit="512"]
use creusot_std::{prelude::*, ghost::{Perm,invariant::Protocol,resource::{Authority,Fragment}},
    logic::{Id,FMap,ra::{RA,excl::Excl}},
    std::sync::{atomic::{AtomicUsize as ModelAtomic,ordering::Relaxed},committer::Committer,
        view::{SyncView,ReleaseSyncView,HasTimestamp}}};
mod event;
use event::{EventAtomic,EventProtocol,RawAtomic};
type Registrations=FMap<Int,Excl<()>>;

#[logic(open(self))]
fn prefix(map:Registrations,n:Int)->bool {
    pearlite! {0<=n && (forall<i:Int> map.contains(i) == (0<=i && i<n))}
}
#[check(ghost)]
#[requires(prefix(auth@,*next))]
#[ensures(prefix((^auth)@,*next+1))]
#[ensures(result@ == FMap::singleton(*next,Excl(())))]
#[ensures(result.id() == auth.id() && (^auth).id() == auth.id())]
fn issue(auth:&mut Authority<Registrations>,next:Snapshot<Int>)->Fragment<Registrations> {
    let before=snapshot!(auth@);
    let one=snapshot!(FMap::singleton(*next,Excl(())));
    proof_assert!(forall<key:Int> (*before).get(key).op((*one).get(key)) != None);
    let result=auth.add_fragment(one);
    proof_assert!(auth@.ext_eq((*before).insert(*next,Excl(()))));
    result
}
struct State {own:Perm<ModelAtomic>, first:Int, latest:Int, issued:Authority<Registrations>, next:Int}
impl Protocol for State {
    type Public=(ModelAtomic,Id);
    #[logic] fn public(self)->Self::Public {(*self.own.ward(),self.issued.id())}
    #[logic(prophetic)] fn protocol(self)->bool {pearlite! {
        prefix(self.issued@,self.next) &&
        self.own.val().get(self.latest) != None &&
        self.own.val().get(self.latest).unwrap_logic().0@ == self.next % (usize::MAX@+1) &&
        self.first<=self.latest &&
        (forall<t:Int> self.own.val().get(t) != None == (self.first<=t && t<=self.latest))
    }}
}
impl EventProtocol for State {
    #[logic] fn atomic(self)->ModelAtomic {*self.own.ward()}
}
struct Ticket {id:Int,fragment:Fragment<Registrations>}
impl Ticket {
    #[logic(open(self))] fn valid(self,registration:Id)->bool {pearlite! {
        self.fragment.id()==registration && self.fragment@==FMap::singleton(self.id,Excl(()))
    }}
}
struct Registry {atomic:EventAtomic<State>}
impl Registry {
    #[logic(open(self))] fn registration(self)->Id {self.atomic.public().1}
    #[ensures(result.1.inner_logic().valid(result.0.registration()))]
    fn new()->(Self,Ghost<Ticket>) {
        let mut current=ghost! {SyncView::new().into_inner()};
        let (raw,own)=RawAtomic::new(1,current.borrow_mut());
        let mut authority=ghost! {Authority::<Registrations>::alloc().into_inner()};
        let ticket=ghost! {Ticket{id:0int,fragment:issue(&mut authority,snapshot!(0))}};
        let state=ghost! {let latest:Snapshot<Int>=snapshot!(raw.model().get_timestamp(*current)); State {own:own.into_inner(),first:latest.into_ghost().into_inner(),latest:latest.into_ghost().into_inner(),
            issued:authority.into_inner(),next:1int}};
        (Self{atomic:EventAtomic::bind(raw,state)},ticket)
    }

    /// Ordinary shared receiver; no Tokens or mutable source-ticket borrow.
    #[requires(source.valid(self.registration()))]
    #[ensures(result.1.inner_logic().valid(self.registration()))]
    #[ensures(result.1.inner_logic().id != source.id)]
    #[ensures(result.0@ == result.1.inner_logic().id % (usize::MAX@+1))]
    fn clone_registration(&self,source:Ghost<&Ticket>)->(usize,Ghost<Ticket>) {
        let mut output=ghost! {None::<Ticket>};
        let mut current=ghost! {SyncView::new().into_inner()};
        let release=ghost! {ReleaseSyncView::new().into_inner()};
        let old=self.atomic.increment(ghost! {|state:&mut State,c:&mut Committer<ModelAtomic,usize,Relaxed,Relaxed>| {
            state.issued.frag_lemma(&source.fragment);
            proof_assert!(state.issued@.contains(source.id));
            proof_assert!(source.id < state.next);
            let next=snapshot!(state.next);
            let fragment=issue(&mut state.issued,next);
            let previous=c.shoot_load(&state.own,&mut current);
            let before=snapshot!(c.val_load()@);
            c.shoot_store(&mut state.own,&mut current,release.into_inner());
            #[cfg(feature="negative_double_commit")]
            c.shoot_store(&mut state.own,&mut current,ReleaseSyncView::new().into_inner());
            proof_assert!(c.timestamp() == state.latest);
            let latest:Snapshot<Int>=snapshot!(c.timestamp()+1);
            state.latest=latest.into_ghost().into_inner();
            state.next=state.next+1int;
            *output=Some(Ticket{id:next.into_ghost().into_inner(),fragment});
        }});
        (old,ghost! {output.into_inner().unwrap()})
    }
}

pub fn shared_receiver_two_clones()->(usize,usize) {
    let (registry,first)=Registry::new();
    let (a,mut second)=registry.clone_registration(first.borrow());
    let (b,third)=registry.clone_registration(first.borrow());
    proof_assert!(first.id != second.id && first.id != third.id);
    ghost! { let registration:Snapshot<Id>=snapshot!(registry.registration()); distinct(&mut second,&third,registration.into_ghost().into_inner()); };
    proof_assert!(second.id != third.id);
    // All three actual affine fragments remain live. No reclamation claim.
    (a,b)
}

#[cfg(feature="negative_duplicate")]
fn duplicate_ticket(ticket:Ticket)->(Ticket,Ticket) {(ticket,ticket)}

#[cfg(feature="negative_reentry")]
#[requires(source.valid(registry.registration()))]
fn recursive_operation(registry:&Registry,source:Ghost<&Ticket>) {
    registry.atomic.increment(ghost! {|state:&mut State,c:&mut Committer<ModelAtomic,usize,Relaxed,Relaxed>| {
        let _=registry.clone_registration(source);
    }});
}

#[cfg(feature="negative_empty_commit")]
fn empty_commit(registry:&Registry) {
    registry.atomic.increment(ghost! {|state:&mut State,c:&mut Committer<ModelAtomic,usize,Relaxed,Relaxed>| {}});
}

#[cfg(feature="negative_wrong_ward")]
#[requires(!c.shot_store())]
#[requires(c.ward() != *other.ward())]
#[check(ghost)]
fn wrong_ward(c:&mut Committer<ModelAtomic,usize,Relaxed,Relaxed>,other:&mut Perm<ModelAtomic>,current:&mut SyncView,release:ReleaseSyncView) {
    c.shoot_store(other,current,release);
}

#[cfg(all(test,not(creusot)))]
mod tests {
    use super::*;
    #[test] fn native_shared_receiver(){
        for _ in 0..32 {assert_eq!(shared_receiver_two_clones(),(1,2));}
    }
    #[test] fn native_parallel_increments(){
        for _ in 0..32 {
            let (registry,first)=Registry::new();
            let results=std::thread::scope(|s|{
                let a=s.spawn(||registry.clone_registration(first.borrow()).0);
                let b=s.spawn(||registry.clone_registration(first.borrow()).0);
                (a.join().unwrap(),b.join().unwrap())
            });
            assert!(results==(1,2)||results==(2,1));
        }
    }
}

#[check(ghost)]
#[requires(a.valid(registration) && b.valid(registration))]
#[ensures(a.id != b.id)]
#[ensures(^a == *a)]
fn distinct(a:&mut Ticket,b:&Ticket,registration:Id) {
    a.fragment.0.valid_op_lemma(&b.fragment.0);
}
#[cfg(feature="negative_wrong_bind")]
#[requires(state.protocol() && state.atomic() != raw.model())]
fn wrong_bind(raw:RawAtomic,state:Ghost<State>) { let _=EventAtomic::bind(raw,state); }

/// Private diagnostic handle, not a replacement Bytes API or byte owner.
struct Registered<'a> {registry:&'a Registry,ticket:Ghost<Ticket>}
impl<'a> Registered<'a> {
    #[logic(open(self))] fn valid(self)->bool {self.ticket.valid(self.registry.registration())}
    #[requires(self.valid())]
    #[ensures(result.valid() && result.registry == self.registry)]
    #[ensures(result.ticket.id != self.ticket.id)]
    fn duplicate(&self)->Self {
        let (_,ticket)=self.registry.clone_registration(self.ticket.borrow());
        Self{registry:self.registry,ticket}
    }
}
fn require_sync<T:Sync>(_:&T) {}

pub fn duplicate_without_context() {
    let (registry,ticket)=Registry::new();
    require_sync(&registry.atomic);
    let first=Registered{registry:&registry,ticket};
    let second=first.duplicate();
    proof_assert!(first.ticket.id != second.ticket.id);
}
