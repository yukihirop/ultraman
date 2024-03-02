# Development

## Example

```bash
cd example/start
cargo run start -p ./Procfile
```

## Release

```bash
cargo build
cargo bump
cargo publish --dry-run
```
