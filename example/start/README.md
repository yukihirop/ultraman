# Ultraman start example

If no additional parameters are passed, `ultraman` will run one instance of each type of process defined in your `Procfile`.  

The following options control how the application is run:

|short|long|default|description|
|-----|----|-------|-----------|
|<kbd>-m</kbd>|<kbd>--formation</kbd>|`all=1`|Specify the number of each process type to run. The value passed in should be in the format process=num,process=num|
|<kbd>-e</kbd>|<kbd>--env</kbd>|`.env`|Specify an environment file to load|
|<kbd>-f</kbd>|<kbd>--procfile</kbd>|`Procfile`|Specify an alternate Procfile to load, implies -d at the Procfile root|
|<kbd>-p</kbd>|<kbd>--port</kbd>||Specify which port to use as the base for this application. Should be a multiple of 1000|
|<kbd>-t</kbd>|<kbd>--timeout</kbd>|`5`|Specify the amount of time (in seconds) processes have to shutdown gracefully before receiving a SIGTERM|
|<kbd>-n</kbd>|<kbd>--no-timestamp</kbd>|`false`|Include timestamp in output|

## Example

Here is an example when the `Procfile` and `.env` files have the following contents

[Procfile]
```
exit_0: ./fixtures/exit_0.sh
exit_1: ./fixtures/exit_1.sh
loop: ./fixtures/loop.sh $MESSAGE
```

[.env]
```
MESSAGE="Hello World"
```

### Full option example (short)

```bash
cargo run start \
  -m loop=2,exit_1=3 \
  -e ./.env \
  -f ./Procfile \
  -p 7000 \
  -t 10 \
  -n
```

<details>

```bash
system    | exit_1.3  start at pid: 64568
system    | exit_1.2  start at pid: 64569
system    | exit_1.1  start at pid: 64570
system    | loop.1    start at pid: 64571
system    | loop.2    start at pid: 64572
loop.2    | Hello World
loop.1    | Hello World
exit_1.1  | failed
exit_1.3  | failed
exit_1.2  | failed
exit_1.1  | exited with code 1
system    | sending SIGTERM for exit_1.3  at pid 64568
system    | sending SIGTERM for exit_1.2  at pid 64569
system    | sending SIGTERM for loop.1    at pid 64571
system    | sending SIGTERM for loop.2    at pid 64572
exit_1.2  | exited with code 1
system    | sending SIGTERM for exit_1.3  at pid 64568
system    | sending SIGTERM for loop.1    at pid 64571
system    | sending SIGTERM for loop.2    at pid 64572
exit_1.3  | exited with code 1
system    | sending SIGTERM for loop.1    at pid 64571
system    | sending SIGTERM for loop.2    at pid 64572
loop.1    | terminated by SIGTERM
loop.2    | terminated by SIGTERM
```

</details>

### Full option example (long)

```bash
cargo run start \
  --formation all=2 \
  --env ./.env \
  --procfile ./Procfile \
  --port 7000 \
  --timeout 10 \
  --no-timestamp
```

<details>

```bash
system    | exit_1.1  start at pid: 65179
system    | exit_0.2  start at pid: 65180
system    | loop.2    start at pid: 65181
system    | exit_0.1  start at pid: 65182
system    | loop.1    start at pid: 65183
system    | exit_1.2  start at pid: 65184
loop.1    | Hello World
loop.2    | Hello World
exit_1.2  | failed
exit_1.1  | failed
exit_1.1  | exited with code 1
system    | sending SIGTERM for exit_0.2  at pid 65180
system    | sending SIGTERM for loop.2    at pid 65181
system    | sending SIGTERM for exit_0.1  at pid 65182
system    | sending SIGTERM for loop.1    at pid 65183
system    | sending SIGTERM for exit_1.2  at pid 65184
exit_1.2  | exited with code 1
system    | sending SIGTERM for exit_0.2  at pid 65180
system    | sending SIGTERM for loop.2    at pid 65181
system    | sending SIGTERM for exit_0.1  at pid 65182
system    | sending SIGTERM for loop.1    at pid 65183
exit_0.1  | terminated by SIGTERM
loop.1    | terminated by SIGTERM
loop.2    | terminated by SIGTERM
exit_0.2  | terminated by SIGTERM
```

</details>
