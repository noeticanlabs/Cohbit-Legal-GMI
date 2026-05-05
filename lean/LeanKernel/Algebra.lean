import Lean

namespace LegalKernel

inductive Decision
| accept
| reject
deriving DecidableEq, Repr

structure State where
  valuation : Int
  hash : Nat
deriving DecidableEq, Repr

structure Delta where
  id : Nat
deriving DecidableEq, Repr

structure Witness where
  id : Nat
deriving DecidableEq, Repr

structure Receipt where
  stepIndex : Nat
  prevHash : Option Nat
  preHash : Nat
  postHash : Nat
  receiptHash : Nat
  decision : Decision
deriving DecidableEq, Repr

structure VerifierResult where
  decision : Decision
  deltaV : Int
  spend : Int
  defect : Int
deriving DecidableEq, Repr

def stable (r : VerifierResult) : Prop :=
  r.deltaV >= r.spend - r.defect

instance (r : VerifierResult) : Decidable (stable r) :=
  inferInstanceAs (Decidable (r.deltaV >= r.spend - r.defect))

structure ApplyResult where
  pre : State
  post : State
  result : VerifierResult
  receipt? : Option Receipt

def apply (s : State) (r : VerifierResult) (nextHash : Nat) : ApplyResult :=
  if r.decision = Decision.accept ∧ stable r then
    {
      pre := s,
      post := { valuation := s.valuation - r.deltaV, hash := nextHash },
      result := r,
      receipt? := some {
        stepIndex := 0,
        prevHash := none,
        preHash := s.hash,
        postHash := nextHash,
        receiptHash := nextHash + 1,
        decision := r.decision
      }
    }
  else
    {
      pre := s,
      post := s,
      result := r,
      receipt? := none
    }

-- === M8 THEOREM TARGETS ===

theorem reject_no_mutation
  (s : State) (r : VerifierResult) (nextH : Nat)
  (h : r.decision = Decision.reject) :
  (apply s r nextH).post = (apply s r nextH).pre := by
  unfold apply
  have h_cond : ¬(r.decision = Decision.accept ∧ stable r) := by
    intro h_and
    have h_dec := h_and.left
    rw [h] at h_dec
    contradiction
  simp [h_cond]

theorem accept_requires_gauntlet
  (s : State) (r : VerifierResult) (nextH : Nat)
  (h_mut : (apply s r nextH).post ≠ (apply s r nextH).pre) :
  r.decision = Decision.accept := by
  unfold apply at h_mut
  by_cases h_cond : r.decision = Decision.accept ∧ stable r
  · exact h_cond.left
  · simp [h_cond] at h_mut

theorem accept_requires_stability
  (s : State) (r : VerifierResult) (nextH : Nat)
  (h_mut : (apply s r nextH).post ≠ (apply s r nextH).pre) :
  stable r := by
  unfold apply at h_mut
  by_cases h_cond : r.decision = Decision.accept ∧ stable r
  · exact h_cond.right
  · simp [h_cond] at h_mut

theorem valuation_continuity
  (s : State) (r : VerifierResult) (nextH : Nat)
  (h_acc : r.decision = Decision.accept ∧ stable r) :
  (apply s r nextH).post.valuation = s.valuation - r.deltaV := by
  unfold apply
  simp [h_acc]

end LegalKernel
