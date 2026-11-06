# Decision semantics

- `Accept`: all configured admission checks pass and the stable, bounded transition mutates committed content.
- `Reject`: supplied material contradicts a required check or is malformed. Committed content and valuation remain unchanged.
- `Escalate`: evidence, authority status, target context, or human judgment is unresolved. Committed content and valuation remain unchanged.

All three outcomes advance audit metadata and receive a canonical receipt. Content state and audit history are distinct: a non-accepting attempt may advance `step_index` and the receipt head without changing the content-state hash.

`CommitOutcome` is derived from the receipt decision, never from whether evaluation returned `Ok`.
