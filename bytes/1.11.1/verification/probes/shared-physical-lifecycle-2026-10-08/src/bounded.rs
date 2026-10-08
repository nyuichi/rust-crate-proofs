//! Bounded original-shaped lifecycle: one source ticket and one affine clone quota.
//! No bytes-specific trusted functions. Native atomic events live in event.rs.
use super::*;
use creusot_std::{ghost::{invariant::Protocol,resource::{Resource,Authority,Fragment}},
    logic::{Id,FMap,real::Real,ra::{RA,excl::Excl,agree::Ag,auth::CancelLocalUpdateUnit}},
    std::sync::view::ReleaseSyncView};
use crate::fraction_map::{self as fm,LiveFractions};

pub trait RecoveryPayload {
    type Metadata;
    #[logic] fn metadata(self)->Self::Metadata;
    #[logic(prophetic)] fn wellformed(self)->bool;
}
pub struct Ticket<T:RecoveryPayload> {
    pub token:LifetimeToken, pub(crate) id:Int, pub(crate) fragment:Fragment<LiveFractions>, pub(crate) recovery:Option<T>,
}
pub struct CloneQuota {resource:Resource<Option<Excl<()>>>}
/// Bounded outcome credit: at most one nonlast and one last event in this
/// one-clone probe. This diagnostic credit is separate from physical fractions.
pub struct Receipt {fragment:Fragment<Option<Ag<bool>>>,left:bool,credit:Resource<Option<Excl<()>>>}
impl Receipt {#[logic] pub fn left(self)->bool {self.left}}
type Publication<T>=Option<Ag<(Int,AtView<Option<T>>)>>;

/// A ticket whose subjective recovery payload has been sealed at the calling view.
pub struct Retiring<T:RecoveryPayload> {
    pub token:LifetimeToken, id:Int, fragment:Fragment<LiveFractions>, sealed:AtView<Option<T>>,
}
impl<T:RecoveryPayload> Retiring<T> {
    #[logic] pub fn id(self)->Int {self.id}
    #[logic(prophetic)] pub fn valid(self,p:<State<T> as Protocol>::Public,current:SyncView)->bool {pearlite! {
        0<=self.id && self.id<2 && self.fragment.id()==p.1 &&
        self.fragment@==FMap::singleton(self.id,Excl(self.token.frac())) && self.token.lft()==p.3 &&
        self.sealed.view()<=current &&
        (if self.id==0 {self.sealed.val()!=None && self.sealed.val().unwrap_logic().wellformed() &&
            self.sealed.val().unwrap_logic().metadata()==p.4} else {self.sealed.val()==None})
    }}
}
pub struct RetiringRest<T:RecoveryPayload> {id:Int,fragment:Fragment<LiveFractions>,sealed:AtView<Option<T>>}
impl<T:RecoveryPayload> RetiringRest<T> {
    #[logic] pub fn paired(self,token:LifetimeToken)->Retiring<T> {
        Retiring{id:self.id,fragment:self.fragment,sealed:self.sealed,token}
    }
    #[check(ghost)]
    #[ensures(result.inner_logic()==rest.paired(*token))]
    pub fn with_token(rest:Ghost<Self>,token:Ghost<LifetimeToken>)->Ghost<Retiring<T>> {
        ghost! {let r=rest.into_inner();Retiring{id:r.id,fragment:r.fragment,sealed:r.sealed,token:token.into_inner()}}
    }
}
impl<T:RecoveryPayload> Retiring<T> {
    #[check(ghost)]
    #[ensures(result.inner_logic().1.paired(result.inner_logic().0)==*input)]
    pub fn split_token(input:Ghost<Self>)->Ghost<(LifetimeToken,RetiringRest<T>)> {
        ghost! {let t=input.into_inner();(t.token,RetiringRest{id:t.id,fragment:t.fragment,sealed:t.sealed})}
    }
}
pub struct Pending<T:RecoveryPayload> {
    sealed:AtView<Option<T>>, full:LifetimeToken, fragment:Fragment<Publication<T>>,
}
impl<T:RecoveryPayload> Pending<T> {
    #[logic] pub fn token(self)->LifetimeToken {self.full}
    #[check(ghost)]
    #[ensures(*result==self.token())]
    pub fn borrow_token(&self)->&LifetimeToken {&self.full}

