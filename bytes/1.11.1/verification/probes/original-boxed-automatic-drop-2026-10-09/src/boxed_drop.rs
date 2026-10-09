use creusot_std::std::ops::FnExt;
type BoxedMetadata = Option<(Id,*mut u8,Int)>;
pub(crate) enum BoxedCompletion { NoAllocation, Freed(physical_projection::FreeReceipt) }
impl BoxedCompletion {
    #[logic] fn freed(self)->bool { match self {Self::NoAllocation=>false,Self::Freed(_)=>true} }
    #[logic] fn valid(self,metadata:BoxedMetadata)->bool {
        pearlite! {match (self,metadata) {
            (Self::NoAllocation,None)=>true,
            (Self::Freed(r),Some((id,pointer,size)))=>r.namespace()==id && r.pointer()==pointer && r.size()==size && r.align()==1 && r.allocated(),
            _=>false
        }}
    }
}
impl PromotableRawProof {
    #[logic] fn metadata(self)->BoxedMetadata { pearlite! {Some((self.capabilities.inner_logic().0.namespace(),self.base.raw_pointer(),self.capacity@))} }
}
impl Bytes {
    #[logic] fn static_repr(self)->bool {match self.original_bytes.inner_logic() {OriginalBytesProof::Static(_)=>true,_=>false}}
    #[logic] fn boxed_only(self)->bool {match self.original_bytes.inner_logic() {
        OriginalBytesProof::Shared(_)=>false,
        OriginalBytesProof::PromotableRaw(_)=>true,
        OriginalBytesProof::Static(_)=>self.len==0usize,
    }}
    #[logic] fn boxed_metadata(self)->BoxedMetadata {match self.original_bytes.inner_logic() {
        OriginalBytesProof::PromotableRaw(p)=>p.metadata(),_=>None
    }}
}

type StaticInput<'a>=(StaticBytesProof,&'a mut Option<BoxedCompletion>);
type StaticSpec<'a>=fn(&'a mut AtomicPtr<()>,*const u8,usize,Ghost<StaticInput<'a>>);
#[requires(input.inner_logic().0.valid_for(offset,len,pointer_event::pointer_model(data)))]
#[requires(len==0usize && *input.inner_logic().1==None)]
#[ensures(^input.inner_logic().1 == Some(BoxedCompletion::NoAllocation))]
fn static_drop_checked(data:&mut AtomicPtr<()>,offset:*const u8,len:usize,input:Ghost<StaticInput>) {
    let (_,mut output)=input.split();
    ghost! {**output=Some(BoxedCompletion::NoAllocation);};
}
#[trusted]
#[ensures(result.0==static_table())]
#[ensures(erased_call::registered3(result.0.drop,result.1.inner_logic()))]
#[ensures(forall<data:&mut AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<StaticInput>> result.1.inner_logic().precondition((data,ptr,len,input))==static_drop_checked.precondition((data,ptr,len,input)))]
#[ensures(forall<data:&mut AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<StaticInput>> result.1.inner_logic().postcondition((data,ptr,len,input),())==static_drop_checked.postcondition((data,ptr,len,input),()))]
fn static_drop_registration<'a>()->(&'static Vtable,Ghost<StaticSpec<'a>>) {unreachable!("closed native vtable ghost-erasure reification")}

