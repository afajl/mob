# list recipies
default:
  @just --list

[group("test")]
test:
  cargo test --all-features -- --include-ignored

[group("test")]
fmt:
  cargo fmt --all

[group("test")]
clippy:
  cargo clippy --all-targets --all-features

[group("test")]
check: fmt clippy test

uml:
	cd assets && plantuml -tsvg state.uml



