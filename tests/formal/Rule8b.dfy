include "Rule3.dfy"

method InternalAdd(xs: seq<NeIntRange>, range: IntRange) returns (r: seq<NeIntRange>)
  requires ValidSeq(xs)
  ensures ValidSeq(r)
  ensures SeqToSet(r) == SeqToSet(xs) + RangeToSet(range)
{
  var (start, end) := range;
  if end < start {
    r := xs;
    return;
  }

  var beforeHi := IndexAtOrBeforePlusOne(xs, start);
  if beforeHi > 0 { // does not go at front
    var (startBefore, endBefore) := xs[beforeHi-1];
    if endBefore+1 < start {
      r := InternalAdd2(xs, range);
    } else if endBefore < end {
      r := xs[..beforeHi-1] + [(startBefore, end)] + xs[beforeHi..];
      InsideOut7Lemma(xs[..beforeHi-1], xs[beforeHi-1], range, (startBefore, end), xs[beforeHi..]);
      assert xs == xs[..beforeHi-1] + [xs[beforeHi-1]] + xs[beforeHi..];
      DeleteLemma(xs, r, range, (startBefore, end), beforeHi);
      r := DeleteExtra(r, (startBefore,end));
    } else{
      SupersetOfPartsLemma(xs, xs[beforeHi-1]);
      r := xs;
    }
  }
  else // goes at front
  {
    r := InternalAdd2(xs, range);
  }
}

method DeleteExtra(xs: seq<NeIntRange>, internalRange: IntRange) returns (r: seq<NeIntRange>)
  requires forall i:nat,j:nat :: i < j < |xs| ==> xs[i].0 < xs[j].0 // starts are sorted
  requires exists i: nat :: i < |xs| && xs[i] == internalRange && ValidSeq(xs[..i+1]) && ValidSeq(xs[i+1..]) // each half is valid
  requires forall i:nat,j:nat :: i < j < |xs| && xs[i] != internalRange && xs[j] != internalRange ==> !Touch(xs[i], xs[j]) // only special might touch
  ensures ValidSeq(r) // result must be valid
  ensures exists i: nat :: i < |xs| && xs[i] == internalRange && SeqToSet(xs[..i+1]) + SeqToSet(xs[i+1..]) == SeqToSet(r) // result must cover same set as the halves
{
  var (start, end) := internalRange;
  var indexAfter := IndexAtOrAfter(xs, start);
  var (startAfter, endAfter) := xs[indexAfter];
  var endNew := end;
  var deleteList := [];
  var indexDel := indexAfter+1;
  while indexDel < |xs|
    invariant indexDel <= |xs|
    invariant endNew == end || endNew == xs[indexDel-1].1
    invariant RangeToSet(xs[indexAfter]) + SeqToSet(xs[indexAfter+1..indexDel]) == RangeToSet((start, endNew))
  {
    var (startDelete, endDelete) := xs[indexDel];
    if startDelete <= end + 1 // e.g. touch?
    {
      CoverMoreLemma(xs, indexAfter, indexDel, endNew);
      endNew := Max(endNew, endDelete);
      deleteList := deleteList + [startDelete];
      indexDel := indexDel + 1;
    }
    else
    {
      break;
    }
  }

  if endNew > end
  {
    endAfter := endNew;
    r := xs[indexAfter :=  (start, endAfter)];
  }
  else
  {
    r := xs;
  }
  var r2 := DeleteFromList(r, deleteList, indexAfter+1, indexDel);
  SetsEqualLemma(xs, r, r2, indexAfter, indexDel);
  r := r2;
}

method InternalAdd2(xs: seq<NeIntRange>, internalRange: NeIntRange) returns (r: seq<NeIntRange>)
  requires ValidSeq(xs)
  requires forall i : nat :: i < |xs| && xs[i].0 < internalRange.0 ==> xs[i].1+1 < internalRange.0
  requires forall i: nat :: i < |xs| ==> xs[i].0 != internalRange.0
  ensures ValidSeq(r)
  ensures SeqToSet(r) == SeqToSet(xs) + RangeToSet(internalRange)
{
  var wasThere: bool;
  r, wasThere := InsertNe(xs, internalRange);
  assert(!wasThere); // in Rust code
  r := DeleteExtra(r, internalRange);
}


