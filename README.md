# Example in rust of the core part of foreman

It is an implementation example in rust of IO processing and signal processing in multithread as done by [foreman](https://github.com/ddollar/foreman) of ruby.  

- IO process in multi thread
- handle signlas in another thread
- child wait in another thread

Since it is a sample, unlike `foreman`, it does not have the flexibility to define a process in `Procfile`.  
The processes that can be executed are as follows.  

|process|concurrency|command|
|---|-----------|-------|
|exit_0|1| `sleep 5 && echo 'success' && exit 0;`|
|exit_1|1| `sleep 5 && echo 'failed' && exit 1;`|
|loop|2|`while :; do sleep 1 && echo 'hello world'; done;`|


The behavior is that when exit_0 or exit_1 exits after 5 seconds, the remaining child processes will be signaled with a `SIGTERM` and killed.

![image](https://user-images.githubusercontent.com/11146767/99907357-bd69dd80-2d1f-11eb-8bb0-aa67d604baee.png)

<details>

```
$ cargo run
    Finished dev [unoptimized + debuginfo] target(s) in 0.09s
     Running `target/debug/eg_foreman`
system    | exit_0.1  start at pid: 42932
system    | exit_1.1  start at pid: 42933
system    | loop.2    start at pid: 42934
system    | loop.1    start at pid: 42935
loop.1    | hello world
loop.2    | hello world
loop.2    | hello world
loop.1    | hello world
loop.2    | hello world
loop.1    | hello world
loop.2    | hello world
loop.1    | hello world
exit_1.1  | failed
exit_0.1  | success
system    | sending SIGTERM for exit_0.1 at pid 42932
system    | sending SIGTERM for loop.2 at pid 42934
system    | sending SIGTERM for loop.1 at pid 42935
system    | exit 1
```

</details>

If <kbd>ctrl-c</kbd> is detected within 5 seconds, `SIGTERM` will be sent to all child processes and the process will be killed.

![image](https://user-images.githubusercontent.com/11146767/99907366-c9ee3600-2d1f-11eb-809f-7ab562ee3698.png)

<details>

```
$ cargo run
    Finished dev [unoptimized + debuginfo] target(s) in 0.09s
     Running `target/debug/eg_foreman`
system    | exit_0.1  start at pid: 43204
system    | loop.1    start at pid: 43205
system    | exit_1.1  start at pid: 43206
system    | loop.2    start at pid: 43207
loop.2    | hello world
loop.1    | hello world
loop.1    | hello world
loop.2    | hello world
^Csystem  | ctrl-c detected
system    | sending SIGTERM for children
system    | sending SIGTERM for exit_0.1  at pid 43204
system    | sending SIGTERM for loop.1    at pid 43205
system    | sending SIGTERM for exit_1.1  at pid 43206
system    | sending SIGTERM for loop.2    at pid 43207
system    | exit 0
```

</details>

If you generalize this, you can make a foreman. You did it. 🎉


## Environment

|name|desc|defaul|
|----|----|------|
|COLOR|Color the output|true|

## Reference

I really referred to the implementation of the following repository.

- [fors](https://github.com/jtdowney/fors)
- [arpx](https://github.com/jaredgorski/arpx)
