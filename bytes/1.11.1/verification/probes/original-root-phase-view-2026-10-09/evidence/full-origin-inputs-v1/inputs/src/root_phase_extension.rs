// AY: one affine Root view across raw and promoted phases.  All inherited
// suffix contracts remain unchanged; Shared windows need not close capacity.
impl SuffixScope {
    #[logic(prophetic)] fn root_view_matches(self,ptr:*const u8,len:usize,model:ModelAtomicPtr<()>,table:&'static Vtable)->bool {pearlite! {
        self.valid() && ptr==self.view.raw_pointer() as *const u8 &&
        0<=self.view@.unwrap_logic().2 && self.view@.unwrap_logic().2+len@<=self.scope.descriptor.capacity@ &&
        (self.scope.is_raw() ==> self.view@.unwrap_logic().2+len@==self.scope.descriptor.capacity@) &&
        model==*self.scope.descriptor.model && table==self.scope.descriptor.table()
    }}
    #[logic(prophetic)] fn root_view_valid(self,value:Bytes)->bool {pearlite! {
        self.root_view_matches(value.ptr,value.len,pointer_event::pointer_model(&value.data),value.vtable) &&
        match value.original_shared.inner_logic() {OriginalSharedProof::Root(d)=>d==self.scope.descriptor,_=>false}
    }}
    #[logic] fn root_view_content(self,len:usize)->Seq<u8> {pearlite! {
        (*self.scope.descriptor.expected).subsequence(self.view@.unwrap_logic().2,self.view@.unwrap_logic().2+len@)
    }}
}

#[requires(scope.inner_logic().root_view_valid(*value) && amount<=value.len)]
#[ensures((^scope).root_view_valid(^value) && (^scope).scope==scope.inner_logic().scope)]
#[ensures((^value).original_shared==value.original_shared && (^value).data==value.data && (^value).vtable==value.vtable)]
#[ensures((^value).len@==value.len@-amount@ && (^value).ptr.addr_logic()@==value.ptr.addr_logic()@+amount@)]
#[ensures((^scope).view@==Some((scope.inner_logic().view@.unwrap_logic().0,
    scope.inner_logic().view@.unwrap_logic().1,scope.inner_logic().view@.unwrap_logic().2+amount@)))]
unsafe fn inc_start_root_view(value:&mut Bytes,amount:usize,mut scope:Ghost<&mut SuffixScope>) {
    debug_assert!(value.len>=amount,"internal: inc_start out of bounds");
    value.len-=amount;
    let (bound,lease)=ghost! {
        let region=match scope.scope.phase.as_ref().unwrap() {
            Phase::Raw(raw)=>&raw.physical,
            Phase::Shared(shared)=>{
                let full:&FullBorrow<raw_vec::PhysicalRegion>=(*shared.root.physical).to_ref();
                full.borrow(&shared.root.ticket.token)
            },
        };
        (&scope.view,crate::cursor_pointer::AdvanceLease::Live(region))
    }.split();
    let (ptr,shifted)=unsafe {crate::cursor_pointer::add(value.ptr,amount,bound,lease)};
    value.ptr=ptr;
    ghost! {scope.view=shifted.into_inner();};
}

#[requires(scope.inner_logic().root_view_valid(*value) && amount<=value.len)]
#[ensures((^scope).root_view_valid(^value) && (^scope).scope==scope.inner_logic().scope)]
#[ensures((^value).original_shared==value.original_shared && (^value).data==value.data && (^value).vtable==value.vtable)]
#[ensures((^value).len@==value.len@-amount@ && (^value).ptr.addr_logic()@==value.ptr.addr_logic()@+amount@)]
#[ensures((^scope).view@==Some((scope.inner_logic().view@.unwrap_logic().0,
    scope.inner_logic().view@.unwrap_logic().1,scope.inner_logic().view@.unwrap_logic().2+amount@)))]