type EvenInput<'a>=(PromotableRawProof,&'a mut Option<BoxedCompletion>);
type EvenSpec<'a>=fn(&'a mut AtomicPtr<()>,*const u8,usize,Ghost<EvenInput<'a>>);
#[requires(input.inner_logic().0.valid_for(offset,len,pointer_event::pointer_model(data),promotable_even_table()))]
#[requires(offset.addr_logic() & 1usize == 0usize)]
#[requires(*input.inner_logic().1==None)]
#[ensures(^input.inner_logic().1 != None && (^input.inner_logic().1).unwrap_logic().valid(input.inner_logic().0.metadata()))]
fn even_drop_checked(data:&mut AtomicPtr<()>,offset:*const u8,len:usize,input:Ghost<EvenInput>) {
    let (proof,mut output)=input.split();
    let stored=pointer_event::get_mut(data,ghost! {&*proof.data_binding});
    let kind=crate::provenance_specs::pointer_addr(stored) & 1usize;
    #[cfg(feature="negative_raw_as_arc")] let kind=0usize;
    if kind==0 {
        // Original native KIND_ARC release_shared branch is proved unreachable.
        proof_assert!(false);
        unreachable!("readonly raw binding excludes native ARC branch");
    } else {
        debug_assert_eq!(kind,1usize);
        #[cfg(not(feature="negative_missing_untag"))]
        let buf=tag_specs::clear_low_bit(stored,ghost! {&proof.base});
        #[cfg(feature="negative_missing_untag")]
        let buf=stored.cast::<u8>();
        let receipt=free_boxed_slice_checked(buf,offset,len,proof);
        #[cfg(not(feature="negative_raw_as_static"))]
        ghost! {**output=Some(BoxedCompletion::Freed(receipt.into_inner()));};
        #[cfg(feature="negative_raw_as_static")]
        ghost! {**output=Some(BoxedCompletion::NoAllocation);};
    }
}

#[trusted]
#[ensures(result.0 == promotable_even_table())]
#[ensures(erased_call::registered3(result.0.drop,result.1.inner_logic()))]
#[ensures(forall<data:&mut AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<EvenInput>> result.1.inner_logic().precondition((data,ptr,len,input))==even_drop_checked.precondition((data,ptr,len,input)))]
#[ensures(forall<data:&mut AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<EvenInput>> result.1.inner_logic().postcondition((data,ptr,len,input),())==even_drop_checked.postcondition((data,ptr,len,input),()))]
fn even_drop_registration<'a>()->(&'static Vtable,Ghost<EvenSpec<'a>>) {unreachable!("closed native vtable ghost-erasure reification")}

type OddInput<'a>=(PromotableRawProof,&'a mut Option<BoxedCompletion>);
type OddSpec<'a>=fn(&'a mut AtomicPtr<()>,*const u8,usize,Ghost<OddInput<'a>>);
#[requires(input.inner_logic().0.valid_for(offset,len,pointer_event::pointer_model(data),promotable_odd_table()))]
#[requires(offset.addr_logic() & 1usize != 0usize)]
#[requires(*input.inner_logic().1==None)]
#[ensures(^input.inner_logic().1 != None && (^input.inner_logic().1).unwrap_logic().valid(input.inner_logic().0.metadata()))]
fn odd_drop_checked(data:&mut AtomicPtr<()>,offset:*const u8,len:usize,input:Ghost<OddInput>) {
    let (proof,mut output)=input.split();
    let stored=pointer_event::get_mut(data,ghost! {&*proof.data_binding});
    let kind=crate::provenance_specs::pointer_addr(stored) & 1usize;
    if kind==0 {
        // Original native KIND_ARC release_shared branch is proved unreachable.
        proof_assert!(false);
        unreachable!("readonly raw binding excludes native ARC branch");
    } else {
        debug_assert_eq!(kind,1usize);
        let buf=stored.cast::<u8>();
        let receipt=free_boxed_slice_checked(buf,offset,len,proof);
        ghost! {**output=Some(BoxedCompletion::Freed(receipt.into_inner()));};
    }
}

#[trusted]
#[ensures(result.0 == promotable_odd_table())]
#[ensures(erased_call::registered3(result.0.drop,result.1.inner_logic()))]
#[ensures(forall<data:&mut AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<OddInput>> result.1.inner_logic().precondition((data,ptr,len,input))==odd_drop_checked.precondition((data,ptr,len,input)))]
#[ensures(forall<data:&mut AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<OddInput>> result.1.inner_logic().postcondition((data,ptr,len,input),())==odd_drop_checked.postcondition((data,ptr,len,input),()))]
fn odd_drop_registration<'a>()->(&'static Vtable,Ghost<OddSpec<'a>>) {unreachable!("closed native vtable ghost-erasure reification")}

