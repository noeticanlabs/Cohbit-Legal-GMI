import LeanKernel.Algebra

namespace LegalKernel

structure TransitionTrace where
  stepIndex : Nat
  decision : Decision
  preStateHash : Nat
  postStateHash : Nat
  prevReceiptHash : Option Nat
  receiptHash : Nat
  deltaV : Int
  spend : Int
  defect : Int
  valuationPre : Int
  valuationPost : Int
deriving DecidableEq, Repr

def checkTrace (t : TransitionTrace) : Prop :=
  match t.decision with
  | Decision.reject => 
      t.postStateHash = t.preStateHash ∧ 
      t.valuationPost = t.valuationPre
  | Decision.escalate =>
      t.postStateHash = t.preStateHash ∧
      t.valuationPost = t.valuationPre
  | Decision.accept =>
      t.deltaV >= t.spend - t.defect ∧
      t.valuationPost = t.valuationPre - t.deltaV

theorem trace_accept_is_stable (t : TransitionTrace) (h : checkTrace t) (h_acc : t.decision = Decision.accept) :
  t.deltaV >= t.spend - t.defect := by
  unfold checkTrace at h
  rw [h_acc] at h
  exact h.left

theorem trace_reject_is_idempotent (t : TransitionTrace) (h : checkTrace t) (h_rej : t.decision = Decision.reject) :
  t.postStateHash = t.preStateHash := by
  unfold checkTrace at h
  rw [h_rej] at h
  exact h.left

theorem trace_escalate_is_idempotent (t : TransitionTrace) (h : checkTrace t) (h_esc : t.decision = Decision.escalate) :
  t.postStateHash = t.preStateHash := by
  unfold checkTrace at h
  rw [h_esc] at h
  exact h.left

end LegalKernel
