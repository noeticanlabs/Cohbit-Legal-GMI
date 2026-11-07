import hashlib
import json
import tempfile
import unittest
from pathlib import Path

from tools.ingest_documents import IngestionError, parse_document, read_from_root, verify_result


class IngestionTests(unittest.TestCase):
    def setUp(self):
        self.workspace = tempfile.TemporaryDirectory()
        self.root = Path(self.workspace.name)

    def tearDown(self):
        self.workspace.cleanup()

    def test_utf8_offsets_not_codepoint_offsets(self):
        data = "§ 2 — Tenant shall pay café rent.\n".encode("utf-8")
        result = parse_document(data)
        norm = next(c for c in result["candidates"] if c["kind"] == "normative_marker")
        self.assertEqual(data[norm["trigger_start_byte"]:norm["trigger_end_byte"]], b"shall")
        self.assertEqual(result["original_sha256"], hashlib.sha256(data).hexdigest())
        self.assertEqual(result["assurance"]["legal_interpretation"], "not_performed")

    def test_crlf_preserved_without_normalizing(self):
        data = b"Section 4.2: Tenant must repair.\r\nExcept in emergencies.\r\n"
        result = parse_document(data)
        self.assertIn("reference_marker", {x["kind"] for x in result["candidates"]})
        self.assertIn("exception_marker", {x["kind"] for x in result["candidates"]})
        self.assertEqual(result["units"][1]["start_byte"], len(b"Section 4.2: Tenant must repair.\r\n"))
        verify_result(data, result)

    def test_definition_crossreference_temporal(self):
        data = b'"Effective Date" means May 1. See Section 3.1. Notice within 14 calendar days.'
        kinds = {x["kind"] for x in parse_document(data)["candidates"]}
        self.assertTrue({"definition_marker", "reference_marker", "temporal_marker"}.issubset(kinds))

    def test_no_normative_term_from_maybe(self):
        result = parse_document(b"Maybe the agency will respond.")
        self.assertFalse(any(c["kind"] == "normative_marker" for c in result["candidates"]))

    def test_unknown_but_valid_prose_has_no_claims(self):
        result = parse_document(b"The sky is blue in this fictional sample.")
        self.assertEqual(result["candidates"], [])
        self.assertFalse(result["assurance"]["full_integrated_runtime"])

    def test_rejects_invalid_utf8(self):
        with self.assertRaises(IngestionError):
            parse_document(b"text \xff")

    def test_rejects_nul(self):
        with self.assertRaises(IngestionError):
            parse_document(b"must\x00not")

    def test_rejects_bom(self):
        with self.assertRaises(IngestionError):
            parse_document(b"\xef\xbb\xbfMust deliver")

    def test_rejects_empty(self):
        with self.assertRaises(IngestionError):
            parse_document(b"\n \t\n")

    def test_rejects_oversized(self):
        with self.assertRaises(IngestionError):
            parse_document(b"x" * 2_000_001)

    def test_rejects_modified_quote(self):
        data = b"Person must comply."
        result = parse_document(data)
        result["candidates"][0]["quote"] = "Person may comply."
        with self.assertRaises(IngestionError):
            verify_result(data, result)

    def test_rejects_modified_trigger(self):
        data = b"Person must comply."
        result = parse_document(data)
        result["candidates"][0]["trigger"] = "shall"
        with self.assertRaises(IngestionError):
            verify_result(data, result)

    def test_rejects_forged_assurance(self):
        data = b"Person must comply."
        result = parse_document(data)
        result["assurance"]["formal_proof"] = "ProvedWithinScope"
        with self.assertRaises(IngestionError):
            verify_result(data, result)

    def test_rejects_forged_candidate_status(self):
        data = b"Person must comply."
        result = parse_document(data)
        result["candidates"][0]["status"] = "ProvedWithinScope"
        with self.assertRaises(IngestionError):
            verify_result(data, result)

    def test_rejects_wrong_parent_evidence(self):
        data = b"Person must comply.\nUnless excused."
        result = parse_document(data)
        result["candidates"][0]["evidence_unit_id"] = "line-2"
        with self.assertRaises(IngestionError):
            verify_result(data, result)

    def test_rejects_modified_document_bytes(self):
        data = b"Person must comply."
        result = parse_document(data)
        with self.assertRaises(IngestionError):
            verify_result(b"Person may comply.", result)

    def test_rejects_parent_traversal(self):
        with self.assertRaises(IngestionError):
            read_from_root(self.root, "../secret.txt")

    def test_rejects_symlink(self):
        (self.root / "note.txt").write_text("Must pay")
        (self.root / "shortcut.txt").symlink_to(self.root / "note.txt")
        with self.assertRaises(IngestionError):
            read_from_root(self.root, "shortcut.txt")

    def test_rejects_pdf_in_component_stage(self):
        (self.root / "notice.pdf").write_bytes(b"%PDF-1.4")
        with self.assertRaises(IngestionError):
            read_from_root(self.root, "notice.pdf")

    def test_result_deterministic(self):
        data = b"Tenant shall pay. Unless notice is given, rent must be paid."
        self.assertEqual(json.dumps(parse_document(data), sort_keys=True), json.dumps(parse_document(data), sort_keys=True))

    def test_never_labels_generated_interpretation_a_proof(self):
        data = b"The law shall apply to all persons."
        result = parse_document(data)
        self.assertEqual(result["assurance"]["formal_proof"], "not_generated")
        self.assertEqual(result["assurance"]["external_authority"], "not_checked")
        for item in result["candidates"]:
            self.assertEqual(item["status"], "uninterpreted_lexical_candidate")


if __name__ == "__main__":
    unittest.main()