fn advance_root_view(value:&mut Bytes,amount:usize,scope:Ghost<&mut SuffixScope>) {
    assert!(amount<=value.len(),"cannot advance past `remaining`: {:?} <= {:?}",amount,value.len());
    unsafe {inc_start_root_view(value,amount,scope)}
}

#[requires(scope.inner_logic().root_view_valid(*value))]
#[ensures(result@==scope.inner_logic().root_view_content(value.len))]
#[ensures(result@.len()==value.len@)]
fn root_view_as_slice<'a>(value:&'a Bytes,scope:Ghost<&'a SuffixScope>)->&'a [u8] {
    let (bound,region)=ghost! {
        let region=match scope.scope.phase.as_ref().unwrap() {
            Phase::Raw(raw)=>&raw.physical,
            Phase::Shared(shared)=>{
                let full:&FullBorrow<raw_vec::PhysicalRegion>=(*shared.root.physical).to_ref();
                full.borrow(&shared.root.ticket.token)
            },
        };
        (&scope.view,region)
    }.split();
    unsafe {physical_projection::borrow(value.ptr,value.len,bound,region)}
}

#[requires(scope.inner_logic().root_view_valid(*value))]
#[ensures(result@==scope.inner_logic().root_view_content(value.len))]
fn chunk_root_view<'a>(value:&'a Bytes,scope:Ghost<&'a SuffixScope>)->&'a [u8] {
    root_view_as_slice(value,scope)
}

enum RootDropEffect { Raw(physical_projection::FreeReceipt), Shared(DetachedScope,Completion) }
impl RootDropEffect {
    #[logic(prophetic)] fn valid_for(self,before:SuffixScope)->bool {pearlite! {match self {
        Self::Raw(receipt)=>before.scope.is_raw() &&
            receipt.namespace()==before.scope.descriptor.base@.unwrap_logic().0 &&
            receipt.pointer()==before.scope.descriptor.base.raw_pointer() &&
            receipt.size()==before.scope.descriptor.capacity@ && receipt.align()==1 && receipt.allocated(),
        Self::Shared(detached,completion)=>before.scope.is_shared() &&
            detached.model()==before.scope.cursor_model() && detached.public()==before.scope.cursor_public() &&
            (*detached.observation()).0==(*before.scope.observation()).0.remove(before.scope.root_id()) &&
            (*detached.observation()).1==(*before.scope.observation()).1 &&
            completion.valid(before.scope.root_metadata()) &&
            completion.reclaimed()==((*before.scope.observation()).0.len()==1) &&
            (completion.reclaimed() ==> (*detached.observation()).0.len()==0),
    }}}
    #[logic] fn reclaimed(self)->bool {match self {Self::Raw(_)=>true,Self::Shared(_,c)=>c.reclaimed()}}
}

// No resource getter: the callback consumes the existing phase and splits its
// owned atomic permission from the actual remaining affine capabilities.
enum RootDropRemainder {
    Raw(raw_vec::Recovery,raw_vec::PhysicalRegion),
    Shared(SharedCore,Cursor),
}
type RootViewDropInput<'a>=(SuffixScope,&'a mut Option<RootDropEffect>);
type RootViewDropSpec<'a>=fn(&'a mut AtomicPtr<()>,*const u8,usize,Ghost<RootViewDropInput<'a>>);

#[requires(scope.inner_logic().root_view_valid(value))]
#[requires(*output.inner_logic()==None)]
#[ensures(^output!=None && (^output).unwrap_logic().valid_for(scope.inner_logic()))]
fn bytes_root_view_terminal_drop(mut value:Bytes,scope:Ghost<SuffixScope>,mut output:Ghost<&mut Option<RootDropEffect>>) {
    let native=value.vtable.drop;
    let descriptor=ghost! {match value.original_shared.into_inner() {
        OriginalSharedProof::Root(d)=>d,_=>{proof_assert!(false);panic!()},
    }};
    let spec=ghost! {
        let base=snapshot!(descriptor.base.raw_pointer()).into_ghost().into_inner();
        let address=crate::provenance_specs::pointer_addr(base);
        if address&1usize==0usize {even_root_view_drop_registration().into_inner()}
        else {odd_root_view_drop_registration().into_inner()}
    };
    erased_call::invoke3(native,(&mut value.data,value.ptr,value.len),ghost! {(scope.into_inner(),&mut **output)},spec);
}

