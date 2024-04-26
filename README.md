# TMT-Web

Rebuilding [tmt](https://github.com/swizzard/tmt) as a proper web app, with
users etc.

See [agenda](./agenda.md) to get a gist.

## Local Development & Testing

1. Start Postgres server, using [`docker-compose`](./docker-compose.yml) if you're into that kind of thing
2. Create dev and test databases
3. Move `.env.example` to `.env` and update `DATABASE_URL` and `TEST_DATABASE_URL`
4. Run `diesel migration run` to create tables in db identified by `DATABASE_URL`  
   (re-run with `--database-url $TEST_DATABASE_URL` to create tables in test db)
5. Run `cargo test` to run tests
