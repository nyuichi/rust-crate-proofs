
// AQ: full-allocation ownership and separately bounded public views.
struct EmptyViewProof {
    bound:raw_vec::BoundPtr,
    binding:Ghost<pointer_event::ReadOnlyPointer>,
}
#[logic(opaque)] fn static_view_table()->&'static Vtable {dead}
#[trusted] #[ensures(result==static_view_table())]
fn static_view_table_reification()->&'static Vtable {unreachable!("native STATIC_VTABLE reification")}

#[logic(prophetic)]
fn bounded_view(core:SharedCore,bound:raw_vec::BoundPtr,ptr:*const u8,len:usize)->bool {
    pearlite! {core.valid() && bound.invariant() && bound@!=None &&
        bound@.unwrap_logic().0==core.bound@.unwrap_logic().0 &&
        bound@.unwrap_logic().1==core.capacity@ &&
        0<=bound@.unwrap_logic().2 && bound@.unwrap_logic().2+len@<=core.capacity@ &&
        ptr==bound.raw_pointer() as *const u8 &&
        bound.current_address()==ptr.addr_logic()@ &&
        bound.current_address()==core.bound.current_address()+bound@.unwrap_logic().2}
}
impl Bytes {
    #[logic] fn view_owned(self)->bool {match self.original_shared.inner_logic() {
        OriginalSharedProof::Child(_)|OriginalSharedProof::View(_,_)=>true,_=>false,
    }}
    #[logic] fn view_bound(self)->raw_vec::BoundPtr {match self.original_shared.inner_logic() {
        OriginalSharedProof::Child(p)=>p.core.bound,
        OriginalSharedProof::View(_,b)=>b,
        OriginalSharedProof::Empty(p)=>p.bound,
        _=>creusot_std::logic::any(),
    }}
    #[logic(prophetic)] fn view_valid(self)->bool {pearlite! {match self.original_shared.inner_logic() {
        OriginalSharedProof::Child(_)=>self.child_valid(),
        OriginalSharedProof::View(p,b)=>bounded_view(p.core,b,self.ptr,self.len) && self.len>0usize &&
            self.vtable==shared_table() &&
            p.binding.inner_logic().model()==pointer_event::pointer_model(&self.data) &&
            p.binding.inner_logic().value()==p.core.shared as *mut (),
        OriginalSharedProof::Empty(p)=>self.len==0usize && self.vtable==static_view_table() &&
            p.bound.invariant() && p.bound@==None && self.ptr==p.bound.raw_pointer() as *const u8 &&
            p.binding.inner_logic().model()==pointer_event::pointer_model(&self.data) &&
            p.binding.inner_logic().value()==crate::view_pointer::null_word(),
        _=>false,
    }}}
    #[logic] fn view_content(self)->Seq<u8> {pearlite! {match self.original_shared.inner_logic() {
        OriginalSharedProof::Child(p)=>p.core.content(),
        OriginalSharedProof::View(p,b)=>p.core.content().subsequence(b@.unwrap_logic().2,b@.unwrap_logic().2+self.len@),
        OriginalSharedProof::Empty(_)=>Seq::empty(),
        _=>creusot_std::logic::any(),
    }}}
    #[logic] fn view_id(self)->Int {match self.original_shared.inner_logic() {
        OriginalSharedProof::Child(p)|OriginalSharedProof::View(p,_)=>p.core.ticket.id(),
        _=>creusot_std::logic::any(),
    }}
    #[logic] fn view_fraction(self)->PositiveReal {match self.original_shared.inner_logic() {
        OriginalSharedProof::Child(p)|OriginalSharedProof::View(p,_)=>p.core.ticket.fraction(),
        _=>creusot_std::logic::any(),
    }}
    #[logic] fn view_public(self)-><lifecycle::State<Payload> as Protocol>::Public {match self.original_shared.inner_logic() {
        OriginalSharedProof::Child(p)|OriginalSharedProof::View(p,_)=>p.core.public(),
        _=>creusot_std::logic::any(),
    }}
    #[logic] fn view_accepts(self,scope:DetachedScope)->bool {match self.original_shared.inner_logic() {
        OriginalSharedProof::Child(p)|OriginalSharedProof::View(p,_)=>p.core.accepts(scope.cursor),
        OriginalSharedProof::Empty(_)=>true,_=>false,
    }}
}

