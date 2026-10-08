use super::*;

use creusot_std::{ghost::{invariant::Protocol,resource::{Resource,Authority,Fragment}},logic::{Id,ra::{excl::Excl,agree::Ag}}};
/// Implementations define a projection; no law or protocol fact is assumed.
pub trait Payload {
    type Metadata;
    #[logic] fn metadata(self)->Self::Metadata;
    #[logic(prophetic)] fn wellformed(self)->bool;
}
type Publication<T>=Option<Ag<(Int,AtView<T>)>>;
type Tickets=(Option<Excl<()>>,Option<Excl<()>>);
pub struct Ticket { resource:Resource<Tickets>, left:bool }
impl Ticket {
    #[logic]
    pub(crate) fn is_left(self)->bool { self.left }
}
struct State<T:Payload> {
    own:Perm<ModelAtomic>, latest:Int, returned:Resource<Tickets>, expected:Snapshot<(T::Metadata,T::Metadata)>,
    left:Option<AtView<T>>,right:Option<AtView<T>>,withdrawn:bool, publication:Authority<Publication<T>>,
}
#[logic]
fn count(r:Tickets)->usize { if r.0 == None {if r.1 == None {2usize} else {1usize}} else {if r.1 == None {1usize} else {0usize}} }
impl<T:Payload> Protocol for State<T> {
    type Public=(ModelAtomic,Id,(T::Metadata,T::Metadata),Id);
    #[logic] fn public(self)->Self::Public { (*self.own.ward(),self.returned.id(),*self.expected,self.publication.id()) }
    #[logic(prophetic)] fn protocol(self)->bool { pearlite! {
        self.own.val().get(self.latest) != None &&
        self.own.val().get(self.latest).unwrap_logic().0 == count(self.returned@) &&
        (forall<t:Int> self.own.val().get(t) != None ==> t <= self.latest) &&
        (self.withdrawn ==> count(self.returned@) == 0usize) &&
        ((self.publication@ != None) == self.withdrawn) &&
        (self.publication@ != None ==> self.publication@.unwrap_logic().0.0 == self.latest &&
            self.publication@.unwrap_logic().0.1.view() == self.own.val().get(self.latest).unwrap_logic().1) &&
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
impl<T:Payload> EventProtocol for State<T> {
    #[logic] fn atomic(self)->ModelAtomic {*self.own.ward()}
}
pub struct SharedRetirement<T:Payload> { atomic:EventAtomic<State<T>> }
impl<T:Payload> SharedRetirement<T> {
    #[logic] pub(crate) fn valid(self)->bool { self.atomic.public().0 == self.atomic.model() }
    #[logic] pub(crate) fn accepts(self,t:Ticket)->bool { pearlite! {
        t.resource.id() == self.atomic.public().1 &&
        t.resource@ == if t.left {(Some(Excl(())),None)} else {(None,Some(Excl(())))}
    } }

    #[logic] pub(crate) fn expected(self)->(T::Metadata,T::Metadata) { self.atomic.public().2 }
    #[logic(open(crate), prophetic)] pub(crate) fn accepts_payload(self,t:Ticket,payload:T)->bool {
        pearlite! { self.accepts(t) && payload.wellformed() && payload.metadata() == if t.is_left() {self.expected().0} else {self.expected().1} }
    }

    #[ensures(result.0.valid() && result.0.accepts(result.1.inner_logic()) && result.0.accepts(result.2.inner_logic()))]
    #[ensures(result.0.expected() == *expected)]
    #[ensures(result.1.inner_logic().is_left() && !result.2.inner_logic().is_left())]
    pub fn new(expected:Snapshot<(T::Metadata,T::Metadata)>)->(Self,Ghost<Ticket>,Ghost<Ticket>) {
        let mut view=ghost! {SyncView::new().into_inner()};
        let (atomic,own)=RawAtomic::new(2,view.borrow_mut());
        let tickets=ghost! {
            let all=Resource::alloc(snapshot!((Some(Excl(())),Some(Excl(()))))).into_inner();
            let (left,right)=all.split(snapshot!((Some(Excl(())),None)),snapshot!((None,Some(Excl(())))));
            let (left,returned)=left.split(snapshot!((Some(Excl(())),None)),snapshot!((None,None)));
            (Ticket{resource:left,left:true},Ticket{resource:right,left:false},returned)
        };
        let (left,rest)=ghost! {let (l,r,u)=tickets.into_inner(); (l,(r,u))}.split();
        let (right,returned)=rest.split();
        let state=ghost! {
            let latest:Snapshot<Int> = snapshot!(atomic.model().get_timestamp(*view));
            let state=State{own:own.into_inner(),latest:latest.into_ghost().into_inner(),
                returned:returned.into_inner(),expected,left:None,right:None,withdrawn:false,publication:Authority::alloc().into_inner()};
            state
        };
        (Self{atomic:EventAtomic::bind(atomic,state)},left,right)
    }

    #[requires(self.valid() && self.accepts_payload(ticket.inner_logic(),resource.inner_logic()))]
    #[ensures(result.0 == (result.1.inner_logic() != None))]
    #[ensures(result.0 ==> result.1.inner_logic().unwrap_logic().0.wellformed() && result.1.inner_logic().unwrap_logic().1.wellformed())]
    #[ensures(result.0 ==> (result.1.inner_logic().unwrap_logic().0.metadata(),result.1.inner_logic().unwrap_logic().1.metadata()) == self.expected())]
    pub fn retire(&self,ticket:Ghost<Ticket>,resource:Ghost<T>)->(bool,Ghost<Option<(T,T)>>) {
        let (mut current,sealed)=AtView::new(resource).split();
        let mut collected:Ghost<Option<(AtView<T>,AtView<T>,Fragment<Publication<T>>)>>=ghost! {None};
        let old=self.atomic.decrement(ghost! {|state:&mut State<T>,c:&mut Committer<ModelAtomic,usize,Relaxed,Release>| {
                let ticket=ticket.into_inner();
                let before=snapshot!(state.returned@);
                state.returned.join_in(ticket.resource);
                proof_assert!(count(*before)>0usize);
                // A relaxed RMW read carries the predecessor's release view
                // as a deferred acquire witness; it does not acquire it yet.
                let previous=c.shoot_load(&state.own,&mut current);
                let published=release::release_rmw(c,&mut state.own,&mut current);
                proof_assert!(c.timestamp() == state.latest);
                proof_assert!(c.val_load() == count(*before));
                proof_assert!(!state.withdrawn);
                let next:Snapshot<Int> = snapshot!(c.timestamp()+1);
                state.latest=next.into_ghost().into_inner();
                if ticket.left {state.left=Some(sealed.into_inner());} else {state.right=Some(sealed.into_inner());}
                let last:Snapshot<bool> = snapshot!(c.val_load() == 1usize);
                if last.into_ghost().into_inner() {
                    let mut left=state.left.take().unwrap();
                    let right=state.right.take().unwrap();
                    left.weaken(published);
                    let stamp=snapshot!(Some(Ag((state.latest,left))));
                    let receipt=state.publication.add_fragment(stamp);
                    *collected=Some((left,right,receipt));
                    state.withdrawn=true;
                }
        }});
        if old == 1 {
            #[cfg(not(feature="negative_no_acquire"))]
            let _seen=self.atomic.acquire(ghost! {|state:&mut State<T>,c:&Committer<ModelAtomic,usize,Acquire,NoStore>| {
                let receipt=&collected.as_ref().unwrap().2;
                state.publication.frag_lemma(receipt);
                proof_assert!(state.withdrawn);
                c.shoot_load(&state.own,&mut current);
                proof_assert!(c.timestamp()==state.latest);
            }});
            (true,ghost! {
                let (left,right,_receipt)=collected.into_inner().unwrap();
                Some((left.sync(*current),right.sync(*current)))
            })
        } else { (false,ghost! {None}) }
    }
}