    #[logic(prophetic)] pub fn valid(self,p:<State<T> as Protocol>::Public,current:SyncView)->bool {pearlite! {
        self.sealed.val()!=None && self.sealed.val().unwrap_logic().wellformed() &&
        self.sealed.val().unwrap_logic().metadata()==p.4 &&
        self.full.lft()==p.3 && self.full.frac()==PositiveReal::from_int(1) &&
        self.fragment.id()==p.5 && self.fragment@!=None &&
        self.fragment@.unwrap_logic().0.1==self.sealed &&
        self.fragment@.unwrap_logic().0.0<=p.0.get_timestamp(current)
    }}
    #[check(ghost)]
    #[requires(owned.valid(*p,*current) && owned.sealed.view()<=*current)]
    #[ensures(result.inner_logic().0.wellformed() && result.inner_logic().0.metadata()==p.4)]
    #[ensures(result.inner_logic().1.lft()==p.3 && result.inner_logic().1.frac()==PositiveReal::from_int(1))]
    pub fn recover(owned:Ghost<Self>,p:Snapshot<<State<T> as Protocol>::Public>,current:Ghost<SyncView>)->Ghost<(T,LifetimeToken)> {
        ghost! {
            let owned=owned.into_inner();


        (owned.sealed.sync(current.into_inner()).unwrap(),owned.full)
    
        
        }
    }
}
impl Receipt {
    #[logic] pub fn valid<T:RecoveryPayload>(self,p:<State<T> as Protocol>::Public,last:bool)->bool {pearlite! {
        self.fragment.id()==if self.left {p.6}else{p.7} && self.fragment@==Some(Ag(last)) &&
        self.credit.id()==if last {p.8}else{p.9} && self.credit@==Some(Excl(()))
    }}
}
#[requires(ticket.valid(*p))]
#[ensures(result.1.inner_logic().valid(*p,*result.0))]
#[ensures(result.1.inner_logic().id()==ticket.id)]
pub fn prepare<T:RecoveryPayload>(ticket:Ghost<Ticket<T>>,p:Snapshot<<State<T> as Protocol>::Public>)
    ->(Ghost<SyncView>,Ghost<Retiring<T>>) {
    let (rest,payload)=ghost! {let t=ticket.into_inner();((t.id,(t.token,t.fragment)),t.recovery)}.split();
    let (current,sealed)=AtView::new(payload).split();
    (current,ghost! {let (id,(token,fragment))=rest.into_inner();Retiring{id,token,fragment,sealed:sealed.into_inner()}})
}

