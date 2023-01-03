## Progress

- [x] commands can be supplied
- [x] `--names` can be passed
- [x] `--name-separator`
- [ ] `--raw`
- [x] `--no-color`
- [ ] `--hide`
- [ ] `--timings`
- [ ] `--passthrough-arguments`
- [x] `--prefix`
  - [x] index
  - [x] pid
  - [x] time
  - [x] command
  - [x] name
- [x] `--prefix-colors`
- [x] `--prefix-length`
- [x] `--timestamp-format`
- [ ] `--kill-others`
- [ ] `--kill-others-on-fail`
- [ ] `--restart-tries`
- [ ] `--restart-after`

## Examples

```
cargo run -- --names "server,client" \
    "nc -lk 1234" \
    "echo 'foo' | nc localhost 1234"
```
