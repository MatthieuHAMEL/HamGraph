Specifiying that "this test should not run with other tests in parallel" :

- https://stackoverflow.com/questions/51694017/how-can-i-avoid-running-some-tests-in-parallel

This is not enough for HAMGRAPH where I use "cargo nextest", since sdl2 also requires that one 
process is started for every test. (Which could actually be possible ... if each test was in 
a separable executable ...)