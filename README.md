To update sqlx definitions.

1) Use the docker compose in the example/server project to start a local environment
2) Install the example schema with
```cargo run --bin example-cli install-schema```

3) In the `example/server` directory run 
```cargo sqlx prepare```
