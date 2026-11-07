#!/usr/bin/env python3
"""Independent, fail-closed evaluation of proposed document interpretations.

This checks held-out labels and evidence offsets; it does NOT prove legal truth,
authority authenticity, independence of the gold set, or execution of CohBit.
No dependencies beyond Python 3.10+.
"""
import argparse
import hashlib
import json
from collections import Counter, defaultdict
from pathlib import Path

GOLD = {"entailed", "contradicted", "not_mentioned", "indeterminate"}
PRED = {"entailed", "contradicted", "not_mentioned", "escalate"}


class EvaluationError(ValueError):
    pass


def read_jsonl(path):
    result = {}
    for line_no, line in enumerate(Path(path).read_text(encoding="utf-8").splitlines(), 1):
        if not line.strip():
            continue
        try:
            record = json.loads(line)
        except (json.JSONDecodeError, UnicodeError) as exc:
            raise EvaluationError(f"{path}:{line_no}: invalid JSON: {exc}") from exc
        if not isinstance(record, dict) or not isinstance(record.get("case_id"), str) or not record["case_id"]:
            raise EvaluationError(f"{path}:{line_no}: missing case_id")
        if record["case_id"] in result:
            raise EvaluationError(f"{path}:{line_no}: duplicate case_id {record['case_id']!r}")
        result[record["case_id"]] = record
    if not result:
        raise EvaluationError(f"{path}: empty dataset")
    return result


def document_bytes(root, relative):
    if not isinstance(relative, str) or not relative:
        raise EvaluationError("missing document_path")
    safe_root = Path(root).resolve()
    path = (safe_root / relative).resolve()
    if not path.is_relative_to(safe_root) or not path.is_file():
        raise EvaluationError(f"unavailable or unsafe document path: {relative}")
    return path.read_bytes()


def verify_spans(spans, data, context):
    if not isinstance(spans, list):
        raise EvaluationError(f"{context}: evidence_spans must be a list")
    positions = []
    for item in spans:
        if not isinstance(item, dict):
            raise EvaluationError(f"{context}: invalid span")
        start, end, quote = item.get("start_byte"), item.get("end_byte"), item.get("quote")
        if type(start) is not int or type(end) is not int or start < 0 or end <= start or end > len(data):
            raise EvaluationError(f"{context}: invalid byte range")
        if not isinstance(quote, str) or data[start:end] != quote.encode("utf-8"):
            raise EvaluationError(f"{context}: quote does not match exact source bytes")
        positions.append((start, end))
    return set(positions)