ghost function RangeToSet(pair: IntRange): set<int>
{
  set i | pair.0 <= i <= pair.1 :: i
}

ghost function SeqToSet(sequence: seq<NeIntRange>): set<int>
  decreases |sequence|
  requires ValidSeq(sequence)
{
  if |sequence| == 0 then {}
  else if |sequence| == 1 then RangeToSet(sequence[0])
  else RangeToSet(sequence[0]) + SeqToSet(sequence[1..])
}

method IndexAtOrBeforePlusOne(xs: seq<NeIntRange>, start: int) returns (i: nat)
  requires ValidSeq(xs)
  ensures i <= |xs|
  ensures forall p | p in xs[..i] :: p.0 <= start
  ensures forall j: nat | j < i :: xs[j].0 <= start
  ensures forall j: nat | i <= j < |xs| :: start < xs[j].0
  ensures i > 0 ==> xs[i-1].0 <= start
  ensures forall j: nat :: j < |xs| && xs[j].0 == start ==> j + 1 == i
  ensures forall j: nat | j < |xs| :: xs[j].0 == start ==> j + 1 == i
{
  i := 0;
  while i < |xs| && xs[i].0 <= start
    invariant i <= |xs|
    invariant forall j:nat | j < i :: xs[j].0 <= start
    invariant forall j: nat | j < i-1 :: xs[j].0 < start
  {
    i := i + 1;
  }
}