pub struct State<T:RecoveryPayload> {
    own:Perm<ModelAtomic>,first:Int,latest:Int,
    alive:Authority<LiveFractions>, next:Int,
    quota:Resource<Option<Excl<()>>>,
    lifetime:Lifetime, pool:Option<LifetimeToken>,
    expected:Snapshot<T::Metadata>,
    recovery:Option<AtView<Option<T>>>, withdrawn:bool,
    publication:Authority<Publication<T>>,
    left_done:Authority<Option<Ag<bool>>>,right_done:Authority<Option<Ag<bool>>>,
    last_credit:Resource<Option<Excl<()>>>,nonlast_credit:Resource<Option<Excl<()>>>,
}
impl<T:RecoveryPayload> Protocol for State<T> {
    type Public=(ModelAtomic,Id,Id,Lifetime,T::Metadata,Id,Id,Id,Id,Id);
    #[logic] fn public(self)->Self::Public {(*self.own.ward(),self.alive.id(),self.quota.id(),self.lifetime,*self.expected,self.publication.id(),self.left_done.id(),self.right_done.id(),self.last_credit.id(),self.nonlast_credit.id())}
    #[logic(prophetic)] fn protocol(self)->bool {pearlite! {
        1<=self.next && self.next<=2 && fm::ids_bounded(self.alive@,self.next) &&
        self.alive@.len()<=self.next &&
        self.own.val().get(self.latest)!=None &&
        self.own.val()[self.latest].0@==self.alive@.len() &&
        self.first<=self.latest &&
        (forall<t:Int> self.own.val().get(t)!=None == (self.first<=t && t<=self.latest)) &&
        (self.quota@ == if self.next==1 {None}else{Some(Excl(()))}) &&
        (self.withdrawn == (self.alive@.len()==0)) &&
        ((self.pool==None)==self.withdrawn) &&
        self.last_credit@==(if self.withdrawn {None}else{Some(Excl(()))}) &&
        self.nonlast_credit@==(if self.left_done@==Some(Ag(false)) || self.right_done@==Some(Ag(false)) {None}else{Some(Excl(()))}) &&
        (self.pool!=None ==> self.pool.unwrap_logic().lft()==self.lifetime &&
            self.pool.unwrap_logic().frac().to_real()+fm::sum_prefix(self.alive@,self.next)==Real::from_int(1)) &&
        ((self.left_done@==None)==self.alive@.contains(0)) &&
        ((self.right_done@==None)==(self.next==1 || self.alive@.contains(1))) &&
        (match (self.left_done@,self.right_done@) {
            (Some(l),Some(r))=>l.0!=r.0,
            (Some(l),None)=>l.0==(self.next==1),
            (None,Some(r))=>!r.0,
            (None,None)=>true
        }) &&
        (if self.withdrawn {self.recovery==None} else {
            ((self.recovery==None)==self.alive@.contains(0)) &&
            (self.recovery!=None ==> self.recovery.unwrap_logic().val()!=None &&
                self.recovery.unwrap_logic().val().unwrap_logic().wellformed() &&
                self.recovery.unwrap_logic().val().unwrap_logic().metadata()==*self.expected &&
                self.recovery.unwrap_logic().view()<=self.own.val()[self.latest].1)
        }) &&
        ((self.publication@!=None)==self.withdrawn) &&
        (self.publication@!=None ==> self.publication@.unwrap_logic().0.0==self.latest &&
            self.publication@.unwrap_logic().0.1.view()==self.own.val()[self.latest].1)
    }}
}
impl<T:RecoveryPayload> EventProtocol for State<T> {
    #[logic] fn atomic(self)->ModelAtomic {*self.own.ward()}
}
impl<T:RecoveryPayload> Ticket<T> {
    #[logic(open(crate),prophetic)] pub fn valid(self,public:<State<T> as Protocol>::Public)->bool {pearlite! {
        0<=self.id && self.id<2 && self.fragment.id()==public.1 &&
        self.fragment@==FMap::singleton(self.id,Excl(self.token.frac())) && self.token.lft()==public.3 &&
        (if self.id==0 {self.recovery!=None && self.recovery.unwrap_logic().wellformed() &&
            self.recovery.unwrap_logic().metadata()==public.4} else {self.recovery==None})
    }}
}
impl CloneQuota {
    #[logic] pub fn valid(self,id:Id)->bool {pearlite! {self.resource.id()==id && self.resource@==Some(Excl(()))}}
}
#[check(ghost)]
#[requires(!auth@.contains(*id))]
#[ensures((^auth)@==auth@.insert(*id,Excl(*fraction)))]
#[ensures(result@==FMap::singleton(*id,Excl(*fraction)))]
#[ensures(result.id()==auth.id() && (^auth).id()==auth.id())]
fn issue(auth:&mut Authority<LiveFractions>,id:Snapshot<Int>,fraction:Snapshot<PositiveReal>)->Fragment<LiveFractions> {
    let before=snapshot!(auth@);
    let one=snapshot!(FMap::singleton(*id,Excl(*fraction)));
    proof_assert!(forall<k:Int> (*before).get(k).op((*one).get(k))!=None);
    let result=auth.add_fragment(one);
    proof_assert!(auth@.ext_eq((*before).insert(*id,Excl(*fraction))));
    result
}
#[check(ghost)]
#[requires(fragment.id()==auth.id() && fragment@==FMap::singleton(*id,Excl(*fraction)))]
#[ensures((^auth)@==auth@.remove(*id))]
#[ensures(auth@.get(*id)==Some(Excl(*fraction)))]
#[ensures((^auth).id()==auth.id())]
fn remove(auth:&mut Authority<LiveFractions>,mut fragment:Fragment<LiveFractions>,id:Snapshot<Int>,fraction:Snapshot<PositiveReal>) {
    auth.frag_lemma(&fragment);
    let before=snapshot!(auth@);
    proof_assert!({fm::singleton_ticket_factor(*before,*id,*fraction); true});
    auth.update(&mut fragment,CancelLocalUpdateUnit);
}

