# CohBit Legal GMI Threat Model: Neutralizing the Citation Goblin

The "Citation Goblin" represents various failure modes in legal AI systems: hallucinations, out-of-date law, and intentional manipulation.

## 1. Hallucinated Quotes (The Fake Span)
- **Threat**: LLM invents a quote and attributes it to a real case.
- **Defense**: **M5 (Quote-Span Receipts)**. The verifier uses a `LegalSourceAdapter` to locate the quote by exact byte-span in the original PDF or text. If the span doesn't match, the claim is rejected.

## 2. Zombie Precedent (The Overruled Case)
- **Threat**: LLM cites a case that was overruled or reversed.
- **Defense**: **M6 (Real Legal Status)**. The verifier checks the current procedural status of every authority. "Good Law" status is bound into the receipt.

## 3. History Rewrite (The Log Edit)
- **Threat**: An attacker modifies an old receipt in the log to change the trajectory.
- **Defense**: **M10 (Persistent Receipt Chain)**. The incremental root commitment ($root_n = H(root_{n-1}, \rho_n)$) ensures that changing any past record invalidates the current root.

## 4. Evidence Swap (The Source Tamper)
- **Threat**: An attacker modifies the source text *after* verification to make a fake claim look real.
- **Defense**: **M12 (Evidence Bundle)**. The bundle hash commits to the actual evidence substrate. If the source text is modified, the bundle verification fails.

## 5. Free Certainty (The Stability Violation)
- **Threat**: A model makes high-certainty claims without providing enough evidentiary "spend."
- **Defense**: **Stability Law**. The kernel enforces $\Delta V \ge spend - defect$. A transition is rejected if the energy balance is negative.