def evaluate(gold, predictions, docs_root, mode, min_cases=1, min_families=1, min_coverage=0.0):
    if not 0 <= min_coverage <= 1:
        raise EvaluationError("min_coverage must be within [0,1]")
    if mode not in {"full_integrated", "component_control"}:
        raise EvaluationError("mode must be full_integrated or component_control")
    if set(gold) != set(predictions):
        raise EvaluationError(f"case IDs differ: missing={sorted(set(gold)-set(predictions))}, extra={sorted(set(predictions)-set(gold))}")
    if len(gold) < min_cases:
        raise EvaluationError(f"too few cases ({len(gold)} < {min_cases})")
    families = Counter()
    outcomes = Counter()
    family_results = defaultdict(Counter)
    missing_gold_spans = 0
    gold_span_count = 0
    failures = []
    for cid in sorted(gold):
        g, p = gold[cid], predictions[cid]
        label, decision = g.get("gold_label"), p.get("decision")
        if label not in GOLD or decision not in PRED:
            raise EvaluationError(f"{cid}: unknown gold_label or decision")
        family = g.get("document_family")
        if not isinstance(family, str) or not family:
            raise EvaluationError(f"{cid}: missing document_family")
        families[family] += 1
        data = document_bytes(docs_root, g.get("document_path"))
        expected_hash = hashlib.sha256(data).hexdigest()
        if g.get("document_sha256") != expected_hash or p.get("document_sha256") != expected_hash:
            raise EvaluationError(f"{cid}: gold/prediction document digest mismatch")
        gs = verify_spans(g.get("evidence_spans", []), data, f"{cid}: gold")
        ps = verify_spans(p.get("evidence_spans", []), data, f"{cid}: prediction")
        if label in {"entailed", "contradicted"} and not gs:
            raise EvaluationError(f"{cid}: decisive gold conclusion has no independently labeled evidence")
        if decision in {"entailed", "contradicted"} and not ps:
            failures.append({"case_id": cid, "failure": "decisive prediction missing evidence"})
        if decision == "escalate":
            outcomes["escalated"] += 1
            family_results[family]["escalated"] += 1
            continue
        if decision == "not_mentioned":
            # A quote-based scorer does not prove a complete search or
            # rule out exceptions; closed-world absence needs another gate.
            outcomes["unsupported_absence"] += 1
            failures.append({"case_id": cid, "failure": "unsupported closed-world not_mentioned conclusion"})
        outcomes["answered"] += 1
        family_results[family]["answered"] += 1
        if label != decision:
            outcomes["incorrect_decisive"] += 1
            family_results[family]["incorrect_decisive"] += 1
            failures.append({"case_id": cid, "failure": "incorrect non-escalated conclusion", "gold": label, "predicted": decision})
        else:
            outcomes["correct_decisive"] += 1
            family_results[family]["correct_decisive"] += 1
        if label == decision and decision in {"entailed", "contradicted"}:
            gold_span_count += len(gs)
            missing_gold_spans += len(gs - ps)
            if gs - ps:
                failures.append({"case_id": cid, "failure": "missing exact independently labeled evidence spans", "missing_count": len(gs - ps)})
    if len(families) < min_families:
        raise EvaluationError(f"too few document families ({len(families)} < {min_families})")
    answered = outcomes["answered"]
    wrong = outcomes["incorrect_decisive"]
    zero_error_upper = (1 - 0.05 ** (1 / answered)) if answered and wrong == 0 else None
    return {
        "run_mode_claimed_by_operator": mode,
        "run_mode_independently_verified": False,
        "claims_of_real_world_legal_correctness": False,
        "cases": len(gold),
        "document_families": dict(sorted(families.items())),
        "answered": answered,
        "escalated": outcomes["escalated"],
        "incorrect_decisive": wrong,
        "correct_decisive": outcomes["correct_decisive"],
        "coverage": round(answered / len(gold), 6),
        "exact_gold_evidence_spans_matched": gold_span_count - missing_gold_spans,
        "exact_gold_evidence_spans_required": gold_span_count,
        "zero_error_one_sided_95pct_upper_bound": zero_error_upper,
        "by_family": {k: dict(v) for k, v in sorted(family_results.items())},
        "failures": failures,
        "pass_zero_false_conclusions_and_exact_evidence": not failures,
        "pass_configured_technical_gate": (not failures and answered > 0 and answered / len(gold) >= min_coverage and mode == "full_integrated"),
        "configured_minimum_coverage": min_coverage,
        "limitations": "Gold labels, source authenticity, execution mode and independence must be verified externally. No verified legal-accuracy or proof-soundness claim follows from this score.",
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--gold", required=True)
    parser.add_argument("--predictions", required=True)
    parser.add_argument("--docs-root", required=True)
    parser.add_argument("--mode", choices=["full_integrated", "component_control"], required=True)
    parser.add_argument("--min-cases", type=int, default=1)
    parser.add_argument("--min-families", type=int, default=1)
    parser.add_argument("--min-coverage", type=float, default=0.0)
    parser.add_argument("--report", help="Optional JSON output path")
    args = parser.parse_args()
    try:
        result = evaluate(read_jsonl(args.gold), read_jsonl(args.predictions), args.docs_root, args.mode, args.min_cases, args.min_families, args.min_coverage)
    except (EvaluationError, OSError) as exc:
        parser.exit(2, f"Evaluation failed closed: {exc}\n")
    rendered = json.dumps(result, indent=2, sort_keys=True)
    if args.report:
        Path(args.report).write_text(rendered + "\n", encoding="utf-8")
    print(rendered)
    if not result["pass_configured_technical_gate"]:
        parser.exit(1)


if __name__ == "__main__":
    main()
