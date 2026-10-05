use super::*;
use creusot_std::std::sync::{atomic::fence_acquire, view::AcquireSyncView};
use creusot_std::{ghost::{invariant::{AtomicInvariant,Protocol,Tokens,declare_namespace},resource::Resource},logic::{Id,ra::excl::Excl}};
declare_namespace! { PUBLICATION }
type Tickets=(Option<Excl<()>>,Option<Excl<()>>);
pub struct Ticket { resource:Resource<Tickets>, left:bool }
struct State<T> {
    own:Perm<ModelAtomic>, latest:Int, returned:Resource<Tickets>, expected:Snapshot<(T,T)>,
    left:Option<AtView<T>>,right:Option<AtView<T>>,withdrawn:bool,
}
#[logic]
fn count(r:Tickets)->usize { if r.0 == None {if r.1 == None {2usize} else {1usize}} else {if r.1 == None {1usize} else {0usize}} }
impl<T> Protocol for State<T> {
    type Public=(ModelAtomic,Id,(T,T));
    #[logic] fn public(self)->Self::Public { (*self.own.ward(),self.returned.id(),*self.expected) }
    #[logic] fn protocol(self)->bool { pearlite! {
        self.own.val().get(self.latest) != None &&
        self.own.val().get(self.latest).unwrap_logic().0 == count(self.returned@) &&
        (forall<t:Int> self.own.val().get(t) != None ==> t <= self.latest) &&
        (self.withdrawn ==> count(self.returned@) == 0usize) &&
        (if self.withdrawn {self.left == None && self.right == None} else {
            (self.left == None) == (self.returned@.0 == None) &&
            (self.right == None) == (self.returned@.1 == None) &&
            (self.left != None ==> self.left.unwrap_logic().val() == self.expected.0) &&
            (self.right != None ==> self.right.unwrap_logic().val() == self.expected.1) &&
            (self.left != None ==> self.left.unwrap_logic().view() <= self.own.val().get(self.latest).unwrap_logic().1) &&
            (self.right != None ==> self.right.unwrap_logic().view() <= self.own.val().get(self.latest).unwrap_logic().1)
        })
    } }
}
pub struct SharedRetirement<T> { atomic:NativeAtomic, invariant:Ghost<AtomicInvariant<State<T>>> }
impl<T> SharedRetirement<T> {
    #[logic] pub(crate) fn valid(self)->bool { self.invariant.public().0 == self.atomic.model() && self.invariant.namespace() == PUBLICATION() }
    #[logic] pub(crate) fn accepts(self,t:Ticket)->bool { pearlite! {
        t.resource.id() == self.invariant.public().1 &&
        t.resource@ == if t.left {(Some(Excl(())),None)} else {(None,Some(Excl(())))}
    } }

    #[logic] pub(crate) fn expected(self)->(T,T) { self.invariant.public().2 }
    #[logic] pub(crate) fn accepts_payload(self,t:Ticket,payload:T)->bool {
        pearlite! { self.accepts(t) && payload == if t.left {self.expected().0} else {self.expected().1} }
    }

    #[ensures(result.0.valid() && result.0.accepts(result.1.inner_logic()) && result.0.accepts(result.2.inner_logic()))]
    #[ensures(result.0.expected() == *expected)]
    #[ensures(result.0.accepts_payload(result.1.inner_logic(),expected.0))]
    #[ensures(result.0.accepts_payload(result.2.inner_logic(),expected.1))]
    pub fn new(expected:Snapshot<(T,T)>)->(Self,Ghost<Ticket>,Ghost<Ticket>) {
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
                returned:returned.into_inner(),expected,left:None,right:None,withdrawn:false};
            AtomicInvariant::new(Ghost::new(state),snapshot!(PUBLICATION())).into_inner()
        };
        (Self{atomic,invariant},left,right)
    }

    #[requires(self.valid() && self.accepts_payload(ticket.inner_logic(),resource.inner_logic()))]
    #[requires(tokens.contains(PUBLICATION()))]
    #[ensures(result.0 == (result.1.inner_logic() != None))]
    #[ensures(result.0 ==> result.1.inner_logic().unwrap_logic() == self.expected())]
    pub fn retire(&self,ticket:Ghost<Ticket>,resource:Ghost<T>,mut tokens:Ghost<Tokens>)->(bool,Ghost<Option<(T,T)>>) {
        let (mut current,sealed)=AtView::new(resource).split();
        let mut collected:Ghost<Option<(T,AtView<T>,bool)>>=ghost! {None};
        let mut pending:Ghost<Option<AcquireSyncView>>=ghost! {None};
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
            })
        } else { (false,ghost! {None}) }
    }
}
