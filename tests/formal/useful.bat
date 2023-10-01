set path=C:\Users\carlk\.vscode-insiders\extensions\dafny-lang.ide-vscode-3.1.2\out\resources\4.2.0\github\dafny;%path%
dafny verify --help
dafny verify seq_of_sets_example7.dfy --verification-time-limit:30 --cores:20 --log-format csv --boogie -randomSeedIterations:10
dafny verify Step7.dfy --verification-time-limit:30 --cores:32 --log-format csv --boogie -randomSeedIterations:100
