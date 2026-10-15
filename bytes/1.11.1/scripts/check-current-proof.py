#!/usr/bin/env python3
"""Save/check direct-source proof evidence without invoking a solver."""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import shutil
import subprocess
import sys

TARGETS = (
    "verif/bytes_rlib/bytes_mut/impl_BytesMut_0/len.coma",
    "verif/bytes_rlib/bytes_mut/impl_BytesMut_0/truncate.coma",
    "verif/bytes_rlib/bytes_mut/impl_BytesMut_0/as_slice.coma",
    "verif/bytes_rlib/bytes_mut/impl_BytesMut_0/set_len.coma",
    "verif/bytes_rlib/bytes_mut/impl_BytesMut_0/from_vec.coma",
    "verif/bytes_rlib/bytes_mut/impl_BytesMut_0/with_capacity.coma",
    "verif/bytes_rlib/bytes_mut/impl_BytesMut_0/new.coma",
    "verif/bytes_rlib/bytes_mut/impl_BytesMut_0/is_empty.coma",
    "verif/bytes_rlib/bytes_mut/impl_BytesMut_0/zeroed.coma",
    "verif/bytes_rlib/bytes_mut/impl_BytesMut_0/clear.coma",
    "verif/bytes_rlib/bytes_mut/impl_BytesMut_0/set_vec_pos.coma",
    "verif/bytes_rlib/bytes_mut/impl_BytesMut_0/get_vec_pos.coma",
    "verif/bytes_rlib/bytes_mut/impl_BytesMut_0/set_visible_len.coma",
    "verif/bytes_rlib/bytes_mut/impl_BytesMut_0/advance_vec_layout.coma",
    "verif/bytes_rlib/bytes_mut/impl_BytesMut_0/advance_shared_layout.coma",
    "verif/bytes_rlib/bytes_mut/impl_BytesMut_0/advance_unchecked.coma",
    "verif/bytes_rlib/bytes_mut/impl_BytesMut_0/promote_to_shared_count_one.coma",
    "verif/bytes_rlib/bytes_mut/impl_BytesMut_0/split_unique_to_shared.coma",
    "verif/bytes_rlib/bytes_mut/impl_BytesMut_0/install_shared_split_views.coma",
    "verif/bytes_rlib/bytes_mut/impl_BytesMut_0/split_shared_view.coma",
    "verif/bytes_rlib/bytes_mut/prove_split_views_preserve_prefix.coma",
    "verif/bytes_rlib/bytes_mut/shared_capsule/prove_mutable_split_right_slots_known.coma",
    "verif/bytes_rlib/bytes_mut/impl_BytesMut_0/split_off.coma",
    "verif/bytes_rlib/bytes_mut/impl_BytesMut_0/split_to.coma",
    "verif/bytes_rlib/bytes_mut/impl_BytesMut_0/split.coma",
    "verif/bytes_rlib/bytes/impl_Bytes_1/with_vtable.coma",
    "verif/bytes_rlib/bytes/impl_Bytes_1/dispose_shared.coma",
    "verif/bytes_rlib/bytes/impl_Bytes_1/dispose_boxed.coma",
    "verif/bytes_rlib/bytes/impl_Bytes_1/as_slice.coma",
    "verif/bytes_rlib/bytes/impl_Bytes_1/copy_shared_readonly_to_vec_with_context.coma",
    "verif/bytes_rlib/bytes/tagged_vec_data_pointer.coma",
    "verif/bytes_rlib/bytes/untagged_vec_data_pointer.coma",
    "verif/bytes_rlib/bytes/boxed_slice_capacity_from_addresses.coma",
    "verif/bytes_rlib/bytes/free_boxed_slice.coma",
    "verif/bytes_rlib/bytes/promotable_from_raw_parts.coma",
    "verif/bytes_rlib/bytes/from_boxed_slice_inner.coma",
    "verif/bytes_rlib/bytes/from_full_capacity_vec.coma",
    "verif/bytes_rlib/bytes/from_spare_capacity_vec.coma",
    "verif/bytes_rlib/loom/sync/atomic/impl_NativePtr_T/into_raw_with_permission.coma",
    "verif/bytes_rlib/bytes_mut/impl_BytesMut_0/kind.coma",
    "verif/bytes_rlib/bytes_mut/impl_Default_for_BytesMut/default.coma",
    "verif/bytes_rlib/bytes_mut/impl_Default_for_BytesMut/default__refines.coma",
    "verif/bytes_rlib/bytes_mut/impl_From_for_BytesMut/from.coma",
    "verif/bytes_rlib/bytes_mut/impl_From_for_BytesMut/from__refines.coma",
    "verif/bytes_rlib/bytes_mut/impl_From_for_BytesMut_0/from.coma",
    "verif/bytes_rlib/bytes_mut/impl_From_for_BytesMut_0/from__refines.coma",
    "verif/bytes_rlib/bytes_mut/original_capacity_to_repr.coma",
    "verif/bytes_rlib/bytes_mut/original_capacity_from_repr.coma",
    "verif/bytes_rlib/capacity_ops/original_capacity_repr_from_data.coma",
    "verif/bytes_rlib/bytes_mut/unique_initialized_prefix_extend.coma",
    "verif/bytes_rlib/bytes_mut/buffer_initialized_prefix_extend.coma",
    "verif/bytes_rlib/bytes_mut/buffer_prefix_mode_equivalence.coma",
    "verif/bytes_rlib/bytes_mut/unique_initialized_prefix_suffix.coma",
    "verif/bytes_rlib/bytes_mut/impl_BufMut_for_BytesMut/advance_mut.coma",
    "verif/bytes_rlib/bytes_mut/vptr.coma",
    "verif/bytes_rlib/bytes_mut/invalid_ptr.coma",
    "verif/bytes_rlib/provenance_specs/signed_wrapping_offset.coma",
    "verif/bytes_rlib/bytes_mut/impl_BytesMut_2/unique_rebuild_base_pointer.coma",
    "verif/bytes_rlib/bytes_mut/impl_BytesMut_2/rebuild_vec.coma",
    "verif/bytes_rlib/bytes_mut/impl_BytesMut_1/dispose_unique.coma",
    "verif/bytes_rlib/ownership_proof/view_region/split_view_region.coma",
    "verif/bytes_rlib/ownership_proof/ref_count_ledger/impl_OwnerLedger/new_root.coma",
    "verif/bytes_rlib/ownership_proof/ref_count_ledger/impl_OwnerLedger/new_promoted_pair.coma",
    "verif/bytes_rlib/ownership_proof/ref_count_ledger/impl_OwnerLedger/register_readonly.coma",
    "verif/bytes_rlib/ownership_proof/ref_count_ledger/impl_OwnerLedger/reclassify.coma",
    "verif/bytes_rlib/ownership_proof/ref_count_ledger/impl_OwnerLedger/split_owner.coma",
    "verif/bytes_rlib/ownership_proof/ref_count_ledger/impl_Clone_for_RefKind/clone.coma",
    "verif/bytes_rlib/ownership_proof/ref_count_ledger/impl_OwnerTicket/from_fragment.coma",
    "verif/bytes_rlib/ownership_proof/ref_count_ledger/impl_OwnerTicket/replace_binding.coma",
    "verif/bytes_rlib/ownership_proof/ref_count_ledger/impl_LocalUpdate_for_OwnerBindingUpdate/frame_preserving__refines.coma",
    "verif/bytes_rlib/ownership_proof/ref_count_ledger/impl_OwnerLedger/drop_owner.coma",
    "verif/bytes_rlib/ownership_proof/ref_count_ledger/impl_OwnerLedger/agrees_with_value.coma",
    "verif/bytes_rlib/ownership_proof/ref_count_ledger/impl_OwnerLedger/prove_ticket_is_active.coma",
    "verif/bytes_rlib/ownership_proof/ref_count_ledger/impl_OwnerLedger/prove_readonly_owner_interval_excluded_from_mutable_coverage.coma",
    "verif/bytes_rlib/ownership_proof/ref_count_ledger/impl_OwnerLedger/prove_readonly_binding_compatible.coma",
    "verif/bytes_rlib/ownership_proof/ref_count_ledger/impl_OwnerLedger/prove_readonly_ticket_excluded_from_mutable_coverage.coma",
    "verif/bytes_rlib/ownership_proof/ref_count_ledger/impl_OwnerLedger/prove_mutable_ticket_covers_interval.coma",
    "verif/bytes_rlib/ownership_proof/ref_count_ledger/prove_owner_removal_effect.coma",
    "verif/bytes_rlib/ownership_proof/ref_count_ledger/prove_owner_reclassification_effect.coma",
    "verif/bytes_rlib/ownership_proof/ref_count_ledger/prove_owner_split_preserves_mutable_coverage.coma",
    "verif/bytes_rlib/ownership_proof/ref_count_ledger/prove_no_active_mutable_on_empty.coma",
    "verif/bytes_rlib/ownership_proof/ref_count_ledger/contradict_two_tickets.coma",
    "verif/bytes_rlib/ownership_proof/ref_count_ledger/impl_RetirePermit/contradicts_live_ticket.coma",
    "verif/bytes_rlib/ownership_proof/ref_count_ledger/map_op_fresh_singleton.coma",
    "verif/bytes_rlib/ownership_proof/ref_count_ledger/factor_singleton_removal.coma",
    "verif/bytes_rlib/ownership_proof/ref_count_ledger/sole_active_id.coma",
    "verif/bytes_rlib/ownership_proof/ref_count_ledger/map_singleton_from_len_one.coma",
    "verif/bytes_rlib/ownership_proof/ref_count_ledger/normalize_singleton_insert.coma",
    "verif/bytes_rlib/ownership_proof/ref_count_ledger/zero_length_map_has_no_ids.coma",
    "verif/bytes_rlib/ownership_proof/ready_registry/impl_Clone_for_ReadyKey/clone.coma",
    "verif/bytes_rlib/ownership_proof/ready_registry/impl_ReadyCap_R/state.coma",
    "verif/bytes_rlib/ownership_proof/ready_registry/impl_ReadyCap_R/fragment_ref.coma",
    "verif/bytes_rlib/ownership_proof/ready_registry/impl_ReadyCap_R/update.coma",
    "verif/bytes_rlib/ownership_proof/ready_registry/impl_ReadyCap_R/key_ghost.coma",
    "verif/bytes_rlib/ownership_proof/ready_registry/impl_ReadyRegistry_R/new.coma",
    "verif/bytes_rlib/ownership_proof/ready_registry/impl_ReadyRegistry_R/entry_state.coma",
    "verif/bytes_rlib/ownership_proof/ready_registry/impl_ReadyRegistry_R/register.coma",
    "verif/bytes_rlib/ownership_proof/ready_registry/impl_ReadyRegistry_R/take.coma",
    "verif/bytes_rlib/ownership_proof/ready_registry/impl_ReadyRegistry_R/return_cap.coma",
    "verif/bytes_rlib/ownership_proof/ready_registry/impl_ReadyRegistry_R/len.coma",
    "verif/bytes_rlib/ownership_proof/ready_registry/impl_ReadyRegistry_R/has_key.coma",
    "verif/bytes_rlib/ownership_proof/shared_context/impl_OwnedSharedContext_R/from_tokens.coma",
    "verif/bytes_rlib/ownership_proof/shared_context/impl_OwnedSharedContext_R/reborrow_view.coma",
    "verif/bytes_rlib/ownership_proof/shared_context/impl_BorrowedSharedContext_R/reborrow_tokens.coma",
    "verif/bytes_rlib/ownership_proof/shared_context/impl_BorrowedSharedContext_R/register.coma",
    "verif/bytes_rlib/ownership_proof/shared_context/impl_BorrowedSharedContext_R/take.coma",
    "verif/bytes_rlib/ownership_proof/shared_context/impl_BorrowedSharedContext_R/return_cap.coma",
    "verif/bytes_rlib/ownership_proof/shared_context/impl_BorrowedSharedContext_R/reborrow.coma",
    "verif/bytes_rlib/ownership_proof/shared_context/impl_BorrowedSharedContext_R/ready_has_key_ghost.coma",
    "verif/bytes_rlib/ownership_proof/shared_context/impl_BorrowedSharedContext_R/prove_ready_has_key_from_entry.coma",
    "verif/bytes_rlib/ownership_proof/shared_context/impl_BorrowedSharedContext_R/prove_tombstone_has_no_key.coma",
    "verif/bytes_rlib/ownership_proof/shared_context/impl_BorrowedSharedContext_R/registered_cap.coma",
    "verif/bytes_rlib/ownership_proof/shared_context/impl_BorrowedSharedContext_R/unchanged.coma",
    "verif/bytes_rlib/ownership_proof/terminal_phase/impl_PhaseAuthority/new.coma",
    "verif/bytes_rlib/ownership_proof/terminal_phase/impl_PhaseAuthority/close.coma",
    "verif/bytes_rlib/ownership_proof/terminal_phase/impl_PhaseAuthority/require_action.coma",
    "verif/bytes_rlib/ownership_proof/terminal_phase/impl_PhaseAuthority/contradicts_action.coma",
    "verif/bytes_rlib/ownership_proof/terminal_phase/impl_TerminalAction/contradicts_other.coma",
    "verif/bytes_rlib/bytes_mut/shared_capsule/make_count_one_packet.coma",
    "verif/bytes_rlib/bytes_mut/shared_capsule/make_count_two_packets.coma",
    "verif/bytes_rlib/bytes_mut/shared_capsule/mutable_owner_ticket_split_result.coma",
    "verif/bytes_rlib/bytes_mut/shared_capsule/mutable_owner_region_split_bounds_result.coma",
    "verif/bytes_rlib/bytes_mut/shared_capsule/mutable_owner_region_split_slots_result.coma",
    "verif/bytes_rlib/bytes_mut/shared_capsule/mutable_owner_split_result.coma",
    "verif/bytes_rlib/bytes_mut/shared_capsule/impl_HeaderLeaseAuthority/new.coma",
    "verif/bytes_rlib/bytes_mut/shared_capsule/impl_HeaderLeaseAuthority/activate.coma",
    "verif/bytes_rlib/bytes_mut/shared_capsule/impl_HeaderLeaseAuthority/require_active_fragment.coma",
    "verif/bytes_rlib/bytes_mut/shared_capsule/impl_HeaderLeaseAuthority/require_idle_cap.coma",
    "verif/bytes_rlib/bytes_mut/shared_capsule/impl_HeaderLeaseAuthority/deactivate.coma",
    "verif/bytes_rlib/bytes_mut/shared_capsule/impl_SharedHeaderReadLoan/lent_region_snapshot.coma",
    "verif/bytes_rlib/bytes_mut/shared_capsule/impl_SharedPayload/prove_public_lease_key.coma",
    "verif/bytes_rlib/bytes_mut/shared_capsule/impl_SharedPayload/prove_public_lease_resource_id.coma",
    "verif/bytes_rlib/bytes_mut/shared_capsule/impl_SharedPayload/prove_public_phase_id.coma",
    "verif/bytes_rlib/bytes_mut/shared_capsule/impl_SharedPayload/prove_root_some_implies_lease_idle.coma",
    "verif/bytes_rlib/bytes_mut/shared_capsule/impl_SharedHeaderReadLoan/from_active_parts.coma",
    "verif/bytes_rlib/bytes_mut/shared_capsule/impl_SharedPayload/prove_stable_active_pool_protocol.coma",
    "verif/bytes_rlib/bytes_mut/shared_capsule/impl_SharedPayload/take_header_region_read_loan.coma",
    "verif/bytes_rlib/bytes_mut/shared_capsule/impl_SharedPayload/restore_header_region_read_loan.coma",
    "verif/bytes_rlib/bytes_mut/shared_capsule/lend_active_readonly_pool_interval.coma",
    "verif/bytes_rlib/bytes_mut/shared_capsule/retire_active_readonly_pool_interval.coma",
    "verif/bytes_rlib/bytes_mut/shared_capsule/impl_SharedPayload/prove_owner_region_excluded_from_pool.coma",
    "verif/bytes_rlib/bytes_mut/shared_capsule/impl_SharedPayload/prove_pool_complement_after_mutable_release.coma",
    "verif/bytes_rlib/bytes_mut/shared_capsule/impl_SharedPayload/prove_pool_complement_after_readonly_release.coma",
    "verif/bytes_rlib/bytes_mut/shared_capsule/impl_SharedPayload/prove_full_pool_after_final_owner_release.coma",
    "verif/bytes_rlib/bytes_mut/shared_capsule/impl_SharedPayload/prove_counter_agreement_after_owner_release.coma",
    "verif/bytes_rlib/bytes_mut/shared_capsule/impl_SharedPayload/drop_packet_owner.coma",
    "verif/bytes_rlib/bytes_mut/shared_capsule/impl_SharedPayload/drop_mutable_packet_owner.coma",
    "verif/bytes_rlib/bytes_mut/shared_capsule/impl_SharedPayload/drop_readonly_packet_owner.coma",
    "verif/bytes_rlib/bytes_mut/shared_capsule/impl_SharedPayload/prove_live_owner_ticket.coma",
    "verif/bytes_rlib/bytes_mut/shared_capsule/impl_SharedPayload/prove_readonly_ticket_pool_known.coma",
    "verif/bytes_rlib/bytes_mut/shared_capsule/impl_SharedPayload/prove_live_owner_matches_counter.coma",
    "verif/bytes_rlib/bytes_mut/shared_capsule/impl_SharedPayload/split_mutable_owner_after_increment.coma",
    "verif/bytes_rlib/bytes_mut/shared_capsule/impl_SharedPayload/freeze_mutable_owner.coma",
    "verif/bytes_rlib/bytes_mut/shared_capsule/impl_SharedPayload/prove_shifted_region_slots_known.coma",
    "verif/bytes_rlib/bytes_mut/shared_capsule/impl_SharedPayload/prove_idle_cap_implies_root_some.coma",
    "verif/bytes_rlib/bytes_mut/shared_capsule/impl_SharedPayload/prove_current_header_lease.coma",
    "verif/bytes_rlib/bytes_mut/shared_capsule/impl_SharedPayload/prove_final_action_excludes_live_state.coma",
    "verif/bytes_rlib/bytes_mut/shared_capsule/impl_SharedPayload/prove_open_phase_matches_action.coma",
    "verif/bytes_rlib/bytes_mut/shared_capsule/impl_SharedPayload/take_header_read_loan.coma",
    "verif/bytes_rlib/bytes_mut/shared_capsule/impl_SharedPayload/restore_header_read_loan.coma",
    "verif/bytes_rlib/bytes_mut/shared_capsule/impl_SharedOwnerPacket/lease_key_ghost.coma",
    "verif/bytes_rlib/bytes_mut/shared_capsule/impl_SharedOwnerPacket/into_ghost.coma",
    "verif/bytes_rlib/bytes_mut/shared_capsule/impl_SharedOwnerPacket/prove_event_namespace.coma",
    "verif/bytes_rlib/bytes_mut/shared_capsule/impl_SharedReadonlyPacket/prove_event_namespace.coma",
    "verif/bytes_rlib/bytes_mut/shared_capsule/impl_SharedReadonlyPacket/take_header_region_read_loan.coma",
    "verif/bytes_rlib/bytes_mut/shared_capsule/impl_SharedReadonlyPacket/borrow_slice_from_header_read_loan.coma",
    "verif/bytes_rlib/bytes_mut/shared_capsule/impl_SharedReadonlyPacket/restore_header_region_read_loan.coma",
    "verif/bytes_rlib/bytes_mut/shared_capsule/impl_SharedOwnerPacket/prove_cursor_within_region.coma",
    "verif/bytes_rlib/bytes_mut/impl_BytesMut/prove_shared_layout_implies_view_layout.coma",
    "verif/bytes_rlib/bytes_mut/shared_capsule/loaned_native_counter.coma",
    "verif/bytes_rlib/bytes_mut/shared_capsule/impl_SharedOwnerPacket/allocation_slot.coma",
    "verif/bytes_rlib/bytes_mut/shared_capsule/impl_SharedOwnerPacket_0/advance_cursor.coma",
    "verif/bytes_rlib/bytes_mut/shared_capsule/impl_SharedOwnerPacket_0/freeze_into_readonly.coma",
    "verif/bytes_rlib/bytes_mut/shared_capsule/impl_SharedOwnerPacket_0/build_split_packets.coma",
    "verif/bytes_rlib/bytes_mut/shared_capsule/impl_SharedOwnerPacket_0/split_after_increment.coma",
    "verif/bytes_rlib/bytes_mut/shared_capsule/impl_SharedOwnerPacket_0/try_take_header_read_loan.coma",
    "verif/bytes_rlib/bytes_mut/shared_capsule/impl_SharedOwnerPacket_0/try_restore_header_read_loan.coma",
    "verif/bytes_rlib/bytes_mut/shared_capsule/impl_SharedOwnerPacket_0/restore_lease_after_owner_release.coma",
    "verif/bytes_rlib/bytes_mut/shared_capsule/impl_SharedOwnerPacket_0/finish_terminal_owner_release.coma",
    "verif/bytes_rlib/bytes_mut/shared_capsule/impl_SharedOwnerPacket/into_release_packet.coma",
    "verif/bytes_rlib/bytes_mut/shared_capsule/impl_SharedReadonlyPacket/into_release_packet.coma",
    "verif/bytes_rlib/bytes_mut/shared_capsule/impl_SharedReleasePacket/try_take_header_read_loan_for.coma",
    "verif/bytes_rlib/bytes_mut/shared_capsule/impl_SharedReleasePacket/try_take_header_read_loan.coma",
    "verif/bytes_rlib/bytes_mut/shared_capsule/impl_SharedReleasePacket_0/release_native_owner.coma",
    "verif/bytes_rlib/bytes_mut/release_shared_common.coma",
    "verif/bytes_rlib/bytes_mut/release_shared_after_nonunique.coma",
    "verif/bytes_rlib/bytes_mut/shared_readonly_slice_to_vec.coma",
    "verif/bytes_rlib/bytes_mut/shared_v_to_vec_nonunique.coma",
    "verif/bytes_rlib/bytes_mut/impl_BytesMut_1/dispose_shared.coma",
    "verif/bytes_rlib/bytes_mut/release_readonly_shared.coma",
    "verif/bytes_rlib/bytes_mut/impl_BytesMut_0/freeze_shared_view.coma",
    "verif/bytes_rlib/bytes_mut/impl_Shared_0/vec_mut.coma",
    "verif/bytes_rlib/bytes_mut/impl_Shared_0/take_vec_storage.coma",
    "verif/bytes_rlib/bytes_mut/impl_Shared_0/drop_empty_vec_field.coma",
    "verif/bytes_rlib/ref_count_ops/impl_RefCountCell/new.coma",
    "verif/bytes_rlib/ref_count_ops/impl_RefCountCell/new_with_owner.coma",
    "verif/bytes_rlib/ref_count_ops/impl_RefCountCell/native_counter.coma",
    "verif/bytes_rlib/ref_count_ops/impl_RefCountCell/load_relaxed.coma",
    "verif/bytes_rlib/ref_count_ops/impl_RefCountCell/load_acquire.coma",
    "verif/bytes_rlib/ref_count_ops/impl_RefCountCell/compare_exchange_relaxed.coma",
    "verif/bytes_rlib/ref_count_ops/impl_RefCountCell/compare_exchange_acqrel_relaxed.coma",
    "verif/bytes_rlib/ref_count_ops/impl_RefCountCell/fetch_sub_release.coma",
    "verif/bytes_rlib/ref_count_ops/impl_RefCountCell/take_bootstrap_owner.coma",
    "verif/bytes_rlib/ref_count_ops/proof/impl_EventBoundInvariant_T/new.coma",
    "verif/bytes_rlib/ref_count_ops/proof/impl_EventBoundInvariant_T/into_inner.coma",
    "verif/bytes_rlib/ref_count_ops/proof/impl_EventBoundInvariant_T/open_with_tokens.coma",
    "verif/bytes_rlib/ref_count_ops/proof/impl_Protocol_for_EventState_T/protocol.coma",
    "verif/bytes_rlib/ref_count_ops/proof/observe_ref_count.coma",
    "verif/bytes_rlib/ref_count_ops/proof/observe_event_counter.coma",
    "verif/bytes_rlib/ref_count_ops/proof/update_ref_count.coma",
    "verif/bytes_rlib/ref_count_ops/proof/impl_NativeCounter/try_increment_with.coma",
    "verif/bytes_rlib/ref_count_limit/next_ref_count.coma",
    "verif/bytes_rlib/ref_count_ops/try_increment.coma",
    "verif/bytes_rlib/ref_count_ops/increment.coma",
    "verif/bytes_rlib/ownership_proof/raw_vec/impl_BoundPtr/advance_within.coma",
    "verif/bytes_rlib/ownership_proof/raw_vec/impl_BoundPtr/raw_pointer.coma",
    "verif/bytes_rlib/ownership_proof/raw_vec/impl_BoundPtr/as_ptr.coma",
    "verif/bytes_rlib/ownership_proof/raw_vec/impl_BoundPtr/as_non_null.coma",
    "verif/bytes_rlib/ownership_proof/raw_vec/impl_PhysicalRegion/split_at.coma",
    "verif/bytes_rlib/ownership_proof/raw_vec/impl_PhysicalPool/kernel_unit_is_empty.coma",
    "verif/bytes_rlib/ownership_proof/raw_vec/impl_PhysicalPool/empty_from.coma",
    "verif/bytes_rlib/ownership_proof/raw_vec/impl_PhysicalPool/lend_interval.coma",
    "verif/bytes_rlib/ownership_proof/raw_vec/impl_PhysicalPool/retire.coma",
    "verif/bytes_rlib/ownership_proof/raw_vec/impl_PhysicalPool/finish.coma",
    "verif/bytes_rlib/ownership_proof/raw_vec/bind_consumed_vec.coma",
    "verif/bytes_rlib/ownership_proof/raw_vec/slots_from_vec.coma",
    "verif/bytes_rlib/ownership_proof/raw_vec/slots_from_vec_values.coma",
    "verif/bytes_rlib/ownership_proof/raw_vec/int_from_usize.coma",
    "verif/bytes_rlib/ownership_proof/raw_vec/impl_RawAllocation/bound_ptr_at_zero.coma",
    "verif/bytes_rlib/ownership_proof/owned_region/impl_OwnedRegion/split_at.coma",
    "verif/bytes_rlib/ownership_proof/owned_region/impl_OwnedRegion/into_parts.coma",
    "verif/bytes_rlib/ownership_proof/owned_region/map_compose_eq.coma",
    "verif/bytes_rlib/ownership_proof/owned_region/map_op_get.coma",
    "verif/bytes_rlib/ownership_proof/owned_region/impl_OwnedRegion/from_model_ledger.coma",
    "verif/bytes_rlib/ownership_proof/owned_region/impl_OwnedRegion/slots.coma",
    "verif/bytes_rlib/ownership_proof/owned_region/impl_OwnedRegion/slot.coma",
    "verif/bytes_rlib/ownership_proof/raw_vec/impl_PhysicalRegion/slot.coma",
    "verif/bytes_rlib/ownership_proof/raw_vec/impl_PhysicalRegion/prove_prefix_known.coma",
    "verif/bytes_rlib/ownership_proof/raw_vec/impl_PhysicalRegion/prove_subregion_prefix_known.coma",
    "verif/bytes_rlib/ownership_proof/raw_vec/impl_RawAllocation/namespace_ghost.coma",
    "verif/bytes_rlib/ownership_proof/raw_vec/impl_RawAllocation_0/into_ghost.coma",
    "verif/bytes_rlib/provenance_specs/wrapping_offset.coma",
    "verif/bytes_rlib/capacity_ops/pack_vec_metadata.coma",
    "verif/bytes_rlib/capacity_ops/set_vec_pos_in_data.coma",
    "verif/bytes_rlib/vec_storage/impl_VecPartsWitness/native_parts.coma",
    "verif/bytes_rlib/vec_storage/impl_VecPartsWitness/contents_snapshot.coma",
    "verif/bytes_rlib/vec_storage/impl_VecRawParts/native_ptr.coma",
    "verif/bytes_rlib/vec_storage/impl_VecRawParts/logical_len.coma",
    "verif/bytes_rlib/vec_storage/impl_VecRawParts/logical_capacity.coma",
    "verif/bytes_rlib/vec_storage/impl_VecRawParts/take.coma",
    "verif/bytes_rlib/vec_storage/impl_VecRawParts/into_proof_authority.coma",
    "verif/bytes_rlib/vec_storage/impl_VecStorage/hold_owned.coma",
    "verif/bytes_rlib/vec_storage/impl_Default_for_VecStorage/default.coma",
    "verif/bytes_rlib/vec_storage/impl_Drop_for_VecStorage/drop.coma",
    "verif/bytes_rlib/vec_storage/impl_VecStorage_0/drop_empty_owner.coma",
)
EVIDENCE_TARGETS = TARGETS
EVIDENCE = "verification/results/direct-bytesmut-len-api"
PINNED = {
    "creusot_sha": "318615be3b8bbc60d1f6d52469ba5c0bdebed4f1",
    "why3_sha": "54c92f96bb0711d6e991c18f10bfbc08d90d028b",
    "why3find_sha": "eab37557d3e24e1913a3c4f44bc5528ef497c6c9",
}
TOOL_BINARIES = {
    "creusot-rustc", "cargo-creusot", "why3", "why3find",
    "alt-ergo", "z3", "cvc4", "cvc5",
}
PROVERS = {"alt-ergo", "z3", "cvc4", "cvc5"}
SPAN = re.compile(r'"(?P<file>/[^"\n]+\.rs)"\s+(?P<start>\d+)\s+\d+\s+(?P<end>\d+)\s+\d+')
WARNING = re.compile(r"no contract|contractless|unsupported|unimplemented|no specification|unproved external", re.I)


