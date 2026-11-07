#!/usr/bin/env python3
"""LEGAL-DOC-002-B/C: byte-preserving UTF-8 ingestion and lexical candidate discovery.

UNTRUSTED COMPONENT DIAGNOSTIC, not a semantic/legal interpreter, source
authority checker, formal proof generator, or integrated Rust/Lean workflow.
"""
import argparse
import hashlib
import json
import re
from pathlib import Path

MAX_BYTES = 2_000_000
MAX_LINES = 20_000
ALLOWED_EXTENSIONS = {".txt", ".md"}
SCHEMA_VERSION = "LEGAL-DOC-002/lexical-candidates-v1"

PATTERNS = {
    "normative_marker": re.compile(r"(?<!\w)(?:shall\s+not|must\s+not|may\s+not|shall|must|may|is\s+prohibited\s+from)(?!\w)", re.I),
    "exception_marker": re.compile(r"(?<!\w)(?:except\s+(?:as\s+)?(?:provided|where|when|that|for|if)|except\b|unless\b|notwithstanding\b|subject\s+to\b|provided\s+that\b)(?!\w)", re.I),
    "definition_marker": re.compile(r"(?<!\w)(?:means|shall\s+mean|is\s+defined\s+as)(?!\w)", re.I),
    "reference_marker": re.compile(r"(?<!\w)(?:(?:section|article|paragraph|subsection|rule)\s+\d+(?:\.\d+)*(?:\([a-z0-9]+\))*)(?!\w)", re.I),
    "temporal_marker": re.compile(r"(?<!\w)(?:within\s+\d+\s+(?:calendar\s+|business\s+)?days?\b|no\s+later\s+than\b|on\s+or\s+before\b|effective\s+(?:on|as\s+of)\b)(?!\w)", re.I),
}


class IngestionError(ValueError):
    """Indicates that evidence extraction must not proceed."""


def read_from_root(root: str | Path, relative: str) -> tuple[str, bytes]:
    if not isinstance(relative, str) or not relative or Path(relative).is_absolute():
        raise IngestionError("document path must be a nonempty relative path")
    if ".." in Path(relative).parts:
        raise IngestionError("parent traversal is prohibited")
    root_path = Path(root).resolve(strict=True)
    candidate = root_path / relative
    if any(p.is_symlink() for p in (candidate, *candidate.parents) if p != root_path and root_path in p.parents):
        raise IngestionError("symlinked document path is prohibited")
    path = candidate.resolve(strict=True)
    if not path.is_relative_to(root_path) or not path.is_file():
        raise IngestionError("document must be a regular file inside the authorized directory")
    if path.suffix.lower() not in ALLOWED_EXTENSIONS:
        raise IngestionError("unsupported format: only UTF-8 .txt/.md files are accepted")
    if path.stat().st_size > MAX_BYTES:
        raise IngestionError("document size limit exceeded")
    data = path.read_bytes()
    if len(data) > MAX_BYTES:
        raise IngestionError("document size limit exceeded")
    return relative, data


