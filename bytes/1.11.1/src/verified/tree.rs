//! Finite scoped binary lease trees. Every internal node uses the proved pair
//! protocol; its full fraction and original ticket are restored before retiring
//! toward its parent. No dynamic atomic invariant payload map is introduced.
use super::*;

struct BranchContext<'a> {
    bytes: &'a [u8],
    parent_ticket: Ghost<Ticket>,
    anchor: Ghost<LifetimeToken>,
    original: Snapshot<(Lifetime, PositiveReal)>,
    children: CloseContext,
}

impl<'a> BranchContext<'a> {
    #[logic(open(self), prophetic)]
    fn restores_to(self, parent: CloseContext) -> bool {
        pearlite! {
            parent.machine.accepts(self.parent_ticket.inner_logic()) &&
            *self.original == if self.parent_ticket.is_left() {
                parent.machine.expected().0
            } else { parent.machine.expected().1 } &&
            self.original.1 <= PositiveReal::from_int(1) &&
            self.anchor.lft() == self.original.0 &&
            self.anchor.frac() == self.original.1 / PositiveReal::from_int(2) &&
            self.children.valid() &&
            self.children.machine.expected().0 ==
                (self.original.0,self.original.1 / PositiveReal::from_int(4)) &&
            self.children.machine.expected().1 ==
                (self.original.0,self.original.1 / PositiveReal::from_int(4))
        }
    }

    #[requires(self.restores_to(*parent))]
    #[requires(first.accepted_by(self.children) && second.accepted_by(self.children))]
    #[requires(first.is_left() != second.is_left())]
    #[ensures(result.accepted_by(*parent))]
    #[ensures(result.contents() == self.bytes@)]
    #[ensures(result.is_left() == self.parent_ticket.is_left())]
    fn finish(self, parent: &CloseContext, first: Closed, second: Closed) -> ReadHandle<'a> {
        let _=parent;
        self.children.machine.finish(first.last,first.receipt,second.last,second.receipt);
        let recovered=if first.last {first.recovered} else {second.recovered};
        let permit=ghost! {
            let (left,right)=recovered.into_inner().unwrap();
            let full=self.anchor.into_inner().join(left).join(right);
            proof_assert!(full.frac().ext_eq(self.original.1));
            full
        };
        ReadHandle{bytes:self.bytes,permit,ticket:self.parent_ticket}
    }
}

impl<'a> ReadHandle<'a> {
    #[requires(self.accepted_by(*parent))]
    #[ensures(result.0.contents() == self.contents() && result.1.contents() == self.contents())]
    #[ensures(result.2.bytes@ == self.contents())]
    #[ensures(result.2.restores_to(*parent))]
    #[ensures(result.0.accepted_by(result.2.children) && result.1.accepted_by(result.2.children))]
    #[ensures(result.0.is_left() && !result.1.is_left())]
    #[ensures(result.2.parent_ticket.is_left() == self.is_left())]
    fn branch(self, parent: &CloseContext) -> (ReadHandle<'a>,ReadHandle<'a>,BranchContext<'a>) {
        let _=parent;
        let original=snapshot!(self.permit.metadata());
        let pieces=ghost! {
            let (anchor,children)=self.permit.into_inner().split();
            (anchor,children.split())
        };
        let (anchor,pair)=pieces.split();
        let (left,right)=pair.split();
        let expected=snapshot!((left.metadata(),right.metadata()));
        let (machine,left_ticket,right_ticket)=SharedRetirement::new(expected);
        (ReadHandle{bytes:self.bytes,permit:left,ticket:left_ticket},
         ReadHandle{bytes:self.bytes,permit:right,ticket:right_ticket},
         BranchContext{bytes:self.bytes,parent_ticket:self.ticket,anchor,original,
            children:CloseContext{machine}})
    }
}