class ReuseMiss(ValueError):
    pass


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def path(root: Path, name: str) -> Path:
    relative = PurePosixPath(name)
    require(not relative.is_absolute() and ".." not in relative.parts,
            f"evidence path escapes crate: {name}")
    return root.joinpath(*relative.parts)


def std_manifest(crate: Path) -> dict:
    try:
        value = json.loads((crate / "verification/std-support/manifest.json").read_bytes())
        require(isinstance(value, dict) and isinstance(value.get("patches"), dict),
                "Std support manifest has no patch inventory")
        return value
    except (OSError, json.JSONDecodeError):
        raise ReuseMiss("cannot read the Std support manifest") from None


def std_package_root(crate: Path, manifest: dict | None = None) -> Path:
    manifest = manifest or std_manifest(crate)
    tool_root = Path(os.environ.get("BYTES_TOOL_ROOT", "/workspace/bytes-proof-tools")).expanduser()
    package = manifest.get("package", {})
    relative = package.get("private_copy_relative_path")
    require(isinstance(relative, str), "Std manifest omits the private package path")
    return path(tool_root.resolve(), relative).resolve()


def std_tree_sha256(root: Path) -> str:
    require(root.is_dir() and not root.is_symlink(), f"missing canonical Std package: {root}")
    files = []
    for item in root.rglob("*"):
        require(not item.is_symlink(), f"canonical Std package has an unexpected symlink: {item}")
        if item.is_file():
            files.append(item)
    digest_value = hashlib.sha256()
    for item in sorted(files, key=lambda value: value.relative_to(root).as_posix()):
        digest_value.update(item.relative_to(root).as_posix().encode("utf-8"))
        digest_value.update(b"\0")
        digest_value.update(digest(item.read_bytes()).encode("ascii"))
        digest_value.update(b"\n")
    return digest_value.hexdigest()


