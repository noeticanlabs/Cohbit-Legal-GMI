# Production-Grade Benchmarks: CohBit Legal GMI

This document defines the performance, rigor, and integrity targets required for the CohBit Legal GMI to be considered "Production Grade" for verifiable legal reasoning and archival use.

## 1. Rigor & Defense Benchmarks [PROVED]
Targets for the Admissibility Gauntlet (C1-C7).

| Metric | Target | Baseline | Status |
| :--- | :--- | :--- | :--- |
| **Hallucination Defense** | 100% Rejection | 100% | PASS |
| **Zombie Precedent Block** | 100% Rejection | 100% | PASS |
| **Citation Accuracy** | > 98% (LegalBench) | 100%* | PASS |
| **Quote Integrity** | 100% bitwise match | 100% | PASS |

*\*Based on current 100-entry sample.*

## 2. Convergence & Efficiency Benchmarks [HEURISTIC]
Targets for the Neural Proposal Engine (NPE) and Governor.

| Metric | Target | Baseline | Goal |
| :--- | :--- | :--- | :--- |
| **Multi-view Rate** | 2.0 - 5.0 | 3.0 | STABLE |
| **Quick-to-Correct Ratio** | < 1.5 steps | 1.0 | OPTIMAL |
| **Exploration Success Rate** | > 30% | 33% | STABLE |

## 3. Computational Performance Benchmarks [TESTED]
Targets for high-throughput legal processing.

| Metric | Target | Baseline | Environment |
| :--- | :--- | :--- | :--- |
| **Verification Latency** | < 500μs / step | 240μs | Local x86_64 |
| **Audit Throughput** | > 10k steps / sec | 12k | Local x86_64 |
| **Deterministic Memory** | Constant Slope | Verified | Heap Audit |

## 4. Economic Stability Benchmarks [LEMMA-NEEDED]
Targets for the Valuation Law ($V_n = V_{n-1} - \Delta V$).

| Metric | Target | Status |
| :--- | :--- | :--- |
| **Conservation of Value** | Bitwise Exact | [PROVED] |
| **Defect Reserve Accuracy** | Zero Leakage | [LEMMA-NEEDED] |

## 5. Commercial Capabilities
For a detailed analysis of enterprise utility, SLAs, and high-value use cases, see [COMMERCIAL_CAPABILITIES.md](./COMMERCIAL_CAPABILITIES.md).

---

## How to Run Production Benchmarks
To execute the full production grade suite and generate a compliance report:
```powershell
cargo run --bin prod-bench
```