impl<T:RecoveryPayload> State<T> {
    /// Body-proved bootstrap for an externally supplied actual atomic permission.
    #[check(ghost)]
    #[requires(payload.wellformed() && full.frac()==PositiveReal::from_int(1))]
    #[requires(own.val()==FMap::singleton((*own.ward()).get_timestamp(*current),(1usize,*current)))]
    #[ensures(result.inner_logic().0.protocol() && result.inner_logic().0.atomic()==*own.ward())]
    #[ensures(result.inner_logic().0.public().3==full.lft() && result.inner_logic().0.public().4==payload.metadata())]
    #[ensures(result.inner_logic().1.0.valid(result.inner_logic().0.public()) && result.inner_logic().1.0.id==0)]
    #[ensures(result.inner_logic().1.1.valid(result.inner_logic().0.public().2))]
    pub fn initialize(own:Ghost<Perm<ModelAtomic>>,current:Snapshot<SyncView>,payload:Ghost<T>,full:Ghost<LifetimeToken>)
        ->Ghost<(Self,(Ticket<T>,CloneQuota))> {
        ghost! {
            let own=own.into_inner();
            let payload=payload.into_inner();
            let full=full.into_inner();



            let (first,pool)=full.split();
            let mut alive=Authority::alloc().into_inner();
            let fragment=issue(&mut alive,snapshot!(0),snapshot!(first.frac()));
            let all=Resource::alloc(snapshot!(Some(Excl(())))).into_inner();
            let (quota,empty)=all.split(snapshot!(Some(Excl(()))),snapshot!(None));
            let expected=snapshot!(payload.metadata());
            let latest:Snapshot<Int>=snapshot!((*own.ward()).get_timestamp(*current));
            let lifetime:Snapshot<Lifetime>=snapshot!(pool.lft());
            let ticket=Ticket{id:Int::new(0).into_inner(),fragment,token:first,recovery:Some(payload)};
            let state=State{own,first:latest.into_ghost().into_inner(),latest:latest.into_ghost().into_inner(),
                alive,next:Int::new(1).into_inner(),quota:empty,lifetime:lifetime.into_ghost().into_inner(),pool:Some(pool),expected,
                recovery:None,withdrawn:false,publication:Authority::alloc().into_inner(),
                left_done:Authority::alloc().into_inner(),right_done:Authority::alloc().into_inner(),
                last_credit:Resource::alloc(snapshot!(Some(Excl(())))).into_inner(),
                nonlast_credit:Resource::alloc(snapshot!(Some(Excl(())))).into_inner()};
            (state,(ticket,CloneQuota{resource:quota}))
    
        
        }
    }
}