#[requires(!ptr.is_null_logic())]
#[ensures(result.view_valid() && !result.view_owned() && result.view_content()==Seq::empty())]
#[ensures(result.len==0usize && result.ptr.addr_logic()==ptr.addr_logic())]
fn new_empty_view(ptr:*const u8)->Bytes {
    debug_assert!(!ptr.is_null());
    let (ptr,bound)=crate::view_pointer::without_provenance(ptr);
    let mut current=ghost! {SyncView::new().into_inner()};
    let null=crate::view_pointer::null_pointer();
    let (data,permission)=pointer_event::new_pointer(null,current.borrow_mut());
    let binding=pointer_event::bind_read_only(&data,null,permission);
    Bytes {ptr,len:0,data,vtable:static_view_table_reification(),
        original_shared:ghost! {OriginalSharedProof::Empty(EmptyViewProof {bound:bound.into_inner(),binding})}}
}

type ViewArcInput<'a>=(&'a SharedCore,raw_vec::BoundPtr,&'a mut Cursor);
#[requires(bounded_view(*input.inner_logic().0,input.inner_logic().1,ptr,len) && len>0usize)]
#[requires(shared==input.inner_logic().0.shared && input.inner_logic().0.accepts(*input.inner_logic().2))]
#[ensures(result.view_valid() && result.view_owned() && result.ptr==ptr && result.len==len)]
#[ensures(result.view_bound()==input.inner_logic().1)]
#[ensures(result.view_public()==input.inner_logic().0.public())]
#[ensures(result.view_content()==input.inner_logic().0.content().subsequence(input.inner_logic().1@.unwrap_logic().2,input.inner_logic().1@.unwrap_logic().2+len@))]
#[ensures(match result.original_shared.inner_logic() {OriginalSharedProof::View(p,_)=>p.core.accepts(^input.inner_logic().2),_=>false})]
#[ensures((^input.inner_logic().2).model()==input.inner_logic().2.model() && (^input.inner_logic().2).public()==input.inner_logic().2.public())]
#[ensures(!(*input.inner_logic().2.observation()).0.contains(result.view_id()))]
#[ensures((*((^input.inner_logic().2).observation())).0==(*input.inner_logic().2.observation()).0.insert(result.view_id(),Excl(result.view_fraction())))]
#[ensures((*((^input.inner_logic().2).observation())).1==(*input.inner_logic().2.observation()).1+1)]
fn shallow_clone_view_checked(shared:*mut Shared,ptr:*const u8,len:usize,input:Ghost<ViewArcInput>)->Bytes {
    let (source,bound,cursor)=input.split();
    let mut current=ghost! {SyncView::new().into_inner()};
    let release=ghost! {ReleaseSyncView::new().into_inner()};
    let mut ticket=ghost! {None::<lifecycle::Ticket<Payload>>};
    let old=field_event::increment_owned::<Shared,lifecycle::State<Payload>,_>(
        shared,ghost! {(*source.control).to_ref()},ghost! {&source.ticket.token},
        ghost! {(*source.invariant).to_ref()},cursor,
        ghost! {|state:&mut lifecycle::State<Payload>,c:&mut Committer<ModelAtomic,usize,Relaxed,Relaxed>| {
            *ticket=Some(lifecycle::State::on_register(Ghost::new(state),Ghost::new(c),
                ghost! {&*source.ticket},current.borrow_mut(),release).into_inner());
        }});
    let mut pointer_view=ghost! {SyncView::new().into_inner()};
    let (data,permission)=pointer_event::new_pointer(shared.cast(),pointer_view.borrow_mut());
    let binding=pointer_event::bind_read_only(&data,shared.cast(),permission);
    Bytes {ptr,len,data,vtable:shared_table_reification(),original_shared:ghost! {
        OriginalSharedProof::View(ChildProof {
            core:SharedCore {shared:source.shared,bound:source.bound,capacity:source.capacity,
                control:source.control,physical:source.physical,invariant:source.invariant,
                ticket:Ghost::new(ticket.into_inner().unwrap())},binding,
        },bound.into_inner())
    }}
}

