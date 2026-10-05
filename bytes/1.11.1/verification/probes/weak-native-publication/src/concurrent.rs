use super::*;
use creusot_std::{ghost::{invariant::{AtomicInvariant,Protocol,Tokens,declare_namespace},resource::Resource},logic::{Id,ra::excl::Excl}};
declare_namespace! { PUBLICATION }
type Tickets=(Option<Excl<()>>,Option<Excl<()>>);
pub struct Ticket { resource:Resource<Tickets>, left:bool }
struct State<T> {
    own:Perm<ModelAtomic>, latest:Int, returned:Resource<Tickets>,
    left:Option<AtView<T>>,right:Option<AtView<T>>,withdrawn:bool,
}
#[logic]
fn count(r:Tickets)->usize { if r.0 == None {if r.1 == None {2} else {1}} else {if r.1 == None {1} else {0}} }
impl<T> Protocol for State<T> {
    type Public=(ModelAtomic,Id);
    #[logic] fn public(self)->Self::Public { (*self.own.ward(),self.returned.id()) }
    #[logic] fn protocol(self)->bool { pearlite! {
        self.own.val().get(self.latest) != None &&
        self.own.val().get(self.latest).unwrap_logic().0 == count(self.returned@) &&
        (forall<t:Int> self.own.val().get(t) != None ==> t <= self.latest) &&
        (self.withdrawn ==> count(self.returned@) == 0usize) &&
        (if self.withdrawn {self.left == None && self.right == None} else {
            (self.left == None) == (self.returned@.0 == None) &&
            (self.right == None) == (self.returned@.1 == None) &&
            (self.left != None ==> self.left.unwrap_logic().view() <= self.own.val().get(self.latest).unwrap_logic().1) &&
            (self.right != None ==> self.right.unwrap_logic().view() <= self.own.val().get(self.latest).unwrap_logic().1)
        })
    } }
}
pub struct SharedRetirement<T> { atomic:NativeAtomic, invariant:Ghost<AtomicInvariant<State<T>>> }
impl<T> SharedRetirement<T> {
    #[logic] fn valid(self)->bool { self.invariant.public().0 == self.atomic.model() && self.invariant.namespace() == PUBLICATION() }
    #[logic] fn accepts(self,t:Ticket)->bool { pearlite! {
        t.resource.id() == self.invariant.public().1 &&
        t.resource@ == if t.left {(Some(Excl(())),None)} else {(None,Some(Excl(())))}
    } }

    #[ensures(result.0.valid() && result.0.accepts(result.1.inner_logic()) && result.0.accepts(result.2.inner_logic()))]
    pub fn new()->(Self,Ghost<Ticket>,Ghost<Ticket>) {
        let mut view=SyncView::new();
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
            let state=State{own:own.into_inner(),latest:*snapshot!(atomic.model().get_timestamp(*view)),
                returned:returned.into_inner(),left:None,right:None,withdrawn:false};
            AtomicInvariant::new(Ghost::new(state),snapshot!(PUBLICATION())).into_inner()
        };
        (Self{atomic,invariant},left,right)
    }

    #[requires(self.valid() && self.accepts(ticket.inner_logic()))]
    #[requires(tokens.contains(PUBLICATION()))]
    pub fn retire(&self,ticket:Ghost<Ticket>,resource:Ghost<T>,mut tokens:Ghost<Tokens>)->Ghost<Option<(T,T)>> {
        let (mut current,sealed)=AtView::new(resource).split();
        let mut collected:Ghost<Option<(AtView<T>,AtView<T>)>>=ghost! {None};
        let mut publication=ghost! { *current };
        let old=self.atomic.decrement(ghost! {|c:&mut Committer<ModelAtomic,usize,Relaxed,Release>| {
            self.invariant.open(tokens.reborrow().into_inner(), |state:&mut State<T>| {
                let ticket=ticket.into_inner();
                let before=snapshot!(state.returned@);
                state.returned.join_in(ticket.resource);
                proof_assert!(count(*before)>0usize);
                let published=primitive::release_rmw(c,&mut state.own,&mut current);
                proof_assert!(c.timestamp() == state.latest);
                proof_assert!(c.val_load() == count(*before));
                proof_assert!(!state.withdrawn);
                state.latest=*snapshot!(c.timestamp()+1);
                if ticket.left {state.left=Some(sealed.into_inner());} else {state.right=Some(sealed.into_inner());}
                if c.val_load() == 1usize {
                    *collected=Some((state.left.take().unwrap(),state.right.take().unwrap()));
                    state.withdrawn=true;
                }
                *publication=published;
            });
        }});
        if old == 1 {
            let zero=self.atomic.acquire(ghost! {|c:&Committer<ModelAtomic,usize,Acquire,NoStore>| {
                self.invariant.open(tokens.reborrow().into_inner(), |state:&mut State<T>| {
                    c.shoot_load(&state.own,&mut current);
                    proof_assert!(*publication <= *current);
                });
            }});
            ghost! {let (l,r)=collected.into_inner().unwrap(); Some((l.sync(*current),r.sync(*current)))}
        } else { ghost! {None} }
    }
}
