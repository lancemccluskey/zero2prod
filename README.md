# zero2prod

### Pre-commit checks

```bash
cargo fmt --all -- --check
cargo clippy -- -D warnings
cargo test
```

### Migrations

For prod migrations, run `DATABASE_URL="" sqlx migrate run`
