method SlowAdd(x: nat, y: nat) returns (r: nat)
  ensures r == x + y
{
  r := x;
  var y2 := y;
  while y2 > 0
    invariant r + y2 == x + y
  {
    r := r + 1;
    y2 := y2 - 1;
  }
}

method Main()
{
  var r := SlowAdd(2, 3);
  print r;
}