def inputs(crate: Path, verify_installed_std: bool = False,
           verify_installed_tool: bool = False) -> dict[str, str]:
    manifest = std_manifest(crate)
    patch_names = manifest["patches"]
    require(all(isinstance(name, str) and PurePosixPath(name).name == name
                for name in patch_names), "Std manifest has an unsafe patch path")
    actual_patches = {item.name for item in (crate / "verification/std-support").glob("*.patch")}
    require(actual_patches == set(patch_names),
            "Std support patch files differ from the canonical manifest inventory")
    fixed = (
        "Cargo.lock", "Cargo.toml", "why3find.json",
        "scripts/check-current-proof.py", "scripts/prepare-proof-std.py",
        "scripts/verify-current.bash", "scripts/setup-bytes-toolchain.sh",
        "verification/std-support/README.md",
        "verification/std-support/manifest.json",
    )
    patch_paths = tuple(f"verification/std-support/{name}" for name in patch_names)
    names = sorted((*fixed, *patch_paths, *(p.relative_to(crate).as_posix()
                              for p in (crate / "src").rglob("*.rs"))))
    try:
        values = {name: digest(path(crate, name).read_bytes()) for name in names}
    except OSError as error:
        raise ReuseMiss(f"proof input is missing: {error.filename}") from None
    expected_tree = manifest.get("candidate_tree", {}).get("sha256")
    require(isinstance(expected_tree, str) and re.fullmatch(r"[0-9a-f]{64}", expected_tree),
            "Std support manifest omits the reviewed candidate tree hash")
    if verify_installed_std:
        external_tree = std_tree_sha256(std_package_root(crate, manifest))
        require(external_tree == expected_tree,
                "canonical private Std package differs from the reviewed overlay tree")
    else:
        external_tree = expected_tree
    values["external_std_package_tree_sha256"] = external_tree
    installed = installed_tool_identity(require_configured=verify_installed_tool)
    values["installed_toolchain_identity_sha256"] = digest(json.dumps(
        {"pinned_revisions": PINNED, **installed}, sort_keys=True).encode("utf-8"))
    return values


