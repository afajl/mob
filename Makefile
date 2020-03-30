test-repos: clean
	mkdir testrepos
	git init testrepos/origin
	touch testrepos/origin/hello
	(cd testrepos/origin && git add -A . && git commit  -m Initial)

	git clone testrepos/origin testrepos/test

clean:
	rm -rf testrepos
