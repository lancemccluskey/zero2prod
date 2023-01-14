# zero2prod

## Running Locally

```bash
./scripts/init_db.sh
cargo run
```

### Pre-commit checks

```bash
cargo fmt --all -- --check
cargo clippy -- -D warnings
cargo test
```

### Migrations

For prod migrations, run `DATABASE_URL="" sqlx migrate run`

### Applying new Digital Ocean spec

```bash
doctl apps update <APP_ID> --spec=spec.yaml
```