def proof_leaves(node: object) -> int:
    if isinstance(node, dict) and "prover" in node:
        require(node.get("prover") in PROVERS and isinstance(node.get("time"), (int, float))
                and node["time"] >= 0, "proof result contains a failed/malformed prover leaf")
        return 1
    if isinstance(node, dict) and isinstance(node.get("children"), list):
        children = node["children"]
        require(bool(children), "proof result contains an empty goal")
        return sum(proof_leaves(child) for child in children)
    raise ValueError("proof result contains an unproved or unknown goal")


def check_result(data: bytes) -> int:
    try:
        goals = json.loads(data)["proofs"]["Coma"]
    except (json.JSONDecodeError, KeyError, TypeError):
        raise ValueError("target has no valid COMA proof result") from None
    require(isinstance(goals, dict) and bool(goals), "target has no proved COMA goals")
    return sum(proof_leaves(goal) for goal in goals.values())


def verify_feature_tree(data: bytes) -> None:
    text = data.decode("utf-8", errors="replace")
    require("creusot-std" in text, "Cargo feature tree omits creusot-std")
    require('bytes feature "default" (command-line)' in text
            and 'bytes feature "std"' in text, "proof did not use default std features")
    require("sc-drf" not in text, "sc-drf must stay disabled")


def target_cone_warnings(crate: Path, log: bytes, comas: list[bytes],
                         extra_roots: tuple[Path, ...] = ()) -> list[str]:
    spans: dict[str, list[tuple[int, int]]] = {}
    roots = (crate.resolve(), *(root.resolve() for root in extra_roots))
    for data in comas:
        for match in SPAN.finditer(data.decode("utf-8", errors="replace")):
            source = Path(match["file"]).resolve()
            for root in roots:
                try:
                    filename = source.relative_to(root).as_posix()
                except ValueError:
                    continue
                spans.setdefault(filename, []).append((int(match["start"]), int(match["end"])))
    warnings = []
    for block in re.split(r"(?m)(?=^warning: )", log.decode("utf-8", errors="replace")):
        if not WARNING.search(block):
            continue
        location = re.search(r"(?:^|[/\s])(src/[\w./-]+\.rs):(\d+):", block)
        if not location:
            continue
        filename, line = location.group(1), int(location.group(2))
        if any(start <= line <= end for start, end in spans.get(filename, [])):
            warnings.append(f"{filename}:{line}: {block.splitlines()[0]}")
    return warnings


