import LeanKernel.LegalProof

def main : IO UInt32 := do
  let input ← (← IO.getStdin).getLine
  match Lean.Json.parse input with
  | .error _ => return 2
  | .ok json =>
    match Lean.fromJson? (α := LegalProof.Certificate) json with
    | .error _ => return 2
    | .ok cert =>
      if LegalProof.check cert then
        (← IO.getStdout).putStrLn "LEGAL-PROOF-001:Accept"
        return 0
      else
        (← IO.getStdout).putStrLn "LEGAL-PROOF-001:Reject"
        return 1
