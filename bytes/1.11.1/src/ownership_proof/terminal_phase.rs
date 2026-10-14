//! One-shot authority for the terminal lifecycle transition of a shared header.
//!
//! The phase authority and its affine action share one resource identity. The
//! authority remains available after closure so that stale actions can be
//! ruled out by the empty phase.

use creusot_std::{
    ghost::resource::{Authority, Fragment},
    logic::{
        ra::{auth::CancelLocalUpdateUnit, excl::Excl, RA},
        Id,
    },
    prelude::*,
};

type PhaseValue = Option<Excl<()>>;

/// The unique one-shot permission to close a phase authority.
#[allow(missing_debug_implementations)]
pub(crate) struct TerminalAction {
    resource_id: Id,
    fragment: Fragment<PhaseValue>,
}

impl TerminalAction {
    #[logic(open(self))]
    pub(crate) fn resource_id(self) -> Id {
        self.resource_id
    }

    #[logic(prophetic)]
    fn valid(self) -> bool {
        pearlite! {
            self.fragment.id() == self.resource_id &&
            self.fragment@ == Some(Excl(()))
        }
    }

    /// Two actual close fragments for the same phase cannot coexist. This is
    /// used to rule out a still-live payload state when the unique terminal
    /// action has already been returned to the finalizer.
    #[check(ghost)]
    #[requires((*self).invariant() && other.invariant())]
    #[requires((*self).resource_id() == other.resource_id())]
    #[ensures(false)]
    pub(crate) fn contradicts_other(&mut self, other: &Self) {
        self.fragment.valid_op_lemma(&other.fragment);
        proof_assert!(self.fragment@.op(other.fragment@) != None);
        proof_assert!(self.fragment@.op(other.fragment@) == None);
        proof_assert!(false);
    }
}

impl Invariant for TerminalAction {
    #[logic(open(self), prophetic)]
    fn invariant(self) -> bool {
        self.valid()
    }
}

/// Persistent authority whose state is either open or irreversibly closed.
#[allow(missing_debug_implementations)]
pub(crate) struct PhaseAuthority {
    resource_id: Id,
    authority: Authority<PhaseValue>,
}

impl PhaseAuthority {
    #[logic(open(self))]
    pub(crate) fn resource_id(self) -> Id {
        self.resource_id
    }

    #[logic(open(self))]
    pub(crate) fn is_open(self) -> bool {
        pearlite! { self.authority@ == Some(Excl(())) }
    }

    #[logic(open(self))]
    pub(crate) fn is_closed(self) -> bool {
        pearlite! { self.authority@ == None }
    }

    #[logic(open(self), prophetic)]
    fn phase_valid(self) -> bool {
        pearlite! {
            self.authority.invariant() && self.authority.id() == self.resource_id &&
            (self.is_open() || self.is_closed())
        }
    }

    /// Allocate one phase authority and its matching affine close action.
    #[check(ghost)]
    #[ensures(result.inner_logic().0.invariant())]
    #[ensures(result.inner_logic().0.is_open())]
    #[ensures(result.inner_logic().1.invariant())]
    #[ensures(result.inner_logic().1.resource_id() == result.inner_logic().0.resource_id())]
    pub(crate) fn new() -> Ghost<(Self, TerminalAction)> {
        ghost! {
            let mut authority: Authority<PhaseValue> = Authority::alloc().into_inner();
            let open_value: Snapshot<PhaseValue> = snapshot!(Some(Excl(())));
            let fragment = authority.add_fragment(open_value);
            let resource_id = authority.id_ghost();
            let phase = PhaseAuthority { resource_id, authority };
            let action = TerminalAction { resource_id, fragment };
            proof_assert!(phase.invariant() && phase.phase_valid() && phase.is_open());
            proof_assert!(action.invariant());
            (phase, action)
        }
    }

    /// Consume the unique close action while retaining the empty authority.
    #[check(ghost)]
    #[requires((*self).invariant() && (*self).is_open())]
    #[requires(action.invariant() && action.resource_id() == (*self).resource_id())]
    #[ensures((^self).invariant() && (^self).is_closed())]
    #[ensures((^self).resource_id() == (*self).resource_id())]
    pub(crate) fn close(&mut self, action: TerminalAction) {
        proof_assert!((*self).phase_valid());
        let TerminalAction {
            resource_id: _,
            mut fragment,
        } = action;
        self.authority.frag_lemma(&fragment);
        self.authority.update(&mut fragment, CancelLocalUpdateUnit);
        proof_assert!(fragment@ == None);
        proof_assert!(self.authority@ == None);
    }

    /// A valid action for this exact phase forces the authority to remain
    /// open; the closed state cannot compose with its real exclusive fragment.
    #[check(ghost)]
    #[requires(self.invariant())]
    #[requires(action.invariant() && action.resource_id() == self.resource_id())]
    #[ensures(self.is_open())]
    pub(crate) fn require_action(&self, action: &TerminalAction) {
        let _ = ghost! {
            self.authority.frag_lemma(&action.fragment);
            let action_fragment: Snapshot<PhaseValue> = snapshot!(action.fragment@);
            let phase_authority: Snapshot<PhaseValue> = snapshot!(self.authority@);
            proof_assert!((*action_fragment).incl(*phase_authority));
            proof_assert!((*action_fragment).incl_eq(Some(Excl(()))));
            proof_assert!(*phase_authority == Some(Excl(())));
            proof_assert!(self.is_open());
        };
    }

    /// A valid action with this resource identity cannot survive closure.
    #[check(ghost)]
    #[requires((*self).invariant() && (*self).is_closed())]
    #[requires(action.invariant() && action.resource_id() == (*self).resource_id())]
    #[ensures(false)]
    pub(crate) fn contradicts_action(&self, action: &TerminalAction) {
        proof_assert!((*self).phase_valid());
        self.authority.frag_lemma(&action.fragment);
        let action_fragment: Snapshot<PhaseValue> = snapshot!(action.fragment@);
        let phase_authority: Snapshot<PhaseValue> = snapshot!(self.authority@);
        proof_assert!((*action_fragment).incl(*phase_authority));
        proof_assert!((*action_fragment).incl_eq(Some(Excl(()))));
        proof_assert!(*phase_authority == None);
        proof_assert!(!(*action_fragment).incl(*phase_authority));
        proof_assert!(false);
    }
}

impl Invariant for PhaseAuthority {
    #[logic(open(self), prophetic)]
    fn invariant(self) -> bool {
        self.phase_valid()
    }
}
