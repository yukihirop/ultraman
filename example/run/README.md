# Ultraman run example

`ultraman run` is used to run your application directly from the command line.  
If no additional parameters are passed, `ultraman` will run one instance of each type of process defined in your `Procfile`.  
If a parameter is passed, `ultraman` will run one instance of the specified application type.  

The following options control how the application is run:

|short|long|default|description|
|-----|----|-------|-----------|
|<kbd>-e</kbd>|<kbd>--env</kbd>|`.env`|Specify an environment file to load|
|<kbd>-f</kbd>|<kbd>--procfile</kbd>|`Procfile`|Specify an alternate Procfile to load|


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

## Full option example (short)

```bash
cargo run run loop \
  -e .env \
  -f Procfile
```

<details>

```bash
Hello World
Hello World
Hello World
Hello World
Hello World
Hello World
Hello World
Hello World
^C%
```

</details>

## Full option example (long)

```bash
cargo run run exit_0 \
  --env .env \
  --procfile Procfile
```

```bash
success
```