def check(crate: Path) -> dict:
    receipt_path = crate / EVIDENCE / "proof.json"
    try:
        receipt = json.loads(receipt_path.read_bytes())
    except (OSError, json.JSONDecodeError) as error:
        raise ValueError(f"cannot read direct proof receipt: {error}") from None
    require(isinstance(receipt, dict) and receipt.get("schema_version") == 1
            and receipt.get("status") == "proved", "direct proof receipt is missing/malformed")
    if receipt.get("targets") != list(EVIDENCE_TARGETS):
        raise ReuseMiss("saved target inventory differs from the direct runner")
    if receipt.get("verified_inputs_sha256") != inputs(crate):
        raise ReuseMiss("a direct source, contract, configuration, or Std input changed")
    require(receipt.get("configuration") == {
        "crate": "bytes 1.11.1", "target": "x86_64-unknown-linux-gnu",
        "features": ["default", "std"], "rustflags": "--cfg bytes_original_unique_gate",
        "sc_drf": "disabled",
    }, "saved proof used a different direct-source configuration")

    tree = receipt.get("feature_tree", {})
    log = receipt.get("translation_diagnostics", {})
    saved_log = None
    for record, field in ((tree, "feature tree"), (log, "translation log")):
        try:
            data = path(crate, record["file"]).read_bytes()
        except (OSError, KeyError, TypeError):
            raise ReuseMiss(f"saved {field} is missing") from None
        if digest(data) != record.get("sha256"):
            raise ReuseMiss(f"saved {field} changed")
        if field == "feature tree":
            verify_feature_tree(data)
        else:
            saved_log = data

    tool = receipt.get("toolchain", {})
    require(tool.get("rust_toolchain") == "nightly-2026-06-22"
            and tool.get("host") == "x86_64-unknown-linux-gnu"
            and tool.get("pinned_revisions") == PINNED,
            "receipt omits the pinned Creusot/Why3 identities")
    hashes = tool.get("binary_sha256", {})
    require(set(hashes) == TOOL_BINARIES and all(re.fullmatch(r"[0-9a-f]{64}", str(v))
                                         for v in hashes.values()),
            "receipt has incomplete tool binary identities")
    require("creusot_toolpatch" not in tool,
            "receipt selects an unsupported patched Creusot toolchain")
    selected = tool.get("selected_compiler", {})
    require(selected.get("name") == "creusot-rustc"
            and isinstance(selected.get("path"), str)
            and Path(selected["path"]).is_absolute()
            and selected.get("sha256") == hashes["creusot-rustc"],
            "receipt does not identify the selected pristine compiler path and hash")
    installed = installed_tool_identity(require_configured=False)
    installed_hashes = {name: record["sha256"] for name, record in installed["binaries"].items()}
    require(hashes == installed_hashes and selected == installed["selected_compiler"],
            "receipt tool identities differ from the installed pristine toolchain")

    std = receipt.get("std_overlay", {})
    manifest = path(crate, "verification/std-support/manifest.json").read_bytes()
    std_manifest = json.loads(manifest)
    require(std == {"manifest_sha256": digest(manifest),
                    "candidate_tree_sha256": std_manifest["candidate_tree"]["sha256"]},
            "receipt omits the current canonical Std overlay identity")

    evidence = receipt.get("target_evidence", {})
    require(set(evidence) == set(EVIDENCE_TARGETS), "receipt omits direct target evidence")
    leaves, coma_blobs = 0, []
    for target in EVIDENCE_TARGETS:
        record = evidence[target]
        require(isinstance(record, dict), f"malformed evidence for {target}")
        expected_coma, expected_proof = evidence_paths(target)
        require(record.get("coma") == expected_coma and record.get("proof") == expected_proof,
                f"unexpected evidence path for {target}")
        for field in ("coma", "proof"):
            relative, expected = record.get(field), record.get(f"{field}_sha256")
            require(isinstance(relative, str) and re.fullmatch(r"[0-9a-f]{64}", str(expected)),
                    f"malformed {field} identity for {target}")
            try:
                data = path(crate, relative).read_bytes()
            except OSError:
                raise ReuseMiss(f"saved target evidence is missing: {relative}") from None
            if digest(data) != expected:
                raise ReuseMiss(f"saved target evidence changed: {relative}")
            if field == "coma":
                require(bool(data.strip()), f"{target} has an empty COMA body")
                coma_blobs.append(data)
            else:
                leaves += check_result(data)
    warnings = target_cone_warnings(crate, saved_log, coma_blobs,
                                    (std_package_root(crate),))
    require(log.get("target_cone_unsupported_calls") == warnings,
            "saved target-cone diagnostic record differs from COMA spans and translation log")
    require(not warnings, "selected targets have unsupported/contractless call diagnostics")
    return {"status": "pass", "mode": "direct-source-reuse", "targets": len(EVIDENCE_TARGETS),
            "prover_leaves": leaves, "solver_execution": "not performed",
            "full_api_coverage": False}


