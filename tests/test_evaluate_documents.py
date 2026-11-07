import hashlib
import tempfile
import unittest
from pathlib import Path

from tools.evaluate_documents import EvaluationError, evaluate


class EvaluationTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self.text = "The tenant must give 30 days' notice.\n"
        (self.root / "sample.txt").write_text(self.text, encoding="utf-8")
        self.digest = hashlib.sha256(self.text.encode()).hexdigest()
        self.end = len("The tenant must give 30 days' notice.".encode())
        self.span = {"start_byte": 0, "end_byte": self.end, "quote": "The tenant must give 30 days' notice."}
        self.gold = {"a": {"case_id": "a", "document_path": "sample.txt", "document_sha256": self.digest, "document_family": "contract", "gold_label": "entailed", "evidence_spans": [self.span]}}
        self.pred = {"a": {"case_id": "a", "document_sha256": self.digest, "decision": "entailed", "evidence_spans": [self.span]}}

    def tearDown(self):
        self.temp.cleanup()

    def test_correct_matching_evidence(self):
        report = evaluate(self.gold, self.pred, self.root, "full_integrated")
        self.assertTrue(report["pass_zero_false_conclusions_and_exact_evidence"])
        self.assertEqual(report["exact_gold_evidence_spans_matched"], 1)
        self.assertFalse(report["run_mode_independently_verified"])

    def test_wrong_conclusion_cannot_pass(self):
        self.pred["a"]["decision"] = "contradicted"
        report = evaluate(self.gold, self.pred, self.root, "full_integrated")
        self.assertFalse(report["pass_zero_false_conclusions_and_exact_evidence"])
        self.assertEqual(report["incorrect_decisive"], 1)

    def test_escalation_is_safe_but_not_accuracy(self):
        self.pred["a"]["decision"] = "escalate"
        self.pred["a"]["evidence_spans"] = []
        report = evaluate(self.gold, self.pred, self.root, "full_integrated")
        self.assertTrue(report["pass_zero_false_conclusions_and_exact_evidence"])
        self.assertEqual(report["coverage"], 0)

    def test_all_escalate_fails_coverage_gate(self):
        self.pred["a"]["decision"] = "escalate"
        self.pred["a"]["evidence_spans"] = []
        report = evaluate(self.gold, self.pred, self.root, "full_integrated", min_coverage=0.5)
        self.assertFalse(report["pass_configured_technical_gate"])

    def test_component_test_cannot_pass_full_system_gate(self):
        report = evaluate(self.gold, self.pred, self.root, "component_control")
        self.assertFalse(report["pass_configured_technical_gate"])

    def test_missing_evidence_fails(self):
        self.pred["a"]["evidence_spans"] = []
        report = evaluate(self.gold, self.pred, self.root, "full_integrated")
        self.assertFalse(report["pass_zero_false_conclusions_and_exact_evidence"])

    def test_forged_quote_fails(self):
        self.pred["a"]["evidence_spans"] = [{**self.span, "quote": "fabricated evidence"}]
        with self.assertRaises(EvaluationError):
            evaluate(self.gold, self.pred, self.root, "full_integrated")

    def test_doc_digest_mismatch_fails(self):
        self.pred["a"]["document_sha256"] = "a" * 64
        with self.assertRaises(EvaluationError):
            evaluate(self.gold, self.pred, self.root, "full_integrated")

    def test_missing_case_fails(self):
        with self.assertRaises(EvaluationError):
            evaluate(self.gold, {}, self.root, "full_integrated")

    def test_path_traversal_fails(self):
        self.gold["a"]["document_path"] = "../../etc/passwd"
        with self.assertRaises(EvaluationError):
            evaluate(self.gold, self.pred, self.root, "full_integrated")

    def test_gold_requires_evidence_for_decisive_answer(self):
        self.gold["a"]["evidence_spans"] = []
        with self.assertRaises(EvaluationError):
            evaluate(self.gold, self.pred, self.root, "full_integrated")


if __name__ == "__main__":
    unittest.main()
