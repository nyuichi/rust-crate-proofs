// Generic library contract missing from the pinned Std slice surface. This
// preserves the actual native slice::is_empty call. Its interpretation is
// exactly slice length == 0, analogous to Std's adjacent slice::len contract;
// no Bytes representation, ownership, recovery or destructor fact is assumed.
creusot_std::macros::extern_spec! {
    impl<T> [T] {
        #[check(ghost)]
        #[ensures(result == (self@.len() == 0))]
        fn is_empty(&self) -> bool;
    }
}

// AR: byte length and ownership are independent.
impl Bytes {
    #[logic(prophetic)] fn api_view_valid(self)->bool {!self.ptr.is_null_logic() && match self.original_shared.inner_logic() {
        OriginalSharedProof::Child(_)=>self.child_valid(),
        OriginalSharedProof::View(p,b)=>bounded_view(p.core,b,self.ptr,self.len) &&
            self.vtable==shared_table() &&
            p.binding.inner_logic().model()==pointer_event::pointer_model(&self.data) &&
            p.binding.inner_logic().value()==p.core.shared as *mut (),
        OriginalSharedProof::Empty(_)=>self.view_valid(),
        _=>false,
    }}
}
impl OriginalSharedProof {
    #[logic] fn api_owned(self)->bool {match self {
        Self::Child(_)|Self::View(_,_)=>true,_=>false,
    }}
    #[logic] fn api_id(self)->Int {match self {
        Self::Child(p)|Self::View(p,_)=>p.core.ticket.id(),_=>creusot_std::logic::any(),
    }}
    #[logic] fn api_public(self)-><lifecycle::State<Payload> as Protocol>::Public {match self {
        Self::Child(p)|Self::View(p,_)=>p.core.public(),_=>creusot_std::logic::any(),
    }}
    #[logic(prophetic)] fn shared_drop_input(self,data:ModelAtomicPtr<()>,cursor:Cursor)->bool {match self {
        Self::Child(p)|Self::View(p,_)=>p.core.valid() && p.core.accepts(cursor) &&
            p.binding.inner_logic().model()==data && p.binding.inner_logic().value()==p.core.shared as *mut (),
        _=>false,
    }}
    #[logic] fn static_drop_input(self)->bool {match self {Self::Empty(_)=>true,_=>false}}
}

type CursorDropInput<'a>=(OriginalSharedProof,&'a mut Cursor,&'a mut Option<ViewEffect>);
type CursorDropSpec<'a>=fn(&'a mut AtomicPtr<()>,*const u8,usize,Ghost<CursorDropInput<'a>>);
#[requires(input.inner_logic().0.shared_drop_input(pointer_event::pointer_model(data),*input.inner_logic().1))]
#[requires(*input.inner_logic().2==None)]
#[ensures((^input.inner_logic().1).model()==input.inner_logic().1.model() && (^input.inner_logic().1).public()==input.inner_logic().1.public())]
#[ensures((*((^input.inner_logic().1).observation())).0==(*input.inner_logic().1.observation()).0.remove(input.inner_logic().0.api_id()))]
#[ensures((*((^input.inner_logic().1).observation())).1==(*input.inner_logic().1.observation()).1)]
#[ensures(^input.inner_logic().2!=None && !(^input.inner_logic().2).unwrap_logic().was_static())]
#[ensures((^input.inner_logic().2).unwrap_logic().valid(input.inner_logic().1.public().3))]
#[ensures((^input.inner_logic().2).unwrap_logic().reclaimed()==((*input.inner_logic().1.observation()).0.len()==1))]
fn cursor_shared_drop_checked(data:&mut AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<CursorDropInput>) {
    let (proof,cursor,mut output)=input.split();
    let child=ghost! {match proof.into_inner() {
        OriginalSharedProof::Child(p)|OriginalSharedProof::View(p,_)=>p,
        _=>{proof_assert!(false);panic!()},
    }};
    let mut completion=ghost! {None::<Completion>};
    child_drop_checked(data,ptr,len,ghost! {(child.into_inner(),cursor.into_inner(),&mut *completion)});
    ghost! {**output=Some(ViewEffect::Shared(completion.into_inner().unwrap()));};
}

#[requires(input.inner_logic().0.static_drop_input())]
#[requires(*input.inner_logic().2==None)]
#[ensures(^input.inner_logic().1==*input.inner_logic().1)]
#[ensures(^data==*data)]
#[ensures(^input.inner_logic().2!=None && (^input.inner_logic().2).unwrap_logic().was_static())]
#[ensures((^input.inner_logic().2).unwrap_logic().valid(input.inner_logic().1.public().3))]
#[ensures(!(^input.inner_logic().2).unwrap_logic().reclaimed())]
fn cursor_static_drop_checked(data:&mut AtomicPtr<()>,_ptr:*const u8,_len:usize,input:Ghost<CursorDropInput>) {
    ghost! {let (_proof,_cursor,output)=input.into_inner();*output=Some(ViewEffect::Static);};
}