#[logic(prophetic)]
fn root_peer_drop_input(value:Bytes,scope:SuffixScope)->bool {pearlite! {
    scope.valid() && scope.scope.is_shared() && value.api_view_valid() && value.view_owned() &&
    value.view_id()!=scope.scope.root_id() && value.view_public()==scope.scope.cursor_public() &&
    (*scope.scope.observation()).0.contains(scope.scope.root_id()) &&
    (*scope.scope.observation()).0.contains(value.view_id()) &&
    match (value.original_shared.inner_logic(),scope.scope.phase) {
        (OriginalSharedProof::Child(p),Some(Phase::Shared(s)))|
        (OriginalSharedProof::View(p,_),Some(Phase::Shared(s)))=>p.core.accepts(s.cursor),
        _=>false,
    }
}}

#[requires(root_peer_drop_input(value,*scope.inner_logic()))]
#[requires(*output.inner_logic()==None)]
#[ensures((^scope).valid() && (^scope).scope.is_shared())]
#[ensures((^scope).view==scope.inner_logic().view && (^scope).scope.descriptor==scope.inner_logic().scope.descriptor)]
#[ensures((^scope).scope.same_root(scope.inner_logic().scope) && (^scope).scope.same_pointer_owner(scope.inner_logic().scope))]
#[ensures((^scope).scope.cursor_model()==scope.inner_logic().scope.cursor_model() && (^scope).scope.cursor_public()==scope.inner_logic().scope.cursor_public())]
#[ensures((*((^scope).scope.observation())).0==(*scope.inner_logic().scope.observation()).0.remove(value.view_id()))]
#[ensures((*((^scope).scope.observation())).1==(*scope.inner_logic().scope.observation()).1)]
#[ensures(^output!=None && !(^output).unwrap_logic().reclaimed() && !(^output).unwrap_logic().was_static())]
#[ensures((^output).unwrap_logic().valid(scope.inner_logic().scope.root_metadata()))]
fn bytes_root_peer_terminal_drop(mut value:Bytes,mut scope:Ghost<&mut SuffixScope>,mut output:Ghost<&mut Option<ViewEffect>>) {
    // Two distinct live tickets leave two removals whose length is nonnegative.
    // This uses only the shipped finite-map laws, not a last-owner assumption.
    proof_assert!({
        let live=(*scope.inner_logic().scope.observation()).0;
        live.remove(value.view_id()).remove(scope.inner_logic().scope.root_id()).len()>=0
    });
    proof_assert!((*scope.inner_logic().scope.observation()).0.len()>1);
    let native=value.vtable.drop;
    let spec=ghost! {cursor_shared_drop_registration().into_inner()};
    let cursor=ghost! {match scope.scope.phase.as_mut().unwrap() {
        Phase::Shared(shared)=>&mut shared.cursor,
        _=>{proof_assert!(false);panic!()},
    }};
    erased_call::invoke3(native,(&mut value.data,value.ptr,value.len),
        ghost! {(value.original_shared.into_inner(),cursor.into_inner(),&mut **output)},spec);
}

