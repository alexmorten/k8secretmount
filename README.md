# k8secretmount

Small utility cli to watch a kubernetes secret and write it into a local directory.

Can be used to emulate mounting a secret into a pod in development.

## Usage

```bash
k8secretmount <secret-name> <mount path> [<namespace>]
```

## Build

```bash
cargo build --release
```

then copy `target/release/k8secretmount` into your `$PATH`
