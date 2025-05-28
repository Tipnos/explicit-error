test:
    cargo test -p explicit-error 
    cargo test -p explicit-error-exit 
    cargo test -p explicit-error-http --lib
    cargo test -p explicit-error-http --lib --features actix-web
    cargo test -p explicit-error-http --doc --features actix-web
    cargo build --example actix