impl<T:RecoveryPayload> State<T> {
    #[check(ghost)]
    #[requires(s.protocol() && source.valid(s.public()) && quota.valid(s.public().2))]
    #[requires(!c.shot_store() && c.ward()==s.atomic())]
    #[requires(if c.val_load()==usize::MAX {c.val_store()==0usize} else {c.val_store()@==c.val_load()@+1})]
    #[ensures((^s).protocol() && (^s).public()==s.public() && (^s).atomic()==s.atomic())]
    #[ensures((^c).shot_store() && c.hist_inv(^c))]
    #[ensures(c.val_load()==1usize && source.id==0 && result.inner_logic().id==1 && result.inner_logic().valid(s.public()))]
    #[ensures(**current<=^current)]
    pub fn on_clone(s:Ghost<&mut Self>,c:Ghost<&mut Committer<ModelAtomic,usize,Relaxed,Relaxed>>,
        source:Ghost<&Ticket<T>>,quota:Ghost<CloneQuota>,current:Ghost<&mut SyncView>,release:Ghost<ReleaseSyncView>)->Ghost<Ticket<T>> {
        ghost! {
            let s=s.into_inner();
            let c=c.into_inner();
            let source=source.into_inner();
            let quota=quota.into_inner();
            let current=current.into_inner();
            let release=release.into_inner();


            s.quota.join_in(quota.resource);
            proof_assert!(s.next==1);
            s.alive.frag_lemma(&source.fragment);
            proof_assert!({fm::cardinality_prefix_is_len(s.alive@,s.next);true});
            proof_assert!(source.id==0 && s.alive@.len()==1);
            let before=snapshot!(s.alive@);
            let token=s.pool.as_mut().unwrap().split_off();
            let fraction=snapshot!(token.frac());
            let fragment=issue(&mut s.alive,snapshot!(1),fraction);
            proof_assert!({fm::insert_fresh(*before,1,*fraction);true});
            proof_assert!({fm::insert_fresh_preserves_bound(*before,1,*fraction);true});
            c.shoot_load(&s.own,current);
            c.shoot_store(&mut s.own,current,release);
            proof_assert!(c.timestamp()==s.latest && c.val_load()==1usize);
            let latest:Snapshot<Int>=snapshot!(c.timestamp()+1);
            s.latest=latest.into_ghost().into_inner();s.next=Int::new(2).into_inner();
            Ticket{id:Int::new(1).into_inner(),fragment,token,recovery:None}
    
        
        }
    }
}

impl<T:RecoveryPayload> State<T> {
    #[check(ghost)]
    #[requires(s.protocol() && input.valid(s.public(),**current))]
    #[requires(!c.shot_store() && c.ward()==s.atomic())]
    #[requires(if c.val_load()==0usize {c.val_store()==usize::MAX} else {c.val_store()@+1==c.val_load()@})]
    #[ensures((^s).protocol() && (^s).public()==s.public() && (^s).atomic()==s.atomic())]
    #[ensures((^c).shot_store() && c.hist_inv(^c) && c.val_load()>0usize)]
    #[ensures(**current<=^current)]
    #[ensures((result.inner_logic().0!=None)==(c.val_load()==1usize))]
    #[ensures(result.inner_logic().0!=None ==> result.inner_logic().0.unwrap_logic().valid(s.public(),^current))]
    #[ensures(result.inner_logic().1.valid::<T>(s.public(),c.val_load()==1usize) && result.inner_logic().1.left()==(input.id()==0))]
    pub fn on_release(s:Ghost<&mut Self>,c:Ghost<&mut Committer<ModelAtomic,usize,Relaxed,Release>>,
        input:Ghost<Retiring<T>>,current:Ghost<&mut SyncView>)->Ghost<(Option<Pending<T>>,Receipt)> {
        ghost! {
            let s=s.into_inner();
            let c=c.into_inner();
            let input=input.into_inner();
            let current=current.into_inner();


            let Retiring{id,token,fragment,sealed}=input;
            let mut collected=None;
            let before=snapshot!(s.alive@);let fraction=snapshot!(token.frac());let identity=snapshot!(id);
            remove(&mut s.alive,fragment,identity,fraction);
            proof_assert!(0<=id && id<s.next && (*before).len()>0);
            proof_assert!({fm::remove_known(*before,id,*fraction,s.next);true});
            proof_assert!({fm::cardinality_prefix_is_len(*before,s.next);true});
            proof_assert!({fm::cardinality_prefix_is_len(s.alive@,s.next);true});
            proof_assert!(s.alive@.len()+1==(*before).len());
            s.pool.as_mut().unwrap().join_in(token);
            c.shoot_load(&s.own,current);
            let published=release::release_rmw(c,&mut s.own,current);
            proof_assert!(c.timestamp()==s.latest && c.val_load()@==(*before).len());
            let latest:Snapshot<Int>=snapshot!(c.timestamp()+1);s.latest=latest.into_ghost().into_inner();
            if id==Int::new(0).into_inner() {s.recovery=Some(sealed);}
            let last:Snapshot<bool>=snapshot!(c.val_load()==1usize);
            let fragment=if id==Int::new(0).into_inner() {s.left_done.add_fragment(snapshot!(Some(Ag(*last))))}
                else {s.right_done.add_fragment(snapshot!(Some(Ag(*last))))};
            let credit=if last.into_ghost().into_inner() {s.last_credit.take()}else{s.nonlast_credit.take()};
            let receipt=Receipt{fragment,left:id==Int::new(0).into_inner(),credit};
            if last.into_ghost().into_inner() {
                proof_assert!({empty_live_map(s.alive@);true});
                proof_assert!({fm::empty_prefix(s.next);true});
                let full=s.pool.take().unwrap();
                proof_assert!(full.frac().ext_eq(PositiveReal::from_int(1)));
                let mut recovery=s.recovery.take().unwrap();recovery.weaken(published);
                let stamp=snapshot!(Some(Ag((s.latest,recovery))));
                let receipt=s.publication.add_fragment(stamp);
                collected=Some(Pending{sealed:recovery,full,fragment:receipt});s.withdrawn=true;
            }
            (collected,receipt)
    
        
        }
    }
    #[check(ghost)]
    #[requires(s.protocol() && pending.valid(s.public(),**current))]
    #[requires(!c.shot_store() && c.ward()==s.atomic())]
    #[ensures((^s).protocol() && (^s).public()==s.public() && (^s).atomic()==s.atomic())]
    #[ensures(**current<=^current && pending.valid(s.public(),^current))]
    #[ensures(pending.sealed.view()<=^current)]
    pub fn on_acquire(s:Ghost<&mut Self>,c:Ghost<&Committer<ModelAtomic,usize,Acquire,NoStore>>,
        pending:Ghost<&Pending<T>>,current:Ghost<&mut SyncView>)->Ghost<()> {
        ghost! {
            let s=s.into_inner();
            let c=c.into_inner();
            let pending=pending.into_inner();
            let current=current.into_inner();


        s.publication.frag_lemma(&pending.fragment);
        c.shoot_load(&s.own,current);
        proof_assert!(c.timestamp()==s.latest);
    
        
        }
    }
}

