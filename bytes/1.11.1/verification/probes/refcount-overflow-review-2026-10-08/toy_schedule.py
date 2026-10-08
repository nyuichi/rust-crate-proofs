#!/usr/bin/env python3
"""Finite arithmetic scheduler model. Does not spawn native threads or free memory."""
import json

def original_schedule(bits):
    modulus = 1 << bits
    limit = modulus // 2 - 1
    count = 1
    handles = 1
    trace = []
    # Every completed clone obeys the actual normal-return guard.
    while count <= limit:
        old = count
        count = (count + 1) % modulus
        assert old <= limit
        handles += 1
    trace.append({'phase': 'normal clones completed', 'count': count, 'handles': handles})
    pending = []
    for _ in range(modulus // 2):
        old = count
        count = (count + 1) % modulus
        assert old > limit
        pending.append(old)  # Suspend before evaluating its post-RMW abort.
    assert count == 0
    trace.append({'phase': 'overflowing calls suspended before abort',
                  'count': count, 'pending_old_values': pending})
    old = count
    count = (count + 1) % modulus
    assert old <= limit
    handles += 1
    trace.append({'phase': 'another clone returns normally', 'old': old, 'count': count,
                  'handles': handles})
    old = count
    count = (count - 1) % modulus
    handles -= 1
    assert old == 1 and handles > 0
    trace.append({'phase': 'release would select last-owner branch', 'old': old,
                  'count': count, 'other_completed_handles': handles,
                  'paused_calls': len(pending)})
    return {'bits': bits, 'modulus': modulus, 'threshold': limit, 'trace': trace}

def guarded_event_check(bits):
    modulus = 1 << bits
    limit = modulus // 2 - 1
    upper = limit + 1
    for old in range(upper + 1):
        # Successful CAS checks expected == actual and old <= limit atomically.
        if old <= limit:
            new = old + 1
            assert new <= upper and new < modulus
        # Every failure/declined update performs no store by that operation.
        assert 0 <= old <= upper
        # A legitimate positive-count decrement also preserves the interval.
        if old > 0:
            assert 0 <= old - 1 <= upper
    return {'bits': bits, 'range': [0, upper], 'all_guarded_event_cases_checked': True}

if __name__ == '__main__':
    print(json.dumps({'scope': 'abstract finite-word interleaving; no physical OS-thread feasibility claim',
                      'counterexample': original_schedule(3),
                      'inductive_guard_checks': [guarded_event_check(w) for w in range(2, 13)]}, indent=2))
