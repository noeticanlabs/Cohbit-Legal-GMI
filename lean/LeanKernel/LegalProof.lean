import Lean

/- LEGAL-PROOF-001: finite, positive Horn entailment. Exception absence and
   priority resolution are explicit positive premises in resolvedDefeaters.
   Qualification of those premises is a separate trust obligation. -/
namespace LegalProof

structure Rule where
  body : List Nat
  resolvedDefeaters : List Nat
  head : Nat
  priority : Nat
  defeats : List Nat
deriving Lean.FromJson, Lean.ToJson, Repr

structure Step where
  atom : Nat
  rule : Option Nat
deriving Lean.FromJson, Lean.ToJson, Repr

structure Certificate where
  premises : List Nat
  rules : List Rule
  steps : List Step
  conclusion : Nat
deriving Lean.FromJson, Lean.ToJson, Repr

def priorityCheck (rules : List Rule) (r : Rule) : Bool :=
  r.defeats.all fun i =>
    match rules[i]? with
    | none => false
    | some defeated => r.priority > defeated.priority

def stepCheck (premises known : List Nat) (rules : List Rule) (s : Step) : Bool :=
  match s.rule with
  | none => premises.contains s.atom
  | some i => match rules[i]? with
    | none => false
    | some r => (r.head == s.atom) &&
        (priorityCheck rules r &&
          (r.body ++ r.resolvedDefeaters).all known.contains)

def run (premises : List Nat) (rules : List Rule) : List Step → List Nat → Option (List Nat)
  | [], known => some known
  | s :: rest, known =>
    if stepCheck premises known rules s then
      run premises rules rest (s.atom :: known)
    else none

def check (c : Certificate) : Bool :=
  match run c.premises c.rules c.steps [] with
  | none => false
  | some known => known.contains c.conclusion

def Model (premises : List Nat) (rules : List Rule) (v : Nat → Prop) : Prop :=
  (∀ a ∈ premises, v a) ∧
  (∀ r ∈ rules, (∀ a ∈ r.body ++ r.resolvedDefeaters, v a) → v r.head)

def Entails (premises : List Nat) (rules : List Rule) (a : Nat) : Prop :=
  ∀ v, Model premises rules v → v a

theorem step_sound (p k : List Nat) (rs : List Rule) (s : Step)
    (v : Nat → Prop) (hm : Model p rs v) (hk : ∀ a ∈ k, v a)
    (h : stepCheck p k rs s = true) : v s.atom := by
  unfold stepCheck at h
  split at h
  · exact hm.1 _ (List.contains_iff_mem.mp h)
  · rename_i i hi
    split at h
    · contradiction
    · rename_i r hr
      simp only [Bool.and_eq_true, beq_iff_eq, List.all_eq_true] at h
      have mem : r ∈ rs := List.mem_of_getElem? hr
      have heads : r.head = s.atom := h.1
      rw [← heads]
      exact hm.2 r mem (fun a ha => hk a (List.contains_iff_mem.mp (h.2.2 a ha)))

theorem run_sound (p : List Nat) (rs : List Rule) (steps : List Step)
    (v : Nat → Prop) (hm : Model p rs v) :
    ∀ k out, (∀ a ∈ k, v a) → run p rs steps k = some out → ∀ a ∈ out, v a := by
  induction steps with
  | nil =>
    intro k out hk h
    simp only [run, Option.some.injEq] at h
    subst out
    exact hk
  | cons s rest ih =>
    intro k out hk h
    simp only [run] at h
    split at h
    · rename_i hs
      apply ih (s.atom :: k) out _ h
      intro a ha
      simp only [List.mem_cons] at ha
      rcases ha with heq | ha
      · subst a
        exact step_sound p k rs s v hm hk hs
      · exact hk a ha
    · contradiction

theorem checker_sound (c : Certificate) (h : check c = true) :
    Entails c.premises c.rules c.conclusion := by
  intro v hm
  unfold check at h
  split at h
  · contradiction
  · rename_i known hr
    exact run_sound c.premises c.rules c.steps v hm [] known
      (by simp) hr c.conclusion (List.contains_iff_mem.mp h)

end LegalProof