#[requires(proof.inner_logic().base.invariant())]
#[requires(proof.inner_logic().capabilities.inner_logic().0.invariant() && proof.inner_logic().capabilities.inner_logic().1.invariant())]
#[requires(proof.inner_logic().base@ == Some((proof.inner_logic().capabilities.inner_logic().0.namespace(),len@,0int)))]
#[requires(proof.inner_logic().capacity==len && len>0usize)]
#[requires(proof.inner_logic().capabilities.inner_logic().0.capacity()==len@ && proof.inner_logic().capabilities.inner_logic().1.capacity()==len@)]
#[requires(proof.inner_logic().capabilities.inner_logic().1.namespace()==proof.inner_logic().capabilities.inner_logic().0.namespace())]
#[requires(proof.inner_logic().capabilities.inner_logic().1.resource_id()==proof.inner_logic().capabilities.inner_logic().0.namespace())]
#[requires(proof.inner_logic().capabilities.inner_logic().1.lo()==0 && proof.inner_logic().capabilities.inner_logic().1.hi()==len@)]
#[requires(buf==proof.inner_logic().base.raw_pointer() && offset==buf as *const u8)]
#[ensures(BoxedCompletion::Freed(result.inner_logic()).valid(proof.inner_logic().metadata()))]
fn free_boxed_slice_checked(buf:*mut u8,offset:*const u8,len:usize,proof:Ghost<PromotableRawProof>)->Ghost<physical_projection::FreeReceipt> {
    let distance=unsafe {tag_specs::equal_pointer_distance(offset,buf)};
    let cap=distance as usize + len;
    #[cfg(feature="negative_wrong_free_size")] let cap=cap-1;
    #[cfg(feature="negative_wrong_free_pointer")] let buf=buf.wrapping_add(1);
    #[cfg(not(feature="negative_missing_free"))]
    {
        let receipt=unsafe {physical_projection::deallocate(buf,cap,ghost! {proof.base},ghost! {proof.into_inner().capabilities.into_inner()})};
        #[cfg(feature="negative_duplicate_free")]
        let _second=unsafe {physical_projection::deallocate(buf,cap,ghost! {proof.base},ghost! {proof.into_inner().capabilities.into_inner()})};
        receipt
    }
    #[cfg(feature="negative_missing_free")]
    {Ghost::conjure()}
}

#[requires(value.original_bytes_valid() && value.boxed_only())]
#[requires(*output.inner_logic()==None)]
#[ensures(^output != None && (^output).unwrap_logic().valid(value.boxed_metadata()))]
fn bytes_terminal_drop(mut value:Bytes,mut output:Ghost<&mut Option<BoxedCompletion>>) {
    let native=value.vtable.drop;
    if value.len==0 {
        let proof=ghost! {match value.original_bytes.into_inner() {OriginalBytesProof::Static(p)=>p,_=>{proof_assert!(false);panic!()}}};
        let (table,spec)=static_drop_registration();
        proof_assert!(value.vtable==table);
        erased_call::invoke3(native,(&mut value.data,value.ptr,value.len),ghost! {(proof.into_inner(),&mut **output)},spec);
    } else {
        let proof=ghost! {match value.original_bytes.into_inner() {OriginalBytesProof::PromotableRaw(p)=>p,_=>{proof_assert!(false);panic!()}}};
        let address=crate::provenance_specs::pointer_addr(value.ptr);
        if address & 1usize == 0usize {
            let (table,spec)=even_drop_registration();
            proof_assert!(value.vtable==table);
            erased_call::invoke3(native,(&mut value.data,value.ptr,value.len),ghost! {(proof.into_inner(),&mut **output)},spec);
        } else {
            let (table,spec)=odd_drop_registration();
            proof_assert!(value.vtable==table);
            erased_call::invoke3(native,(&mut value.data,value.ptr,value.len),ghost! {(proof.into_inner(),&mut **output)},spec);
        }
    }
}