def evidence_paths(target: str) -> tuple[str, str]:
    coma = f"{EVIDENCE}/coma/{target}"
    proof = f"{EVIDENCE}/results/{PurePosixPath(target).with_suffix('')}/proof.json"
    return coma, proof


def installed_tool_identity(require_configured: bool = True) -> dict:
    manifest = Path(os.environ.get("BYTES_TOOL_ROOT", "/workspace/bytes-proof-tools")) \
        / "installation-manifest.json"
    try:
        value = json.loads(manifest.read_bytes())
    except (OSError, json.JSONDecodeError) as error:
        raise ValueError(f"cannot read installed tool identities: {error}") from None
    revisions = {name: value.get(name) for name in PINNED}
    require(revisions == PINNED, "installed tools differ from the pinned setup")
    binaries = value.get("binaries", {})
    require(set(binaries) == TOOL_BINARIES and "creusot_toolpatch" not in value,
            "installed manifest is not the pristine pinned toolchain")
    selected_name = "creusot-rustc"
    try:
        selected_path = Path(binaries[selected_name]["path"]).resolve(strict=True)
        selected_hash = binaries[selected_name]["sha256"]
    except (KeyError, TypeError, OSError):
        raise ValueError("installed manifest omits the pristine Creusot compiler") from None
    tool_root = Path(os.environ.get("BYTES_TOOL_ROOT", "/workspace/bytes-proof-tools")).resolve()
    require(selected_path == (tool_root / "vanilla/creusot-rustc").resolve(strict=True),
            "installed manifest does not select the pinned vanilla compiler path")
    require(re.fullmatch(r"[0-9a-f]{64}", str(selected_hash))
            and digest(selected_path.read_bytes()) == selected_hash,
            "selected pristine Creusot compiler hash differs from its installation manifest")
    for name, record in binaries.items():
        try:
            binary_path = Path(record["path"]).resolve(strict=True)
            binary_hash = record["sha256"]
        except (KeyError, TypeError, OSError):
            raise ValueError(f"installed manifest omits a valid binary identity: {name}") from None
        require(re.fullmatch(r"[0-9a-f]{64}", str(binary_hash))
                and digest(binary_path.read_bytes()) == binary_hash,
                f"installed tool binary differs from its pinned identity: {name}")
    if require_configured:
        configured = os.environ.get("CREUSOT_RUSTC")
        require(configured is not None and Path(configured).resolve(strict=True) == selected_path,
                "CREUSOT_RUSTC is not the pristine compiler selected by the installation manifest")
    return {"selected_compiler": {"name": selected_name, "path": str(selected_path),
                                  "sha256": selected_hash},
            "binaries": binaries}


