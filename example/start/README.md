# Ultraman start example

If no additional parameters are passed, `ultraman` will run one instance of each type of process defined in your `Procfile`.  

The following options control how the application is run:

|short|long|default|description|
|-----|----|-------|-----------|
|<kbd>-m</kbd>|<kbd>--formation</kbd>|`all=1`|Specify the number of each process type to run. The value passed in should be in the format process=num,process=num|
|<kbd>-e</kbd>|<kbd>--env</kbd>|`.env`|Specify an environment file to load|
|<kbd>-f</kbd>|<kbd>--procfile</kbd>|`Procfile`|Specify an alternate Procfile to load, implies -d at the Procfile root|
|<kbd>-p</kbd>|<kbd>--port</kbd>||Specify which port to use as the base for this application. Should be a multiple of 1000|

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
  -p 7000
```

<details>

```
13:04:47 system    | exit_1.2  start at pid: 14651
13:04:48 system    | loop.2    start at pid: 14655
13:04:47 system    | exit_1.3  start at pid: 14652
13:04:47 system    | loop.1    start at pid: 14653
13:04:48 system    | exit_1.1  start at pid: 14654
13:04:49 loop.2    | Hello World
13:04:49 loop.1    | Hello World
13:04:50 exit_1.3  | failed
13:04:50 exit_1.1  | failed
13:04:50 exit_1.2  | failed
13:04:50 exit_1.1  | exited with code 1
13:04:50 system    | sending SIGTERM for exit_1.2  at pid 14651
13:04:50 system    | sending SIGTERM for loop.2    at pid 14655
13:04:50 system    | sending SIGTERM for exit_1.3  at pid 14652
13:04:50 system    | sending SIGTERM for loop.1    at pid 14653
13:04:50 exit_1.3  | exited with code 1
13:04:50 system    | sending SIGTERM for exit_1.2  at pid 14651
13:04:50 system    | sending SIGTERM for loop.2    at pid 14655
13:04:50 system    | sending SIGTERM for loop.1    at pid 14653
13:04:50 exit_1.2  | exited with code 1
13:04:50 system    | sending SIGTERM for loop.2    at pid 14655
13:04:50 system    | sending SIGTERM for loop.1    at pid 14653
13:04:50 loop.2    | terminated by SIGTERM
13:04:50 loop.1    | terminated by SIGTERM
```

</details>

### Full option example (long)

```bash
cargo run start \
  --formation all=2 \
  --env ./.env \
  --procfile ./Procfile \
  --port 7000
```

<details>

```
13:05:59 system    | exit_1.1  start at pid: 14847
13:05:59 system    | loop.1    start at pid: 14848
13:05:59 system    | loop.2    start at pid: 14849
13:05:59 system    | exit_0.2  start at pid: 14850
13:05:59 system    | exit_0.1  start at pid: 14851
13:05:59 system    | exit_1.2  start at pid: 14852
13:06:00 loop.1    | Hello World
13:06:00 loop.2    | Hello World
13:06:01 loop.2    | Hello World
13:06:01 exit_1.1  | failed
13:06:01 loop.1    | Hello World
13:06:01 exit_1.2  | failed
13:06:01 exit_1.1  | exited with code 1
13:06:01 system    | sending SIGTERM for loop.1    at pid 14848
13:06:01 system    | sending SIGTERM for loop.2    at pid 14849
13:06:01 system    | sending SIGTERM for exit_0.2  at pid 14850
13:06:01 system    | sending SIGTERM for exit_0.1  at pid 14851
13:06:01 system    | sending SIGTERM for exit_1.2  at pid 14852
13:06:01 exit_1.2  | exited with code 1
13:06:01 system    | sending SIGTERM for loop.1    at pid 14848
13:06:01 system    | sending SIGTERM for loop.2    at pid 14849
13:06:01 system    | sending SIGTERM for exit_0.2  at pid 14850
13:06:01 system    | sending SIGTERM for exit_0.1  at pid 14851
13:06:01 loop.2    | terminated by SIGTERM
13:06:01 exit_0.2  | terminated by SIGTERM
13:06:01 exit_0.1  | terminated by SIGTERM
13:06:01 loop.1    | terminated by SIGTERM
```

</details>
