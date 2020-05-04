install:
	cargo build --release
	cp target/release/mob /usr/local/bin

pull:
	git pull

update: pull install

uml:
	plantuml -tsvg state.uml
.PHONY: uml

test-repos:
	mkdir testrepos
	git init --bare testrepos/origin
	git clone testrepos/origin testrepos/first
	git clone testrepos/origin testrepos/second
	touch testrepos/first/hello
	(cd testrepos/first && git add -A . && git commit  -m Initial && git push)

clean-repos:
	rm -rf testrepos

