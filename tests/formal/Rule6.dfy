include "Rule3.dfy"

method InternalAdd(xs: seq<NeIntRange>, a: IntRange) returns (rs: seq<NeIntRange>)
  requires ValidSeq(xs)
  ensures ValidSeq(rs)
  ensures SeqToSet(rs) == SeqToSet(xs) + RangeToSet(a)
{
  if IsEmpty(a)
  {
    rs := xs;
  }
  else
  {
    var notTouching, merged := PartitionAndMerge(xs, a);
    var indexAfter := NoTouchIndexAfter(notTouching, merged);
    rs := InsertAt(notTouching, [merged], indexAfter);
  }
}

ghost function RangeToSet(pair: IntRange): set<int>
{
  set i {:autotriggers false} | pair.0 <= i <= pair.1 :: i
}

ghost function SeqToSet(sequence: seq<NeIntRange>): set<int>
  decreases |sequence|
  requires ValidSeq(sequence)
{
  if |sequence| == 0 then {}
  else if |sequence| == 1 then RangeToSet(sequence[0])
  else RangeToSet(sequence[0]) + SeqToSet(sequence[1..])
}

method PartitionAndMerge(xs: seq<NeIntRange>, a: NeIntRange) returns (notTouching: seq<NeIntRange>, merged: NeIntRange)
  requires ValidSeq(xs)

  ensures ValidSeq(notTouching)
  ensures RangeToSet(merged) >= RangeToSet(a)
  ensures forall range | range in notTouching :: !Touch(range, merged)
  ensures SeqToSet(xs) + RangeToSet(a) == SeqToSet(notTouching) + RangeToSet(merged)
{
  // Split into touching and not touching seqs
  var touching: seq<NeIntRange>;
  touching, notTouching := Partition(a, xs);

  // Merge the touching seq into one range with our original range
  merged := UnionSeq(a, touching);
}

method Partition(a: NeIntRange, xs: seq<NeIntRange>) returns (touching: seq<NeIntRange>, notTouching: seq<NeIntRange>)
  requires ValidSeq(xs)

  ensures ValidSeq(touching) && ValidSeq(notTouching)
  ensures forall range | range in xs :: (range in touching) != (range in notTouching)
  ensures forall i:nat | i < |touching| :: touching[i] in xs
  ensures forall i:nat | i < |notTouching| :: notTouching[i] in xs
  ensures forall i:nat | i < |xs| && Touch(xs[i], a) :: xs[i] in touching
  ensures forall i:nat | i < |xs| && !Touch(xs[i], a) :: xs[i] in notTouching
  ensures SeqToSet(touching) + SeqToSet(notTouching) == SeqToSet(xs)
{
  var touchA := (b: NeIntRange) => Touch(a,b);
  if |xs| == 0
  {
    touching, notTouching := [], [];
  }
  else
  {
    if Touch(xs[0],a)
    {
      var t1, nt1 := Partition(a, xs[1..]);
      touching := [xs[0]] + t1;
      notTouching := nt1;
    }
    else
    {
      var t1, nt1 := Partition(a, xs[1..]);
      touching := t1;
      notTouching := [xs[0]] + nt1;
    }
  }
}

method NoTouchIndexAfter(xs: seq<NeIntRange>, a: NeIntRange) returns (i: nat)
  requires ValidSeq(xs)
  requires forall j: nat | j < |xs| :: !Touch(xs[j], a)
  ensures i <= |xs|
  ensures forall p | p in xs[..i] :: p.1+1 < a.0
  ensures forall p | p in xs[i..] :: a.1+1 < p.0
{
  i := 0;
  while i < |xs| && xs[i].1+1 < a.0
    invariant i <= |xs|
    invariant forall j:nat | j < i :: xs[j].0 < a.0
  {
    i := i + 1;
  }
}