type ViewCloneInput<'a>=(&'a ChildProof,raw_vec::BoundPtr,&'a mut Cursor);
type ViewCloneSpec<'a>=fn(&'a AtomicPtr<()>,*const u8,usize,Ghost<ViewCloneInput<'a>>)->Bytes;
#[requires(bounded_view(input.inner_logic().0.core,input.inner_logic().1,ptr,len) && len>0usize)]
#[requires(input.inner_logic().0.core.accepts(*input.inner_logic().2))]
#[requires(input.inner_logic().0.binding.inner_logic().model()==pointer_event::pointer_model(data))]
#[requires(input.inner_logic().0.binding.inner_logic().value()==input.inner_logic().0.core.shared as *mut ())]
#[ensures(result.view_valid() && result.view_owned() && result.ptr==ptr && result.len==len)]
#[ensures(result.view_bound()==input.inner_logic().1)]
#[ensures(result.view_public()==input.inner_logic().0.core.public())]
#[ensures(result.view_content()==input.inner_logic().0.core.content().subsequence(input.inner_logic().1@.unwrap_logic().2,input.inner_logic().1@.unwrap_logic().2+len@))]
#[ensures(match result.original_shared.inner_logic() {OriginalSharedProof::View(p,_)=>p.core.accepts(^input.inner_logic().2),_=>false})]
#[ensures((^input.inner_logic().2).model()==input.inner_logic().2.model() && (^input.inner_logic().2).public()==input.inner_logic().2.public())]
#[ensures(!(*input.inner_logic().2.observation()).0.contains(result.view_id()))]
#[ensures((*((^input.inner_logic().2).observation())).0==(*input.inner_logic().2.observation()).0.insert(result.view_id(),Excl(result.view_fraction())))]
#[ensures((*((^input.inner_logic().2).observation())).1==(*input.inner_logic().2.observation()).1+1)]
fn shared_view_clone_checked(data:&AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<ViewCloneInput>)->Bytes {
    let (source,bound,cursor)=input.split();
    let shared=pointer_event::load_relaxed(data,ghost! {&*source.binding});
    shallow_clone_view_checked(shared.cast(),ptr,len,ghost! {(&source.core,bound.into_inner(),cursor.into_inner())})
}
#[trusted]
#[ensures(result.0==shared_table())]
#[ensures(erased_call::registered3(result.0.clone,result.1.inner_logic()))]
#[ensures(forall<data:&AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<ViewCloneInput>>
    result.1.inner_logic().precondition((data,ptr,len,input))==shared_view_clone_checked.precondition((data,ptr,len,input)))]
#[ensures(forall<data:&AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<ViewCloneInput>,output:Bytes>
    result.1.inner_logic().postcondition((data,ptr,len,input),output)==shared_view_clone_checked.postcondition((data,ptr,len,input),output))]
fn shared_view_clone_registration<'a>()->(&'static Vtable,Ghost<ViewCloneSpec<'a>>) {unreachable!("native Shared view clone erasure")}

#[requires(source.view_valid() && source.view_owned() && source.view_accepts(*scope.inner_logic()))]
#[ensures(result.view_valid() && result.view_owned() && result.view_content()==source.view_content())]
#[ensures(match result.original_shared.inner_logic() {OriginalSharedProof::View(_,_)=>true,_=>false})]
#[ensures(result.view_bound()==source.view_bound() && result.ptr==source.ptr && result.len==source.len)]
#[ensures(result.view_public()==source.view_public() && result.view_accepts(^scope))]
#[ensures((^scope).model()==scope.inner_logic().model() && (^scope).public()==scope.inner_logic().public())]
#[ensures(!(*scope.inner_logic().observation()).0.contains(result.view_id()))]
#[ensures((*((^scope).observation())).0==(*scope.inner_logic().observation()).0.insert(result.view_id(),Excl(result.view_fraction())))]
#[ensures((*((^scope).observation())).1==(*scope.inner_logic().observation()).1+1)]
fn clone_shared_view(source:&Bytes,mut scope:Ghost<&mut DetachedScope>)->Bytes {
    let (proof,bound)=ghost! {match &*source.original_shared {
        OriginalSharedProof::Child(p)=>(p,p.core.bound),OriginalSharedProof::View(p,b)=>(p,*b),
        _=>{proof_assert!(false);panic!()},
    }}.split();
    let native=source.vtable.clone;
    let (table,spec)=shared_view_clone_registration();
    proof_assert!(source.vtable==table);
    erased_call::invoke3(native,(&source.data,source.ptr,source.len),ghost! {(*proof,bound.into_inner(),&mut scope.cursor)},spec)
}

