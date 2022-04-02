# Ultraman check example

`ultraman check` checks if one or more processes are defined in the Procfile.  
It does not check the contents of the process.

|short|long|default|description|
|-----|----|-------|-----------|
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
cargo run check \
  -f Procfile
```

<details>

```bash
valid procfile detected (exit_0, exit_1, loop)
```

```bash
echo $?
0
```

</details>

### case Procfile do not exist

```bash
cargo run check \
  -f ./tmp/do_not_exist/Procfile
```

<details>

```bash
./tmp/do_not_exist/Procfile does not exist.
```

```bash
echo $?
1
```

</details>

## Full option example (long)

```bash
cargo run check \
  --procfile Procfile
```

```bash
valid procfile detected (exit_0, exit_1, loop)
```

```bash
echo $?
0
```