#[logic]
#[requires(live.len()==0)]
#[ensures(live==LiveFractions::empty())]
fn empty_live_map(live:LiveFractions) {
    proof_assert!(forall<id:Int> live.remove(id).len()>=0);
    proof_assert!(forall<id:Int> live.get(id)==None);
    proof_assert!(live.ext_eq(LiveFractions::empty()));
}

pub struct Registry<T:RecoveryPayload> {atomic:EventAtomic<State<T>>}
impl<T:RecoveryPayload> Registry<T> {
    #[logic] pub fn public(self)-><State<T> as Protocol>::Public {self.atomic.public()}
    #[logic] fn valid(self)->bool {self.public().0==self.atomic.model()}
    #[logic] fn receipt(self,r:Receipt,last:bool)->bool {r.valid::<T>(self.public(),last)}

    #[requires(payload.wellformed())]
    #[requires(full.frac()==PositiveReal::from_int(1))]
    #[ensures(result.0.valid())]
    #[ensures(result.0.public().3==full.lft() && result.0.public().4==payload.metadata())]
    #[ensures(result.1.inner_logic().valid(result.0.public()))]
    #[ensures(result.2.inner_logic().valid(result.0.public().2))]
    pub fn new(payload:Ghost<T>,full:Ghost<LifetimeToken>)->(Self,Ghost<Ticket<T>>,Ghost<CloneQuota>) {
        let mut current=ghost! {SyncView::new().into_inner()};
        let (raw,own)=RawAtomic::new(1,current.borrow_mut());
        let setup=ghost! {State::initialize(own,snapshot!(*current),payload,full).into_inner()};
        let (state,rest)=setup.split();let (first,quota)=rest.split();
        (Self{atomic:EventAtomic::bind(raw,state)},first,quota)
    }

