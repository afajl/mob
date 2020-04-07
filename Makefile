test-repos:
	mkdir testrepos
	git init --bare testrepos/origin
	git clone testrepos/origin testrepos/first
	git clone testrepos/origin testrepos/second
	touch testrepos/first/hello
	(cd testrepos/first && git add -A . && git commit  -m Initial && git push)

clean-repos:
	rm -rf testrepos