#[requires(source.view_valid() && source.view_accepts(*scope.inner_logic()))]
#[requires(range.start<=range.end && range.end<=source.len)]
#[ensures(result.view_valid() && result.view_content()==source.view_content().subsequence(range.start@,range.end@))]
#[ensures(result.len@==range.end@-range.start@ && result.view_owned()==(range.start<range.end))]
#[ensures(result.view_accepts(^scope))]
#[ensures(result.view_owned() ==> result.view_public()==source.view_public())]
#[ensures((^scope).model()==scope.inner_logic().model() && (^scope).public()==scope.inner_logic().public())]
#[ensures(if result.view_owned() {
    !(*scope.inner_logic().observation()).0.contains(result.view_id()) &&
    (*((^scope).observation())).0==(*scope.inner_logic().observation()).0.insert(result.view_id(),Excl(result.view_fraction())) &&
    (*((^scope).observation())).1==(*scope.inner_logic().observation()).1+1
} else {(^scope).observation()==scope.inner_logic().observation()})]
fn slice_view(source:&Bytes,range:core::ops::Range<usize>,mut scope:Ghost<&mut DetachedScope>)->Bytes {
    use core::ops::{Bound,RangeBounds};
    let len=source.len;
    let view_begin=match range.start_bound() {
        Bound::Included(&n)=>n,
        Bound::Excluded(&n)=>n.checked_add(1).expect("out of range"),
        Bound::Unbounded=>0,
    };
    let end=match range.end_bound() {
        Bound::Included(&n)=>n.checked_add(1).expect("out of range"),
        Bound::Excluded(&n)=>n,
        Bound::Unbounded=>len,
    };
    assert!(view_begin<=end,"range start must not be greater than end: {:?} <= {:?}",view_begin,end);
    assert!(end<=len,"range end out of bounds: {:?} <= {:?}",end,len);
    if end==view_begin {
        let bound=ghost! {match &*source.original_shared {
            OriginalSharedProof::Child(p)=>p.core.bound,OriginalSharedProof::View(_,b)=>*b,
            OriginalSharedProof::Empty(p)=>p.bound,_=>{proof_assert!(false);panic!()},
        }};
        let (ptr,_bound)=crate::view_pointer::wrapping_bounded(source.ptr,view_begin,bound.borrow());
        return new_empty_view(ptr);
    }
    let mut ret=clone_shared_view(source,ghost! {&mut **scope});
    ret.len=end-view_begin;
    let (bound,region)=ghost! {
        match &*ret.original_shared {
            OriginalSharedProof::View(p,b)=>{
                let full:&FullBorrow<raw_vec::PhysicalRegion>=(*p.core.physical).to_ref();
                (b,full.borrow(&p.core.ticket.token))
            },_=>{proof_assert!(false);panic!()},
        }
    }.split();
    let (ptr,shifted)=unsafe {crate::view_pointer::add_live(ret.ptr,view_begin,bound,region)};
    ret.ptr=ptr;
    ret.original_shared=ghost! {match ret.original_shared.into_inner() {
        OriginalSharedProof::View(p,_)=>OriginalSharedProof::View(p,shifted.into_inner()),
        _=>{proof_assert!(false);panic!()},
    }};
    ret
}

