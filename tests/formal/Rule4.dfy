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
    assume false; // cheat for now
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