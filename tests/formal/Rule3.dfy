ghost predicate ValidSeq(sequence: seq<NeIntRange>) {
  (forall i:nat, j:nat | i < j < |sequence| :: sequence[i].0 < sequence[j].0)
  && (forall i:nat, j:nat | i < j < |sequence| :: !Touch(sequence[i], sequence[j]))
}

type IntRange = (int, int)
type NeIntRange = x: IntRange | !IsEmpty(x) witness (0,0)

function IsEmpty(r: IntRange): bool
{
  r.0 > r.1
}

predicate Touch(i: NeIntRange, j: NeIntRange)
  ensures Touch(i, j) == exists i0, j0 ::
                           Contains(i, i0) && Contains(j, j0) && -1 <= i0 - j0 <= 1
{
  assert Contains(i, i.0) && Contains(i, i.1) && Contains(j, j.0) && Contains(j, j.1);
  if i.1 < j.0 then
    assert  (-1 <= i.1 - j.0 <= 1) == (i.1+1 == j.0);
    i.1+1 == j.0
  else if j.1 < i.0 then
    assert (-1 <= j.1 - i.0 <= 1) == (j.1+1 == i.0);
    j.1+1 == i.0
  else
    var k0 := Max(i.0, j.0);
    assert Contains(i, k0) && Contains(j, k0);
    true
}

function Contains(r: IntRange, i: int): bool
{
  r.0 <= i && i <= r.1
}

function Max(a: int, b: int): int
{
  if a < b then b else a
}