def tool_identity(crate: Path) -> dict:
    installed = installed_tool_identity()
    binaries = installed["binaries"]
    names = sorted(TOOL_BINARIES)
    hashes = {name: binaries.get(name, {}).get("sha256") for name in names}
    for name, expected in hashes.items():
        try:
            binary = Path(binaries[name]["path"])
        except (KeyError, TypeError):
            raise ValueError(f"installed tool manifest omits binary: {name}") from None
        require(re.fullmatch(r"[0-9a-f]{64}", str(expected))
                and digest(binary.read_bytes()) == expected,
                f"installed tool binary differs from its pinned identity: {name}")
    rust_toolchain = os.environ.get("RUSTUP_TOOLCHAIN")
    require(rust_toolchain == "nightly-2026-06-22", "unexpected Rust toolchain")
    version = subprocess.run(["rustc", "-vV"], check=True, capture_output=True, text=True).stdout
    host = re.search(r"(?m)^host: (\S+)$", version)
    require(host is not None and host.group(1) == "x86_64-unknown-linux-gnu",
            "proof must target x86_64-unknown-linux-gnu")
    require(os.environ.get("RUSTFLAGS") == "--cfg bytes_original_unique_gate"
            and not os.environ.get("CARGO_ENCODED_RUSTFLAGS")
            and not os.environ.get("CARGO_BUILD_TARGET")
            and not any(name.startswith("CARGO_TARGET_") and name.endswith("_RUSTFLAGS")
                        for name in os.environ),
            "unexpected proof cfg or additional Rust flags")
    return {"rust_toolchain": rust_toolchain, "host": host.group(1),
            "pinned_revisions": PINNED, "binary_sha256": hashes,
            "selected_compiler": installed["selected_compiler"]}