def parse_document(data: bytes, document_id: str = "document") -> dict:
    if not isinstance(data, bytes):
        raise IngestionError("original document must be bytes")
    if not 0 < len(data) <= MAX_BYTES:
        raise IngestionError("document is empty or too large")
    if data.startswith((b"\xff\xfe", b"\xfe\xff", b"\xef\xbb\xbf")):
        raise IngestionError("byte-order-mark format requires a separate reviewed extractor")
    try:
        text = data.decode("utf-8", errors="strict")
    except UnicodeDecodeError as exc:
        raise IngestionError("document is not valid UTF-8") from exc
    if not text.strip():
        raise IngestionError("document contains no non-whitespace content")
    if any((ord(ch) < 32 and ch not in "\n\r\t") or ord(ch) == 127 for ch in text):
        raise IngestionError("document contains binary/control characters")
    lines = text.splitlines(keepends=True)
    if len(lines) > MAX_LINES:
        raise IngestionError("document exceeds line limit")
    boundaries = [0]
    for char in text:
        boundaries.append(boundaries[-1] + len(char.encode("utf-8")))
    assert boundaries[-1] == len(data)
    units, candidates = [], []
    char_offset = 0
    for line_number, raw_line in enumerate(lines, 1):
        content = raw_line.rstrip("\r\n")
        if content.strip():
            start_char, end_char = char_offset, char_offset + len(content)
            start_byte, end_byte = boundaries[start_char], boundaries[end_char]
            if data[start_byte:end_byte] != content.encode("utf-8"):
                raise IngestionError("loss of source fidelity")
            unit_id = f"line-{line_number}"
            units.append({
                "unit_id": unit_id, "line_number": line_number,
                "start_byte": start_byte, "end_byte": end_byte, "quote": content,
            })
            for kind, pattern in PATTERNS.items():
                for match in pattern.finditer(content):
                    trig_start = boundaries[start_char + match.start()]
                    trig_end = boundaries[start_char + match.end()]
                    candidates.append({
                        "candidate_id": f"c-{len(candidates)+1}",
                        "kind": kind,
                        "evidence_unit_id": unit_id,
                        "start_byte": start_byte, "end_byte": end_byte,
                        "quote": content,
                        "trigger_start_byte": trig_start, "trigger_end_byte": trig_end,
                        "trigger": match.group(),
                        "status": "uninterpreted_lexical_candidate",
                    })
        char_offset += len(raw_line)
    result = {
        "schema_version": SCHEMA_VERSION,
        "document_id": document_id,
        "original_sha256": hashlib.sha256(data).hexdigest(),
        "original_length_bytes": len(data),
        "encoding": "utf-8-strict-no-normalization",
        "extractor": "lexical-lines-v1",
        "assurance": {
            "input_fidelity": "byte_spans_verified",
            "legal_interpretation": "not_performed",
            "external_authority": "not_checked",
            "formal_proof": "not_generated",
            "full_integrated_runtime": False,
            "production_qualified": False,
        },
        "units": units, "candidates": candidates,
    }
    verify_result(data, result)
    return result


def verify_result(data: bytes, result: dict) -> None:
    if hashlib.sha256(data).hexdigest() != result.get("original_sha256"):
        raise IngestionError("document hash mismatch")
    if result.get("original_length_bytes") != len(data):
        raise IngestionError("document length mismatch")
    units = {unit["unit_id"]: unit for unit in result.get("units", [])}
    for item in [*result.get("units", []), *result.get("candidates", [])]:
        s, e, quote = item.get("start_byte"), item.get("end_byte"), item.get("quote")
        if type(s) is not int or type(e) is not int or not 0 <= s < e <= len(data):
            raise IngestionError("invalid evidence span")
        if not isinstance(quote, str) or data[s:e] != quote.encode("utf-8"):
            raise IngestionError("evidence quote does not match original bytes")
    for c in result.get("candidates", []):
        if c.get("evidence_unit_id") not in units:
            raise IngestionError("candidate references non-existent source unit")
        s, e = c.get("trigger_start_byte"), c.get("trigger_end_byte")
        if type(s) is not int or type(e) is not int or not 0 <= s < e <= len(data):
            raise IngestionError("invalid trigger offsets")
        if data[s:e] != c.get("trigger", "").encode("utf-8"):
            raise IngestionError("candidate trigger does not match source bytes")
        if not (c["start_byte"] <= s < e <= c["end_byte"]):
            raise IngestionError("candidate trigger outside evidence")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--docs-root", required=True, help="Trusted directory containing UTF-8 text")
    parser.add_argument("--document", required=True, help="Relative .txt/.md path within docs-root")
    parser.add_argument("--output", help="Optional output JSON file; otherwise print to stdout")
    args = parser.parse_args()
    try:
        relative, data = read_from_root(args.docs_root, args.document)
        artifact = parse_document(data, document_id=relative)
        output = json.dumps(artifact, ensure_ascii=False, sort_keys=True, indent=2) + "\n"
        if args.output:
            Path(args.output).write_text(output, encoding="utf-8")
        else:
            print(output, end="")
    except (IngestionError, OSError) as exc:
        parser.exit(2, f"Ingestion refused: {exc}\n")


if __name__ == "__main__":
    main()