    #[requires(self.valid() && source.valid(self.public()) && quota.valid(self.public().2))]
    #[ensures(result.0==1usize && result.1.inner_logic().valid(self.public()))]
    #[ensures(result.1.inner_logic().id==1 && source.id==0)]
    pub fn clone_bounded(&self,source:Ghost<&Ticket<T>>,quota:Ghost<CloneQuota>)->(usize,Ghost<Ticket<T>>) {
        let mut current=ghost! {SyncView::new().into_inner()};
        let release=ghost! {ReleaseSyncView::new().into_inner()};
        let mut output=ghost! {None::<Ticket<T>>};
        let old=self.atomic.increment(ghost! {|s:&mut State<T>,c:&mut Committer<ModelAtomic,usize,Relaxed,Relaxed>| {
            *output=Some(State::on_clone(Ghost::new(s),Ghost::new(c),source,quota,current.borrow_mut(),release).into_inner());
        }});
        (old,ghost! {output.into_inner().unwrap()})
    }

    #[requires(self.valid() && ticket.valid(self.public()))]
    #[ensures(result.0==(result.1.inner_logic()!=None))]
    #[ensures(result.0 ==> result.1.inner_logic().unwrap_logic().0.wellformed() &&
        result.1.inner_logic().unwrap_logic().0.metadata()==self.public().4 &&
        result.1.inner_logic().unwrap_logic().1.lft()==self.public().3 &&
        result.1.inner_logic().unwrap_logic().1.frac()==PositiveReal::from_int(1))]
    #[ensures(self.receipt(result.2.inner_logic(),result.0))]
    #[ensures(result.2.inner_logic().left()==(ticket.id==0))]
    pub fn retire(&self,ticket:Ghost<Ticket<T>>)->(bool,Ghost<Option<(T,LifetimeToken)>>,Ghost<Receipt>) {
        let (mut current,input)=prepare(ticket,snapshot!(self.public()));
        let mut collected=ghost! {None::<Pending<T>>};
        let mut receipt=ghost! {None::<Receipt>};
        let old=self.atomic.decrement(ghost! {|s:&mut State<T>,c:&mut Committer<ModelAtomic,usize,Relaxed,Release>| {
            let (pending,done)=State::on_release(Ghost::new(s),Ghost::new(c),input,current.borrow_mut()).into_inner();
            *collected=pending;*receipt=Some(done);
        }});
        if old==1 {
            #[cfg(not(feature="negative_no_acquire"))]
            self.atomic.acquire(ghost! {|s:&mut State<T>,c:&Committer<ModelAtomic,usize,Acquire,NoStore>| {
                State::on_acquire(Ghost::new(s),Ghost::new(c),Ghost::new(collected.as_ref().unwrap()),current.borrow_mut());
            }});
            (true,ghost! {Some(Pending::recover(Ghost::new(collected.into_inner().unwrap()),snapshot!(self.public()),current).into_inner())},
                ghost! {receipt.into_inner().unwrap()})
        } else {(false,ghost! {None},ghost! {receipt.into_inner().unwrap()})}
    }
    #[requires(self.valid() && self.receipt(a.inner_logic(),a_last) && self.receipt(b.inner_logic(),b_last))]
    #[requires(a.left()!=b.left())]
    #[ensures(a_last!=b_last)]
    pub fn finish(&self,a_last:bool,a:Ghost<Receipt>,b_last:bool,b:Ghost<Receipt>) {
        reconcile::<T>(snapshot!(self.public()),a_last,a,b_last,b);
    }
}

/// Pure bounded reconciliation. No native load, field access, or capability
/// recovery occurs here; the two affine outcome credits cannot have one class.
#[check(ghost)]
#[requires(a.valid::<T>(*public,a_last) && b.valid::<T>(*public,b_last))]
#[ensures(a_last!=b_last)]
pub fn reconcile<T:RecoveryPayload>(public:Snapshot<<State<T> as Protocol>::Public>,
    a_last:bool,a:Ghost<Receipt>,b_last:bool,b:Ghost<Receipt>) {
    let _checked=ghost! {
        let mut a=a.into_inner();let b=b.into_inner();
        if a_last==b_last {
            a.credit.valid_op_lemma(&b.credit);
            proof_assert!(false);
        }
    };
}
