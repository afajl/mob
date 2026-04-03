install:
	cargo build --release
	cp target/release/mob /usr/local/bin

pull:
	git checkout master
	git pull --ff-only

update: pull install

uml:
	cd assets && plantuml -tsvg state.uml
.PHONY: uml
