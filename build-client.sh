cargo run --features runtime-env --manifest-path server/Cargo.toml --bin redoc_ci > ./frontends/shared/client/openapi.json

cd ./frontends/shared/; yarn; yarn genclient;
