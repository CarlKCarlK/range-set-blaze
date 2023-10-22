// does not validate

include "Rule3.dfy"

method InternalAdd(xs: seq<NeIntRange>, range: IntRange) returns (r: seq<NeIntRange>)
  requires ValidSeq(xs)
  ensures ValidSeq(r)
  ensures SeqToSet(r) == SeqToSet(xs) + RangeToSet(range)
{
  var (start, end) := range;
  if end < start {
    r := xs;
    assert SeqToSet(r) == SeqToSet(xs) + RangeToSet(range); // case 0 - validates
    return;
  }

  var beforeHi := IndexAtOrBeforePlusOne(xs, start);
  if beforeHi > 0 { // does not go at front
    var (startBefore, endBefore) := xs[beforeHi-1];
    if endBefore+1 < start {
      r := InternalAdd2(xs, range);
      assert SeqToSet(r) == SeqToSet(xs) + RangeToSet(range); // case 1 - validates
    } else if endBefore < end {
      r := xs[..beforeHi-1] + [(startBefore, end)] + xs[beforeHi..];
      assume exists i: nat :: i < |r| && r[i] == (startBefore,end) && ValidSeq(r[..i+1]) && ValidSeq(r[i+1..]);
      r := DeleteExtra(r, (startBefore,end));
      assert SeqToSet(r) == SeqToSet(xs) + RangeToSet(range); // case 2 - fails
    } else{
      r := xs;
      assert RangeToSet(xs[beforeHi-1]) >= RangeToSet(range);
      assert xs[beforeHi-1] in xs;
      SupersetOfPartsLemma(xs, xs[beforeHi-1]);
      assert SeqToSet(r) == SeqToSet(xs) + RangeToSet(range); // case 3 - now validates
    }
  }
  else // goes at front
  {
    r := InternalAdd2(xs, range);
    assert SeqToSet(r) == SeqToSet(xs) + RangeToSet(range); // case 4 - validates
  }
}

lemma SupersetOfPartsLemma(xs: seq<NeIntRange>, range: NeIntRange)
  requires ValidSeq(xs)
  requires range in xs
  ensures SeqToSet(xs) >= RangeToSet(range)
{
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
  {
    i := i + 1;
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
  {
    var (startDelete, endDelete) := xs[indexDel];
    if startDelete <= end + 1 // e.g. touch?
    {
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
  r := r2;
}

method InsertNe(s: seq<NeIntRange>, pair: NeIntRange) returns (r: seq<NeIntRange>, wasThere: bool)
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
  }
  else
  {
    r := s[..i] + [pair] + s[i..];
  }

  wasThere := false;
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
  {
    i := i + 1;
  }
}

function DeleteFromList(r: seq<NeIntRange>, deleteList: seq<int>,
                        indexAfterOne: nat, indexDel: nat
) : seq<NeIntRange>
  requires 0 < indexAfterOne <= |r|
  requires indexAfterOne <= indexDel <= |r|
{
  r[..indexAfterOne] + (if indexDel < |r| then r[indexDel..] else [])
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