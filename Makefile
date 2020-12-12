.PHONY: create-man
create_man:
					cargo run --bin man > ./tmp/ultraman.1;

.PHONY: man
man: create_man
					man ./tmp/ultraman.1;

.PHONY: install_man
install_man: create_man
					install -Dm644 ./tmp/ultraman.1 /usr/local/share/man/man1/ultraman.1;