#[requires(0usize < leaves)]
#[requires(parent.valid() && handle.accepted_by(*parent))]
#[requires(tokens.contains(retirement::PUBLICATION()))]
#[ensures(result.0 == if index@ < handle.contents().len() {Some(handle.contents()[index@])} else {None})]
#[ensures(result.1 == leaves)]
#[ensures(result.2.accepted_by(*parent) && result.2.is_left() == handle.is_left())]
fn worker(handle:ReadHandle<'_>,parent:&CloseContext,tokens:Ghost<Tokens>,index:usize,leaves:usize)
    ->(Option<u8>,usize,Closed)
{
    use creusot_std::std::thread::{self,JoinHandleExt};
    if leaves == 1 {
        let byte=if index < handle.len() {Some(handle.read(index))} else {None};
        (byte,1,handle.close(parent,tokens))
    } else {
        let left_count=leaves/2;
        let right_count=leaves-left_count;
        // Structural descent is proved here. The stock thread contracts only
        // provide normal-return correctness, not scheduler termination.
        proof_assert!(0usize < left_count && left_count < leaves &&
            0usize < right_count && right_count < leaves);
        let (left,right,branch)=handle.branch(parent);
        let child_context=&branch.children;
        let (left,right)=thread::scope(move |scope| {
            let left=scope.spawn(move |tokens| worker(left,child_context,tokens,index,left_count));
            let right=scope.spawn(move |tokens| worker(right,child_context,tokens,index,right_count));
            (left.join_unwrap(),right.join_unwrap())
        });
        proof_assert!(left.0 == right.0);
        let restored=branch.finish(parent,left.2,right.2);
        (left.0,left.1+right.1,restored.close(parent,tokens))
    }
}

/// Explicit cleanup also covers runtime-rejected requests.
fn cleanup_vec(input:Vec<u8>) {
    let (base,_len,capacity,caps)=bound_ptr::detach_bound_vec(input);
    unsafe { raw_vec::deallocate_bound_vec(base,capacity,caps); }
}

/// Read through any finite number of scoped leaves. Counts below two are
/// rejected before detaching storage. Runtime resource exhaustion/panic is
/// outside the normal-return contract, as for the two-reader admission.
#[ensures((result == None) == (leaves < 2usize))]
#[ensures(result != None ==> result.unwrap_logic().0 ==
    if index@ < input@.len() {Some(input@[index@])} else {None})]
#[ensures(result != None ==> result.unwrap_logic().1 == leaves)]
#[ensures(result != None ==> result.unwrap_logic().2 != result.unwrap_logic().3)]
pub fn scoped_tree(input:Vec<u8>,index:usize,leaves:usize)
    ->Option<(Option<u8>,usize,bool,bool)>
{
    use creusot_std::std::thread::{self,JoinHandleExt};
    if leaves < 2 { cleanup_vec(input); return None; }
    let mut owner=Owner::new(input);
    let (left,right,context)=owner.share_pair();
    let left_count=leaves/2;
    let right_count=leaves-left_count;
    let parent=&context;
    let (left,right)=thread::scope(move |scope| {
        let left=scope.spawn(move |tokens| worker(left,parent,tokens,index,left_count));
        let right=scope.spawn(move |tokens| worker(right,parent,tokens,index,right_count));
        (left.join_unwrap(),right.join_unwrap())
    });
    proof_assert!(left.0 == right.0);
    let count=left.1+right.1;
    let flags=owner.close(context,left.2,right.2);
    Some((left.0,count,flags.0,flags.1))
}

#[cfg(all(test,not(creusot)))]
mod tests {
    use super::*;
    #[test]
    fn finite_tree_reads_and_cleanup() {
        for leaves in [0,1,2,3,4,5,7,8,13,17] {
            for len in [0,1,5] { for spare in [0,8] { for index in [0,4,8] {
                let mut input=Vec::with_capacity(len+spare);
                input.extend((0..len).map(|i| (71+i) as u8));
                let expected=input.get(index).copied();
                let result=scoped_tree(input,index,leaves);
                if leaves < 2 {assert!(result.is_none());}
                else {let (byte,count,a,b)=result.unwrap();assert_eq!(byte,expected);assert_eq!(count,leaves);assert_ne!(a,b);}
            }}}
        }
    }
}