/// Both native runtime branches join the same Root continuation. The peer is
/// consumed at its lexical block end, before the second public advance.
#[requires(input@.len()>0 && first@<=input@.len() && second@<=input@.len()-first@)]
#[ensures(result@==input@.subsequence(first@+second@,input@.len()))]
pub(crate) fn root_phase_scope(input:Box<[u8]>,first:usize,second:usize,promote:bool)->Vec<u8> {
    let expected=snapshot!(input@);
    let (mut value,scope)=from_box_scoped(input);
    let mut suffix=SuffixScope::new(scope);
    advance_root_view(&mut value,first,suffix.borrow_mut());
    if promote {
        let peer=clone_suffix_root(&value,suffix.borrow_mut());
        let root_id=snapshot!(suffix.scope.root_id());
        let peer_id=snapshot!(peer.view_id());
        proof_assert!(*root_id!=*peer_id && (*suffix.scope.observation()).0.len()==2);
        let mut peer_receipt=ghost! {None::<ViewEffect>};
        bytes_root_peer_terminal_drop(peer,suffix.borrow_mut(),peer_receipt.borrow_mut());
        proof_assert!(!peer_receipt.inner_logic().unwrap_logic().reclaimed());
        proof_assert!((*suffix.scope.observation()).0.len()==1);
        proof_assert!((*suffix.scope.observation()).0.contains(*root_id));
    }
    proof_assert!(suffix.root_view_valid(value));
    advance_root_view(&mut value,second,suffix.borrow_mut());
    let result=chunk_root_view(&value,suffix.borrow()).to_vec();
    let saved_return=result;
    let mut receipt=ghost! {None::<RootDropEffect>};
    bytes_root_view_terminal_drop(value,suffix,receipt.borrow_mut());
    proof_assert!(receipt.inner_logic()!=None && receipt.inner_logic().unwrap_logic().reclaimed());
    saved_return
}

#[requires(input.inner_logic().0.root_view_matches(offset,len,pointer_event::pointer_model(data),even_table()))]
#[requires(input.inner_logic().0.scope.descriptor.base.raw_pointer().addr_logic()&1usize==0usize)]
#[requires(*input.inner_logic().1==None)]
#[ensures(^input.inner_logic().1!=None && (^input.inner_logic().1).unwrap_logic().valid_for(input.inner_logic().0))]
fn even_root_view_drop_checked(data:&mut AtomicPtr<()>,offset:*const u8,len:usize,input:Ghost<RootViewDropInput>) {
    let (suffix,mut output)=input.split();
    let before=snapshot!(*suffix);
    let (descriptor,view,own,remainder)=ghost! {
        let mut suffix=suffix.into_inner();
        let (own,remainder)=match suffix.scope.phase.take().unwrap() {
            Phase::Raw(raw)=>(raw.own,RootDropRemainder::Raw(raw.recovery,raw.physical)),
            Phase::Shared(shared)=>(shared.own,RootDropRemainder::Shared(shared.root,shared.cursor)),
        };
        (suffix.scope.descriptor,suffix.view,own,remainder)
    }.split();
    let (word,_terminal_timestamp)=owned_pointer::get_mut_finish(data,own);
    let kind=crate::provenance_specs::pointer_addr(word)&1usize;
    if kind==0usize {
        let (root,mut cursor)=ghost! {match remainder.into_inner() {
            RootDropRemainder::Shared(root,cursor)=>(root,cursor),
            RootDropRemainder::Raw(_,_)=>{proof_assert!(false);panic!()},
        }}.split();
        proof_assert!(word==root.shared as *mut ());
        let mut completion=ghost! {None::<Completion>};
        release_core(word.cast(),root,cursor.borrow_mut(),completion.borrow_mut());
        ghost! {**output=Some(RootDropEffect::Shared(DetachedScope {cursor:cursor.into_inner()},completion.into_inner().unwrap()));};
    } else {
        debug_assert_eq!(kind,1usize);
        let (recovery,physical)=ghost! {match remainder.into_inner() {
            RootDropRemainder::Raw(recovery,physical)=>(recovery,physical),
            RootDropRemainder::Shared(_,_)=>{proof_assert!(false);panic!()},
        }}.split();
        proof_assert!(word==descriptor.word());
        let base=tag_specs::clear_low_bit(word,ghost! {&descriptor.base});
        let receipt=unsafe {free_raw_suffix_checked(base,offset,len,ghost! {
            (descriptor.into_inner(),view.into_inner(),recovery.into_inner(),physical.into_inner())
        })};
        ghost! {**output=Some(RootDropEffect::Raw(receipt.into_inner()));};
    }
}

