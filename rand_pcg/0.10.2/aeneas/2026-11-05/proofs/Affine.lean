import Aeneas
namespace AffineAdvance
abbrev Affine (w : Nat) := BitVec w × BitVec w
/-- Compose left-to-right: apply (mul f g) = apply g after apply f. -/
def mul (f g : Affine w) : Affine w := (f.1*g.1, f.2*g.1+g.2)
def one (w : Nat) : Affine w := (1#w,0#w)
def power (f : Affine w) : Nat → Affine w
  | 0 => one w
  | n+1 => mul (power f n) f

theorem mul_assoc (f g h : Affine w) : mul (mul f g) h = mul f (mul g h) := by
  apply Prod.ext
  · exact BitVec.mul_assoc _ _ _
  · change (f.2*g.1+g.2)*h.1+h.2=f.2*(g.1*h.1)+(g.2*h.1+h.2)
    rw [BitVec.add_mul, BitVec.mul_assoc, BitVec.add_assoc]
@[simp] theorem mul_one (f : Affine w) : mul f (one w) = f := by
  cases f
  simp [mul,one]
@[simp] theorem one_mul (f : Affine w) : mul (one w) f = f := by
  cases f
  simp [mul,one]
@[simp] theorem power_zero (f : Affine w) : power f 0=one w := rfl
theorem power_succ (f : Affine w) (n : Nat) : power f (n+1)=mul (power f n) f := rfl
@[simp] theorem power_one (f : Affine w) : power f 1=f := by simp [power]

theorem power_add (f : Affine w) (n m : Nat) :
    power f (n+m)=mul (power f n) (power f m) := by
  induction m with
  | zero => simp
  | succ m ih => rw [← Nat.add_assoc, power_succ, ih, mul_assoc, power_succ]

theorem square_power (f : Affine w) (n : Nat) : power (mul f f) n=power f (n+n) := by
  induction n with
  | zero => rfl
  | succ n ih =>
    rw [power_succ, ih]
    have h : n+1+(n+1)=(n+n)+1+1 := by omega
    rw [h, power_succ, power_succ]
    exact (mul_assoc _ _ _).symm

theorem binary_power (f : Affine w) (n : Nat) :
    power f n=if n%2=0 then power (mul f f) (n/2)
      else mul f (power (mul f f) (n/2)) := by
  rw [square_power]
  split
  next h => congr 1; omega
  next h =>
    have hn : n=1+(n/2+n/2) := by omega
    calc
      power f n = power f (1+(n/2+n/2)) := congrArg (power f) hn
      _ = mul (power f 1) (power f (n/2+n/2)) := power_add _ _ _
      _ = mul f (power f (n/2+n/2)) := by rw [power_one]

/-- Ordinary linear repetition, independent of binary exponentiation. -/
def iterate (m c : BitVec w) : Nat → BitVec w → BitVec w
  | 0,x => x
  | n+1,x => m*(iterate m c n x)+c

theorem square_pair (c q : BitVec w) :
    (c*c,(c+1#w)*q)=mul (c,q) (c,q) := by
  unfold mul
  rw [BitVec.add_mul,BitVec.one_mul,BitVec.mul_comm c q]

def applyAffine (f : Affine w) (x : BitVec w) := f.1*x+f.2

theorem apply_mul (f g : Affine w) (x : BitVec w) :
    applyAffine (mul f g) x=applyAffine g (applyAffine f x) := by
  have hm : f.1*g.1*x=g.1*(f.1*x) := by
    rw [BitVec.mul_comm f.1 g.1, BitVec.mul_assoc]
  unfold applyAffine mul
  rw [BitVec.mul_add, hm, BitVec.mul_comm g.1 f.2, BitVec.add_assoc]

theorem apply_power (f : Affine w) (n : Nat) (x : BitVec w) :
    applyAffine (power f n) x=iterate f.1 f.2 n x := by
  induction n with
  | zero => simp [power,one,applyAffine,iterate]
  | succ n ih =>
    rw [power_succ,apply_mul,ih]
    rfl
end AffineAdvance
#print axioms AffineAdvance.mul_assoc
#print axioms AffineAdvance.binary_power
#print axioms AffineAdvance.apply_power