#[requires(value.view_valid() && value.view_accepts(*scope.inner_logic()))]
#[ensures(result@==value.view_content())]
fn read_view<'a>(value:&'a Bytes,scope:Ghost<&'a DetachedScope>)->&'a [u8] {
    if value.len==0 {
        let bound=ghost! {match &*value.original_shared {
            OriginalSharedProof::Empty(p)=>&p.bound,_=>{proof_assert!(false);panic!()},
        }};
        unsafe {physical_projection::borrow_empty(value.ptr,bound)}
    } else {
        let (proof,bound)=ghost! {match &*value.original_shared {
            OriginalSharedProof::Child(p)=>(p,&p.core.bound),OriginalSharedProof::View(p,b)=>(p,b),
            _=>{proof_assert!(false);panic!()},
        }}.split();
        let region=ghost! {
            let full:&FullBorrow<raw_vec::PhysicalRegion>=(*proof.core.physical).to_ref();
            full.borrow(&proof.core.ticket.token)
        };
        unsafe {physical_projection::borrow(value.ptr,value.len,bound,region)}
    }
}

// Static means no ticket existed. Shared is the actual release_core receipt.
enum ViewEffect { Static, Shared(Completion) }
impl ViewEffect {
    #[logic] fn reclaimed(self)->bool {match self {Self::Static=>false,Self::Shared(c)=>c.reclaimed()}}
    #[logic] fn valid(self,metadata:<Payload as lifecycle::RecoveryPayload>::Metadata)->bool {
        match self {Self::Static=>true,Self::Shared(c)=>c.valid(metadata)}
    }
    #[logic] fn was_static(self)->bool {match self {Self::Static=>true,_=>false}}
}

type StaticDropSpec<'a>=fn(&'a mut AtomicPtr<()>,*const u8,usize,Ghost<()>);
#[ensures(^data==*data)]
fn static_view_drop_checked(data:&mut AtomicPtr<()>,_ptr:*const u8,_len:usize,_input:Ghost<()>) {}
#[trusted]
#[ensures(result.0==static_view_table())]
#[ensures(erased_call::registered3(result.0.drop,result.1.inner_logic()))]
#[ensures(forall<data:&mut AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<()>>
    result.1.inner_logic().precondition((data,ptr,len,input))==static_view_drop_checked.precondition((data,ptr,len,input)))]
#[ensures(forall<data:&mut AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<()>>
    result.1.inner_logic().postcondition((data,ptr,len,input),())==static_view_drop_checked.postcondition((data,ptr,len,input),()))]
fn static_view_drop_registration<'a>()->(&'static Vtable,Ghost<StaticDropSpec<'a>>) {unreachable!("native static_drop erasure")}

#[requires(value.view_valid() && value.view_accepts(*scope.inner_logic()))]
#[requires(*output.inner_logic()==None)]
#[ensures((^scope).model()==scope.inner_logic().model() && (^scope).public()==scope.inner_logic().public())]
#[ensures((*((^scope).observation())).1==(*scope.inner_logic().observation()).1)]
#[ensures((*((^scope).observation())).0==if value.view_owned() {
    (*scope.inner_logic().observation()).0.remove(value.view_id())
} else {(*scope.inner_logic().observation()).0})]
#[ensures(^output!=None && (^output).unwrap_logic().valid(scope.inner_logic().public().3))]
#[ensures((^output).unwrap_logic().was_static()==!value.view_owned())]
#[ensures((^output).unwrap_logic().reclaimed()==(value.view_owned() && (*scope.inner_logic().observation()).0.len()==1))]
fn bytes_view_terminal_drop(mut value:Bytes,mut scope:Ghost<&mut DetachedScope>,mut output:Ghost<&mut Option<ViewEffect>>) {
    let native=value.vtable.drop;
    if value.len==0 {
        let (table,spec)=static_view_drop_registration();
        proof_assert!(value.vtable==table);
        erased_call::invoke3(native,(&mut value.data,value.ptr,value.len),ghost! {()},spec);
        ghost! {**output=Some(ViewEffect::Static);};
    } else {
        let proof=ghost! {match value.original_shared.into_inner() {
            OriginalSharedProof::Child(p)|OriginalSharedProof::View(p,_)=>p,
            _=>{proof_assert!(false);panic!()},
        }};
        let (table,spec)=child_drop_registration();
        proof_assert!(value.vtable==table);
        let mut completion=ghost! {None::<Completion>};
        erased_call::invoke3(native,(&mut value.data,value.ptr,value.len),ghost! {(proof.into_inner(),&mut scope.cursor,&mut *completion)},spec);
        ghost! {**output=Some(ViewEffect::Shared(completion.into_inner().unwrap()));};
    }
}
