import Lean

namespace LegalKernel

inductive Decision
| accept
| reject
| escalate
deriving DecidableEq, Repr

structure State where
  valuation : Rat
  hash : Nat
  stepIndex : Nat
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
  deltaV : Rat
  spend : Rat
  defect : Rat
deriving DecidableEq, Repr

def stable (r : VerifierResult) : Prop :=
  r.deltaV >= r.spend - r.defect

def contentChanged (before after : State) : Prop :=
  before.valuation ≠ after.valuation ∨ before.hash ≠ after.hash

instance (r : VerifierResult) : Decidable (stable r) :=
  inferInstanceAs (Decidable (r.deltaV >= r.spend - r.defect))

structure ApplyResult where
  pre : State
  post : State
  result : VerifierResult
  receipt : Receipt

def apply (s : State) (r : VerifierResult) (nextHash : Nat) : ApplyResult :=
  if r.decision = Decision.accept ∧ stable r then
    {
      pre := s,
      post := { valuation := s.valuation - r.deltaV, hash := nextHash, stepIndex := s.stepIndex + 1 },
      result := r,
      receipt := {
        stepIndex := s.stepIndex + 1,
        prevHash := some s.hash,
        preHash := s.hash,
        postHash := nextHash,
        receiptHash := nextHash + 1,
        decision := Decision.accept
      }
    }
  else
    {
      pre := s,
      post := { valuation := s.valuation, hash := s.hash, stepIndex := s.stepIndex + 1 },
      result := r,
      receipt := {
        stepIndex := s.stepIndex + 1,
        prevHash := some s.hash,
        preHash := s.hash,
        postHash := s.hash,
        receiptHash := s.hash + 2,
        decision := if r.decision = Decision.escalate then Decision.escalate else Decision.reject
      }
    }

-- === M8 THEOREM TARGETS ===

theorem reject_preserves_state_content
  (s : State) (r : VerifierResult) (nextH : Nat)
  (h : r.decision = Decision.reject) :
  (apply s r nextH).post.valuation = s.valuation ∧ 
  (apply s r nextH).post.hash = s.hash := by
  unfold apply
  have h_cond : ¬(r.decision = Decision.accept ∧ stable r) := by
    intro h_and
    have h_dec := h_and.left
    rw [h] at h_dec
    contradiction
  simp [h_cond]

theorem apply_always_increments_step
  (s : State) (r : VerifierResult) (nextH : Nat) :
  (apply s r nextH).post.stepIndex = s.stepIndex + 1 := by
  unfold apply
  by_cases h_cond : r.decision = Decision.accept ∧ stable r
  · simp [h_cond]
  · simp [h_cond]

theorem accept_requires_gauntlet
  (s : State) (r : VerifierResult) (nextH : Nat)
  (h_mut : contentChanged (apply s r nextH).pre (apply s r nextH).post) :
  r.decision = Decision.accept := by
  unfold contentChanged apply at h_mut
  by_cases h_cond : r.decision = Decision.accept ∧ stable r
  · exact h_cond.left
  · simp [h_cond] at h_mut

theorem accept_requires_stability
  (s : State) (r : VerifierResult) (nextH : Nat)
  (h_mut : contentChanged (apply s r nextH).pre (apply s r nextH).post) :
  stable r := by
  unfold contentChanged apply at h_mut
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
