.PHONY: create-man
create-man:
					cargo run --bin man > ./tmp/ultraman.1;

.PHONY: man
man: create-man
					man ./tmp/ultraman.1;

.PHONY: install-man
install-man: create-man
					sudo install -Dm644 ./tmp/ultraman.1 /usr/share/man/man1/ultraman.1;
