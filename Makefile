SHELL   = /bin/bash
DATADIR = $(CURDIR)/data
DBS     = $(addprefix $(DATADIR)/,superheroes.db companies.db)

all:
	@

install: dbs

dbs: $(DBS)
$(DBS):
	. $(CURDIR)/script/dbs.sh

.PHONY: cmd-%
cmd-%:
	cargo run data/sample.db .$(subst cmd-,,$@)