def save(crate: Path, feature_file: Path, log_file: Path, snapshot_file: Path) -> dict:
    try:
        receipt_inputs = json.loads(snapshot_file.read_bytes())
    except (OSError, json.JSONDecodeError) as error:
        raise ValueError(f"cannot read pre-translation source snapshot: {error}") from None
    require(isinstance(receipt_inputs, dict) and receipt_inputs == inputs(crate, True, True),
            "proof inputs changed during translation or proving; refusing stale evidence")
    feature_data, log_data = feature_file.read_bytes(), log_file.read_bytes()
    verify_feature_tree(feature_data)
    require(bool(log_data), "direct translation log is empty")
    evidence, sources, leaves, coma_blobs = {}, {}, 0, []
    for target in EVIDENCE_TARGETS:
        coma = path(crate, target)
        sources[target] = coma
        result = coma.with_suffix("") / "proof.json"
        try:
            coma_data, proof_data = coma.read_bytes(), result.read_bytes()
        except OSError:
            raise ValueError(f"missing direct output for {target}") from None
        require(bool(coma_data.strip()), f"{target} has an empty COMA body")
        coma_blobs.append(coma_data)
        leaves += check_result(proof_data)
        coma_dst, proof_dst = evidence_paths(target)
        evidence[target] = {"coma": coma_dst, "coma_sha256": digest(coma_data),
                            "proof": proof_dst, "proof_sha256": digest(proof_data)}

    warnings = target_cone_warnings(crate, log_data, coma_blobs,
                                    (std_package_root(crate),))
    require(not warnings, "selected target call cone has unsupported diagnostics: "
            + "; ".join(warnings))
    tool = tool_identity(crate)
    shutil.rmtree(path(crate, f"{EVIDENCE}/coma"), ignore_errors=True)
    shutil.rmtree(path(crate, f"{EVIDENCE}/results"), ignore_errors=True)

    for target, record in evidence.items():
        source = sources[target]
        for src, dst in ((source, record["coma"]),
                         (source.with_suffix("") / "proof.json", record["proof"])):
            output = path(crate, dst)
            output.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(src, output)
    feature_dst, log_dst = f"{EVIDENCE}/feature-tree.txt", f"{EVIDENCE}/translation.log"
    path(crate, feature_dst).write_bytes(feature_data)
    path(crate, log_dst).write_bytes(log_data)
    receipt = {
        "schema_version": 1, "status": "proved", "verified_inputs_sha256": receipt_inputs,
        "configuration": {"crate": "bytes 1.11.1", "target": "x86_64-unknown-linux-gnu",
                          "features": ["default", "std"], "rustflags": "--cfg bytes_original_unique_gate",
                          "sc_drf": "disabled"},
        "toolchain": tool,
        "std_overlay": {
            "manifest_sha256": digest(path(crate, "verification/std-support/manifest.json").read_bytes()),
            "candidate_tree_sha256": json.loads(path(crate, "verification/std-support/manifest.json").read_bytes())["candidate_tree"]["sha256"],
        },
        "feature_tree": {"file": feature_dst, "sha256": digest(feature_data)},
        "translation_diagnostics": {"file": log_dst, "sha256": digest(log_data),
                                    "target_cone_unsupported_calls": warnings},
        "targets": list(EVIDENCE_TARGETS), "target_evidence": evidence,
    }
    output = crate / EVIDENCE / "proof.json"
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(receipt, indent=2) + "\n")
    return {"status": "saved", "targets": len(EVIDENCE_TARGETS), "prover_leaves": leaves,
            "evidence": output.relative_to(crate).as_posix()}


def main() -> int:
    if sys.argv[1:] == ["--targets"]:
        print("\n".join(TARGETS))
        return 0
    if sys.argv[1:] == ["--std-root"]:
        print(std_package_root(Path(__file__).resolve().parents[1]))
        return 0
    crate = Path(__file__).resolve().parents[1]
    try:
        if sys.argv[1:] == ["--snapshot"]:
            print(json.dumps(inputs(crate, True, True), sort_keys=True))
            return 0
        if len(sys.argv) == 5 and sys.argv[1] == "--save":
            result = save(crate, Path(sys.argv[2]), Path(sys.argv[3]), Path(sys.argv[4]))
        elif sys.argv[1:] in ([], ["--check"]):
            result = check(crate)
        else:
            print("usage: check-current-proof.py [--targets | --std-root | --snapshot | --check | --save FEATURE_TREE TRANSLATION_LOG INPUT_SNAPSHOT]",
                  file=sys.stderr)
            return 2
        print(json.dumps(result, indent=2))
        return 0
    except ReuseMiss as error:
        print("proof reuse miss: " + str(error), file=sys.stderr)
    except (OSError, TypeError, ValueError) as error:
        print("proof check failed: " + str(error), file=sys.stderr)
    return 1


if __name__ == "__main__":
    sys.exit(main())