lemma InsideOut7Lemma(a: seq<NeIntRange>, b: NeIntRange, newRange: NeIntRange, b': NeIntRange, c : seq<NeIntRange>)
  requires ValidSeq(a + [b] + c)
  requires ValidSeq(c) && ValidSeq(a)
  requires ValidSeq(a + [b'])
  requires RangeToSet(b) + RangeToSet(newRange) == RangeToSet(b')
  ensures SeqToSet(a + [b] + c) + RangeToSet(newRange) == SeqToSet(a + [b']) + SeqToSet(c)
{
  InsideOut1Lemma(a, b, c);
  InsideOut2Lemma(a, b');
}

lemma DeleteLemma(xs: seq<NeIntRange>, r: seq<NeIntRange>, range: NeIntRange, range2: NeIntRange, beforeHi: nat)
  requires 0 < beforeHi <= |r|
  requires  |r| == |xs|
  requires ValidSeq(xs)
  requires ValidSeq(r[..beforeHi])
  requires r == xs[..beforeHi-1] + [range2] + xs[beforeHi..]
  requires RangeToSet(range2) >= RangeToSet(xs[beforeHi-1])
  ensures exists i: nat :: i == beforeHi-1 && r[i] == range2 && ValidSeq(r[..i+1]) && ValidSeq(r[i+1..])
{
  assert r[beforeHi-1] == range2 && ValidSeq(r[..beforeHi]) && ValidSeq(r[beforeHi..]);
}

lemma SupersetOfPartsLemma(xs: seq<NeIntRange>, range: NeIntRange)
  requires ValidSeq(xs)
  requires range in xs
  ensures SeqToSet(xs) >= RangeToSet(range)
{

}

method IndexAtOrAfter(xs: seq<NeIntRange>, start: int) returns (i: nat)
  requires forall i:nat,j:nat :: i < j < |xs| ==> xs[i].0 < xs[j].0
  ensures i <= |xs|
  ensures forall p | p in xs[..i] :: p.0 < start
  ensures forall j: nat | j < i :: xs[j].0 < start
  ensures forall j: nat | i <= j < |xs| :: start <= xs[j].0
{
  i := 0;
  while i < |xs| && xs[i].0 < start
    invariant i <= |xs|
    invariant forall j:nat | j < i :: xs[j].0 < start
  {
    i := i + 1;
  }
}

lemma CoverMoreLemma(xs : seq<NeIntRange>, a: nat, d: nat, endNew: int)
  requires forall i:nat,j:nat :: i < j < |xs| ==> xs[i].0 < xs[j].0
  requires a < d < |xs|
  requires ValidSeq(xs[a+1..])
  requires RangeToSet(xs[a]) + SeqToSet(xs[a+1..d]) + RangeToSet(xs[d]) == RangeToSet((xs[a].0, endNew))+RangeToSet(xs[d])
  requires RangeToSet((xs[a].0, endNew))+RangeToSet(xs[d]) == RangeToSet((xs[a].0, Max(endNew, xs[d].1)))
  ensures RangeToSet(xs[a]) + SeqToSet(xs[a+1..d+1]) == RangeToSet((xs[a].0, Max(endNew, xs[d].1)))
{
  assert xs[a+1..d] + [xs[d]] == xs[a+1..d+1];
  InsideOut2Lemma(xs[a+1..d], xs[d]);
}

function DeleteFromList(r: seq<NeIntRange>, deleteList: seq<int>,
                        indexAfterOne: nat, indexDel: nat
) : seq<NeIntRange>
  requires 0 < indexAfterOne <= |r|
  requires indexAfterOne <= indexDel <= |r|
{
  r[..indexAfterOne] + (if indexDel < |r| then r[indexDel..] else [])
}

lemma {:vcs_split_on_every_assert} SetsEqualLemma(xs: seq<NeIntRange>, r: seq<NeIntRange>, r2: seq<NeIntRange>, specialIndex: nat, indexDel: nat)
  requires specialIndex < indexDel <= |xs|
  requires specialIndex < |r2|
  requires specialIndex < |xs| == |r|
  requires forall i :nat, j:nat| i < j < |xs| :: xs[i].0 < xs[j].0
  requires forall i:nat,j:nat :: i < j < |xs| && i != specialIndex && j != specialIndex ==> !Touch(xs[i], xs[j]) // only special might touch
  requires RangeToSet(xs[specialIndex]) + SeqToSet(xs[specialIndex+1..indexDel]) == RangeToSet(r[specialIndex])
  requires ValidSeq(r[..specialIndex+1]) && ValidSeq(r[specialIndex+1..]) // each half is valid
  requires ValidSeq(xs[..specialIndex+1]) && ValidSeq(xs[specialIndex+1..]) // each half is valid
  requires if indexDel < |r| then r[indexDel..] ==  r2[specialIndex+1..] else |r2| == specialIndex+1
  requires indexDel < |xs| ==> r[specialIndex].1+1 < r[indexDel].0
  requires r[..specialIndex+1] == r2[..specialIndex+1]
  requires xs[specialIndex := r[specialIndex]]== r
  ensures SeqToSet(xs[..specialIndex+1]) + SeqToSet(xs[specialIndex+1..]) == SeqToSet(r2)
  ensures forall i :nat, j:nat| i < j < |r2| :: r2[i].0 < r2[j].0
  ensures forall i:nat,j:nat :: i < j < |r2| ==> !Touch(r2[i], r2[j])
{
  RDoesntTouchLemma(xs, r, r2, specialIndex, indexDel);
  var a := xs[..specialIndex];
  var b := xs[specialIndex..specialIndex+1];
  var b' := r[specialIndex..specialIndex+1];
  var c := xs[specialIndex+1..indexDel];
  var d := if indexDel < |xs| then xs[indexDel..] else [];
  assert xs[specialIndex+1..] == c+d;
  assert d == if indexDel < |xs| then r[indexDel..] else [];
  InsideOut2Lemma(xs[..specialIndex], xs[specialIndex]);
  assert xs[..specialIndex+1] == xs[..specialIndex] + [xs[specialIndex]];
  assert r2 == a + b' + d;
  InsideOut3Lemma(a, b, xs[..specialIndex+1]);
  InsideOut3Lemma(c, d, xs[specialIndex+1..]);
  InsideOut4Lemma(a,b',d);
  assert SeqToSet(xs[..specialIndex+1]) == SeqToSet(a+b);
  assert SeqToSet(xs[specialIndex+1..]) == SeqToSet(c+d);
  assert SeqToSet(a+b) + SeqToSet(c+d) == SeqToSet(a + b' + d);
}

method {:vcs_split_on_every_assert} InsertNe(s: seq<NeIntRange>, pair: NeIntRange) returns (r: seq<NeIntRange>, wasThere: bool)
  requires ValidSeq(s)
  requires forall i:nat | i < |s| :: s[i].0 < pair.0 ==> s[i].1+1 < pair.0
  requires forall i:nat | i < |s| :: s[i].0 != pair.0
  ensures SortedMapFlat.Valid(r)
  ensures SortedMapFlat.KeyToSet(r) == SortedMapFlat.KeyToSet(s) + {pair.0}
  ensures pair in r
  ensures forall i:nat | i < |s| && s[i].0 != pair.0 :: s[i] in r
  ensures wasThere == (pair.0 in SortedMapFlat.KeyToSet(s))
  ensures exists i: nat :: i < |r| && r[i] == pair && ValidSeq(r[..i+1]) && ValidSeq(r[i+1..])
  ensures exists i: nat :: i < |r| && r[i] == pair && SeqToSet(r[..i+1]) + SeqToSet(r[i+1..]) == SeqToSet(s) + RangeToSet(pair)
{
  var i := IndexAtOrAfter(s, pair.0);
  assert i <= |s|;
  if i == |s| {
    r := s[..i] + [pair] + s[i..];
    assert r == s + [pair];
    assert r[..i+1] == r;
    assert r[i] == pair;
    assert |r| == |s| + 1;
    assert i < |r| && r[i] == pair && ValidSeq(r[..i+1]) && ValidSeq(r[i+1..]);
    InsideOut2Lemma(s, pair);
    assert r == s + [pair];
    assert  SeqToSet(r) == SeqToSet(s) + RangeToSet(pair);
  }
  else
  {
    assert ValidSeq(s[..i] + [pair]);
    assert s == s[..i] + s[i..];
    InsideOut3Lemma(s[..i], s[i..], s);
    assert ValidSeq(s[..i]+s[i..]);
    InsideOut6Lemma(s[..i], pair, s[i..]);
    assert SeqToSet(s[..i] + [pair]) + SeqToSet(s[i..]) == SeqToSet(s[..i] + s[i..]) + RangeToSet(pair);
    r := s[..i] + [pair] + s[i..];
    assert r[..i+1] == s[..i] + [pair];
    assert SeqToSet(r[..i+1]) == SeqToSet(s[..i] + [pair]);
    assert r[i+1..] == s[i..];
    assert SeqToSet(r[i+1..]) == SeqToSet(s[i..]);
    assert s == s[..i] + s[i..] by
    {
      ConcatLemma(s, i);
    }
    assert SeqToSet(s) == SeqToSet(s[..i]) + SeqToSet(s[i..]);
    assert SeqToSet(r[..i+1]) + SeqToSet(r[i+1..]) == SeqToSet(s) + RangeToSet(pair);
  }

  wasThere := false;
}

lemma InsideOut1Lemma(a: seq<NeIntRange>, b: NeIntRange, c: seq<NeIntRange>)
  requires ValidSeq(a) && ValidSeq(c) && ValidSeq(a + [b] + c)
  ensures SeqToSet(a) + RangeToSet(b) + SeqToSet(c) == SeqToSet(a + [b] + c)
{
  if |a| > 0
  {
    assert (a + [b] + c)[1..] == a[1..] + [b] + c;
    InsideOut1Lemma(a[1..], b, c);
  }
}

lemma InsideOut2Lemma(a: seq<NeIntRange>, b: NeIntRange)
  requires ValidSeq(a) && ValidSeq(a + [b])
  ensures SeqToSet(a) + RangeToSet(b)== SeqToSet(a + [b])
{
  if |a | > 0
  {
    assert (a + [b])[1..] == a[1..] + [b];
    InsideOut2Lemma(a[1..], b);
  }
}

lemma RDoesntTouchLemma(xs: seq<NeIntRange>, r: seq<NeIntRange>, r2: seq<NeIntRange>, specialIndex: nat, indexDel: nat)
  requires specialIndex < |xs| == |r|
  requires specialIndex < |r2|
  requires forall i :nat, j:nat| i < j < |xs| :: xs[i].0 < xs[j].0
  requires ValidSeq(xs[..specialIndex+1]) && ValidSeq(xs[specialIndex+1..]) // each half is valid
  requires ValidSeq(r[..specialIndex+1]) && ValidSeq(r[specialIndex+1..]) // each half is valid
  requires forall i:nat,j:nat :: i < j < |xs| && i != specialIndex && j != specialIndex ==> !Touch(xs[i], xs[j]) // only special might touch
  requires xs[specialIndex := r[specialIndex]]== r
  requires specialIndex < indexDel <= |xs|
  requires indexDel < |xs| ==> r[specialIndex].1+1 < r[indexDel].0
  requires r[..specialIndex+1] == r2[..specialIndex+1]
  requires if indexDel < |r| then r[indexDel..] ==  r2[specialIndex+1..] else |r2| == specialIndex+1
  ensures forall i:nat,j:nat :: i < j < |r2| ==> !Touch(r2[i], r2[j])
  ensures forall i :nat, j:nat| i < j < |r2| :: r2[i].0 < r2[j].0
{
}


lemma InsideOut3Lemma(a: seq<NeIntRange>, c: seq<NeIntRange>, d: seq<NeIntRange>)
  requires d == a + c
  requires ValidSeq(a) && ValidSeq(c) && ValidSeq(d)
  ensures SeqToSet(a) + SeqToSet(c) == SeqToSet(d)
{
  if |a| > 0
  {
    assert d[1..] == a[1..] + c;
    InsideOut3Lemma(a[1..], c, d[1..]);
  }
  else
  {
    assert a+c == c == d;
  }
}

lemma {:vcs_split_on_every_assert} InsideOut4Lemma(a: seq<NeIntRange>, b: seq<NeIntRange>, c: seq<NeIntRange>)
  requires ValidSeq(a) && ValidSeq(b) && ValidSeq(c) && ValidSeq(a + b + c) && ValidSeq(b + c)
  ensures SeqToSet(a) + SeqToSet(b) + SeqToSet(c) == SeqToSet(a + b + c)
{
  if |a | > 0
  {
    assert (a + b + c)[1..] == a[1..] + b + c;
  }
  else
  {
    InsideOut3Lemma(b, c, b+c);
    assert [] + b + c == b + c;
  }
}

lemma InsideOut6Lemma(a: seq<NeIntRange>, b: NeIntRange, c: seq<NeIntRange>)
  requires ValidSeq(a) && ValidSeq(c) && ValidSeq(a + c) && ValidSeq(a + [b])
  ensures SeqToSet(a + [b]) + SeqToSet(c) == SeqToSet(a + c) + RangeToSet(b)
{
  if |a| == 0
  {
    InsideOut3Lemma(a, c, a+c);
  }
  else
  {
    assert (a+c)[1..] == a[1..] + c;
    assert (a+[b])[1..] == a[1..] + [b];
  }
}

lemma ConcatLemma(xs: seq<IntRange>, i: nat)
  requires i < |xs|
  ensures xs == xs[..i] + xs[i..]
{
}


module SortedMapFlat {

  // Check if a sequence of integer pairs is sorted and distinct (by key)
  predicate Valid(sorted_seq: seq<(int,int)>) {
    forall i:nat, j:nat | i < j < |sorted_seq| :: sorted_seq[i].0 < sorted_seq[j].0
  }

  function KeyToSet(m: seq<(int,int)>): set<int>
  {
    set i | i in m :: i.0
  }

}