// Mode-correct generic registration: no native function/table value is produced.
// Exact callback contracts are those of the ordinary body-proved helpers above.
#[trusted]
#[check(ghost)]
#[ensures(erased_call::registered3(shared_table().drop,result.inner_logic()))]
#[ensures(forall<data:&mut AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<CursorDropInput>>
    result.inner_logic().precondition((data,ptr,len,input))==cursor_shared_drop_checked.precondition((data,ptr,len,input)))]
#[ensures(forall<data:&mut AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<CursorDropInput>>
    result.inner_logic().postcondition((data,ptr,len,input),())==cursor_shared_drop_checked.postcondition((data,ptr,len,input),()))]
fn cursor_shared_drop_registration<'a>()->Ghost<CursorDropSpec<'a>> {Ghost::conjure()}

#[trusted]
#[check(ghost)]
#[ensures(erased_call::registered3(static_view_table().drop,result.inner_logic()))]
#[ensures(forall<data:&mut AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<CursorDropInput>>
    result.inner_logic().precondition((data,ptr,len,input))==cursor_static_drop_checked.precondition((data,ptr,len,input)))]
#[ensures(forall<data:&mut AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<CursorDropInput>>
    result.inner_logic().postcondition((data,ptr,len,input),())==cursor_static_drop_checked.postcondition((data,ptr,len,input),()))]
fn cursor_static_drop_registration<'a>()->Ghost<CursorDropSpec<'a>> {Ghost::conjure()}

#[requires(value.api_view_valid() && value.view_accepts(*scope.inner_logic()))]
#[requires(*output.inner_logic()==None)]
#[ensures((^scope).model()==scope.inner_logic().model() && (^scope).public()==scope.inner_logic().public())]
#[ensures((*((^scope).observation())).1==(*scope.inner_logic().observation()).1)]
#[ensures((*((^scope).observation())).0==if value.view_owned() {
    (*scope.inner_logic().observation()).0.remove(value.view_id())
} else {(*scope.inner_logic().observation()).0})]
#[ensures(^output!=None && (^output).unwrap_logic().valid(scope.inner_logic().public().3))]
#[ensures((^output).unwrap_logic().was_static()==!value.view_owned())]
#[ensures((^output).unwrap_logic().reclaimed()==(value.view_owned() && (*scope.inner_logic().observation()).0.len()==1))]
fn bytes_cursor_terminal_drop(mut value:Bytes,mut scope:Ghost<&mut DetachedScope>,mut output:Ghost<&mut Option<ViewEffect>>) {
    let native=value.vtable.drop;
    let spec:Ghost<CursorDropSpec>=ghost! {match &*value.original_shared {
        OriginalSharedProof::Child(_)|OriginalSharedProof::View(_,_)=>cursor_shared_drop_registration().into_inner(),
        OriginalSharedProof::Empty(_)=>cursor_static_drop_registration().into_inner(),
        _=>{proof_assert!(false);panic!()},
    }};
    erased_call::invoke3(native,(&mut value.data,value.ptr,value.len),
        ghost! {(value.original_shared.into_inner(),&mut scope.cursor,&mut **output)},spec);
}

impl Bytes {
    /// Exact affine owner/binding frame, independent of the displayed byte view.
    #[logic(prophetic)] fn same_api_owner(self,before:Bytes)->bool {
        match (before.original_shared.inner_logic(),self.original_shared.inner_logic()) {
            (OriginalSharedProof::Child(a),OriginalSharedProof::View(b,_))|
            (OriginalSharedProof::View(a,_),OriginalSharedProof::View(b,_))=>a==b,
            (OriginalSharedProof::Empty(a),OriginalSharedProof::Empty(b))=>a.binding==b.binding,
            _=>false,
        }
    }
}

#[requires(value.api_view_valid() && cursor_by<=value.len)]
#[ensures((^value).api_view_valid() && (^value).same_api_owner(*value))]
#[ensures((^value).data==value.data && (^value).vtable==value.vtable)]
#[ensures((^value).len@==value.len@-cursor_by@)]
#[ensures((^value).ptr.addr_logic()@==value.ptr.addr_logic()@+cursor_by@)]
#[ensures((^value).view_content()==value.view_content().subsequence(cursor_by@,value.len@))]
#[ensures((^value).view_owned()==value.view_owned())]
#[ensures(value.view_owned() ==> (^value).view_id()==value.view_id() &&
    (^value).view_fraction()==value.view_fraction() && (^value).view_public()==value.view_public())]