#[trusted]
#[check(ghost)]
#[ensures(erased_call::registered3(even_table().drop,result.inner_logic()))]
#[ensures(forall<data:&mut AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<RootViewDropInput>>
    result.inner_logic().precondition((data,ptr,len,input))==even_root_view_drop_checked.precondition((data,ptr,len,input)))]
#[ensures(forall<data:&mut AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<RootViewDropInput>>
    result.inner_logic().postcondition((data,ptr,len,input),())==even_root_view_drop_checked.postcondition((data,ptr,len,input),()))]
fn even_root_view_drop_registration<'a>()->Ghost<RootViewDropSpec<'a>> {Ghost::conjure()}

#[requires(input.inner_logic().0.root_view_matches(offset,len,pointer_event::pointer_model(data),odd_table()))]
#[requires(input.inner_logic().0.scope.descriptor.base.raw_pointer().addr_logic()&1usize!=0usize)]
#[requires(*input.inner_logic().1==None)]
#[ensures(^input.inner_logic().1!=None && (^input.inner_logic().1).unwrap_logic().valid_for(input.inner_logic().0))]
fn odd_root_view_drop_checked(data:&mut AtomicPtr<()>,offset:*const u8,len:usize,input:Ghost<RootViewDropInput>) {
    let (suffix,mut output)=input.split();
    let before=snapshot!(*suffix);
    let (descriptor,view,own,remainder)=ghost! {
        let mut suffix=suffix.into_inner();
        let (own,remainder)=match suffix.scope.phase.take().unwrap() {
            Phase::Raw(raw)=>(raw.own,RootDropRemainder::Raw(raw.recovery,raw.physical)),
            Phase::Shared(shared)=>(shared.own,RootDropRemainder::Shared(shared.root,shared.cursor)),
        };
        (suffix.scope.descriptor,suffix.view,own,remainder)
    }.split();
    let (word,_terminal_timestamp)=owned_pointer::get_mut_finish(data,own);
    let kind=crate::provenance_specs::pointer_addr(word)&1usize;
    if kind==0usize {
        let (root,mut cursor)=ghost! {match remainder.into_inner() {
            RootDropRemainder::Shared(root,cursor)=>(root,cursor),
            RootDropRemainder::Raw(_,_)=>{proof_assert!(false);panic!()},
        }}.split();
        proof_assert!(word==root.shared as *mut ());
        let mut completion=ghost! {None::<Completion>};
        release_core(word.cast(),root,cursor.borrow_mut(),completion.borrow_mut());
        ghost! {**output=Some(RootDropEffect::Shared(DetachedScope {cursor:cursor.into_inner()},completion.into_inner().unwrap()));};
    } else {
        debug_assert_eq!(kind,1usize);
        let (recovery,physical)=ghost! {match remainder.into_inner() {
            RootDropRemainder::Raw(recovery,physical)=>(recovery,physical),
            RootDropRemainder::Shared(_,_)=>{proof_assert!(false);panic!()},
        }}.split();
        proof_assert!(word==descriptor.word());
        let base=word.cast::<u8>();
        let receipt=unsafe {free_raw_suffix_checked(base,offset,len,ghost! {
            (descriptor.into_inner(),view.into_inner(),recovery.into_inner(),physical.into_inner())
        })};
        ghost! {**output=Some(RootDropEffect::Raw(receipt.into_inner()));};
    }
}

#[trusted]
#[check(ghost)]
#[ensures(erased_call::registered3(odd_table().drop,result.inner_logic()))]
#[ensures(forall<data:&mut AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<RootViewDropInput>>
    result.inner_logic().precondition((data,ptr,len,input))==odd_root_view_drop_checked.precondition((data,ptr,len,input)))]
#[ensures(forall<data:&mut AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<RootViewDropInput>>
    result.inner_logic().postcondition((data,ptr,len,input),())==odd_root_view_drop_checked.postcondition((data,ptr,len,input),()))]
fn odd_root_view_drop_registration<'a>()->Ghost<RootViewDropSpec<'a>> {Ghost::conjure()}

