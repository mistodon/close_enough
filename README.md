close_enough
===

To do
---

1.  Cleaner input reading
2.  Replace panics with elegant error reporting and exit code
3.  Fix the Vec<T> -> Vec<&T> conversion problem
4.  Be a bit less copy-happy (recursive function?)
5.  Add convenience cli endpoint for ce to make the help/usage message prettier
    1.  Should return non-zero on help so that the following cd command can be ignored
6.  Fix bodge for gen subcommand (SubcommandsNegateReqs)