unsafe fn inc_start_api(value:&mut Bytes,cursor_by:usize) {
    debug_assert!(value.len>=cursor_by,"internal: inc_start out of bounds");
    value.len-=cursor_by;
    let (bound,lease)=ghost! {match &*value.original_shared {
        OriginalSharedProof::Child(p)=>{
            let full:&FullBorrow<raw_vec::PhysicalRegion>=(*p.core.physical).to_ref();
            (p.core.bound,crate::cursor_pointer::AdvanceLease::Live(full.borrow(&p.core.ticket.token)))
        },
        OriginalSharedProof::View(p,b)=>{
            let full:&FullBorrow<raw_vec::PhysicalRegion>=(*p.core.physical).to_ref();
            (*b,crate::cursor_pointer::AdvanceLease::Live(full.borrow(&p.core.ticket.token)))
        },
        OriginalSharedProof::Empty(p)=>(p.bound,crate::cursor_pointer::AdvanceLease::Zero),
        _=>{proof_assert!(false);panic!()},
    }}.split();
    let (ptr,shifted)=unsafe {crate::cursor_pointer::add(value.ptr,cursor_by,bound.borrow(),lease)};
    value.ptr=ptr;
    ghost! {
        let old=core::mem::replace(&mut *value.original_shared,OriginalSharedProof::Vacant);
        *value.original_shared=match old {
            OriginalSharedProof::Child(p)|OriginalSharedProof::View(p,_)=>
                OriginalSharedProof::View(p,shifted.into_inner()),
            OriginalSharedProof::Empty(mut p)=>{
                p.bound=shifted.into_inner();OriginalSharedProof::Empty(p)
            },
            _=>{proof_assert!(false);panic!()},
        };
    };
}

#[requires(value.api_view_valid() && count<=value.len)]
#[ensures((^value).api_view_valid() && (^value).same_api_owner(*value))]
#[ensures((^value).data==value.data && (^value).vtable==value.vtable)]
#[ensures((^value).len@==value.len@-count@)]
#[ensures((^value).ptr.addr_logic()@==value.ptr.addr_logic()@+count@)]
#[ensures((^value).view_content()==value.view_content().subsequence(count@,value.len@))]
#[ensures((^value).view_owned()==value.view_owned())]
#[ensures(value.view_owned() ==> (^value).view_id()==value.view_id() &&
    (^value).view_fraction()==value.view_fraction() && (^value).view_public()==value.view_public())]
fn advance_api(value:&mut Bytes,count:usize) {
    assert!(count<=value.len,"cannot advance past `remaining`: {:?} <= {:?}",count,value.len);
    unsafe {inc_start_api(value,count)}
}

#[requires(value.api_view_valid())]
#[ensures(result==value.len && result@==value.view_content().len())]
fn remaining_api(value:&Bytes)->usize {value.len}

#[requires(value.api_view_valid())]
#[ensures(result@==value.view_content())]
fn read_api<'a>(value:&'a Bytes)->&'a [u8] {
    if value.len==0 {
        let bound=ghost! {match &*value.original_shared {
            OriginalSharedProof::Child(p)=>&p.core.bound,
            OriginalSharedProof::View(_,b)=>b,
            OriginalSharedProof::Empty(p)=>&p.bound,
            _=>{proof_assert!(false);panic!()},
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

#[requires(value.api_view_valid())]
#[ensures(result@==value.view_content())]
fn chunk_api<'a>(value:&'a Bytes)->&'a [u8] {read_api(value)}

/// Logical prefix fold: no resource, ticket, state or native effect is returned.
#[logic]
#[requires(0<=count && count<=steps.len() && 0<=capacity)]
#[ensures(0<=result && result<=capacity)]
#[variant(count)]
fn consumed(steps:Seq<usize>,count:Int,capacity:Int)->Int {
    pearlite! {if count==0 {0} else {
        let previous=consumed(steps,count-1,capacity);
        if previous+steps[count-1]@<=capacity {previous+steps[count-1]@} else {capacity}
    }}
}

// Exact AQ slice body with a stronger, body-proved nonnull API entry post.
#[requires(source.view_valid() && source.view_accepts(*scope.inner_logic()))]
#[requires(range.start<=range.end && range.end<=source.len)]
#[ensures(result.api_view_valid())]
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
fn slice_cursor_entry(source:&Bytes,range:core::ops::Range<usize>,mut scope:Ghost<&mut DetachedScope>)->Bytes {
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