method InsertAt(xs: seq<NeIntRange>, ys: seq<NeIntRange>, i: nat) returns (r: seq<NeIntRange>)
  requires ValidSeq(xs)
  requires ValidSeq(ys)
  requires i <= |xs|
  requires forall p | p in xs[..i] :: forall q | q in ys :: p.1+1 < q.0
  requires forall p | p in ys :: forall q | q in xs[i..] :: p.1+1 < q.0
  ensures ValidSeq(r)
  ensures SeqToSet(r) == SeqToSet(xs) + SeqToSet(ys)
  ensures r == xs[..i] + ys + xs[i..]
{
  var left, right := SplitAt(xs, i);
  var left2 := Concat(left,ys);
  r := Concat(left2, right);
}

method SplitAt(xs: seq<NeIntRange>, i: nat) returns (left: seq<NeIntRange>, right: seq<NeIntRange>)
  requires ValidSeq(xs)
  requires i <= |xs|

  ensures ValidSeq(left)
  ensures ValidSeq(right)
  ensures forall p | p in left :: forall q | q in right :: p.1+1 < q.0
  ensures forall i: nat | i < |left| :: forall j: nat | j < |right| :: left[i].1+1 < right[j].0
  ensures SeqToSet(xs) == SeqToSet(left) + SeqToSet(right)
  ensures SeqToSet(left) !! SeqToSet(right)
  ensures xs == left + right
  ensures |left| == i
{
  left := xs[..i];
  ElementsBelowAndBelowLemma(xs[..i]);
  right := xs[i..];
  ElementsBelowAndBelowLemma(xs[i..]);
  ghost var c := Concat(left,right);
  assert xs == c;
}

method Concat(xs: seq<NeIntRange>, ys: seq<NeIntRange>) returns (r: seq<NeIntRange>)
  requires ValidSeq(xs)
  requires ValidSeq(ys)
  requires forall p | p in xs :: forall q | q in ys :: p.1+1 < q.0
  ensures ValidSeq(r)
  ensures SeqToSet(r) == SeqToSet(xs) + SeqToSet(ys)
  ensures r == xs + ys
{
  if xs == [] {
    r := ys;
  }
  else {
    var t := Concat(xs[1..], ys);
    r := [xs[0]] +  t;
  }
}

method UnionSeq(a: NeIntRange, touching: seq<NeIntRange>) returns (merged: NeIntRange)
  requires ValidSeq(touching)
  requires forall i:nat | i < |touching| :: Touch(a, touching[i])
  ensures forall el | Contains(merged,el) :: Contains(a, el) || exists i:nat :: i < |touching| && Contains(touching[i], el)
  ensures forall i:nat | i < |touching| :: forall el :: Contains(touching[i], el) ==> Contains(merged, el)
  ensures forall el | Contains(a, el) :: Contains(merged, el)
  ensures RangeToSet(merged) == RangeToSet(a) + SeqToSet(touching)
{
  if |touching| == 0 {
    merged := a;
  }
  else
  {
    var rest_set := UnionSeq(a, touching[1..]);
    merged := UnionRange(touching[0], rest_set);
  }
}

lemma ElementsBelowAndBelowLemma(xs: seq<NeIntRange>)
  requires ValidSeq(xs)
  ensures forall e | e in SeqToSet(xs) :: e <= xs[|xs|-1].1
  ensures forall e | e in SeqToSet(xs) :: xs[0].0 <= e
{
}

function UnionRange(x: IntRange, y: IntRange): IntRange
  requires IsEmpty(x) || IsEmpty(y) || Touch(x, y)
  ensures RangeToSet(x) + RangeToSet(y) == RangeToSet(UnionRange(x,y))
{
  if IsEmpty(x) then y
  else if IsEmpty(y) then x
  else (Min(x.0, y.0), Max(x.1, y.1))
}

function Min(a: int, b: int): int
{
  if a < b
  then a
  else b
}

method Main()
{
  var xs := [(101,102), (400,402), (404,404), (500,500)];
  var a := (401,403);
  var rs := InternalAdd(xs, a);
  print rs;
}