# Mortar workspace targets: test-one, test-group, check, test and accept. A make file named in MAKEFILES may add
# its own targets beside them.
CARGO ?= cargo
.PHONY: test-one test-group check test accept

# Three kinds of test: GROUP=unit (default), integration or validation; scripts/test_one.sh says what each runs.
GROUP ?= unit
test-one:
	@CARGO="$(CARGO)" CARGO_TEST_ARGS="$(CARGO_TEST_ARGS)" sh "$(CURDIR)/scripts/test_one.sh" "$(DIR)" "$(GROUP)"

test-group:
	@rc=0; for d in crates/*/; do [ -f "$$d/Cargo.toml" ] || continue; \
	  CARGO="$(CARGO)" sh "$(CURDIR)/scripts/test_one.sh" "$${d%/}" "$(GROUP)" || rc=1; \
	done; exit $$rc

check:
	$(CARGO) check --workspace --all-targets

test:
	@$(MAKE) --no-print-directory test-group GROUP=$(GROUP)

# The acceptance drive, on throwaway repositories: builds every binary of the cli package (mortar and mr), runs
# the cli package's ignored tests in the test target accept (crates/cli/tests/accept) one at a time, and prints the
# report found at $(ACCEPT_REPORT). ACCEPT_AGENT, ACCEPT_MOOT_COPY and ACCEPT_TEAMS pass through from the environment
# for the drive to read; a step whose variable is unset is reported as not run. ACCEPT_TEAMS is the folder of team
# manifests the disk step reads the registered repositories from: no default, and with ACCEPT_MOOT_COPY set and
# ACCEPT_TEAMS unset the disk step fails. Exits with cargo test's status.
ACCEPT_REPORT ?= $(CURDIR)/target/accept/report.txt
accept:
	$(CARGO) build -p cli --bins
	@mkdir -p "$(dir $(ACCEPT_REPORT))"; rm -f "$(ACCEPT_REPORT)"
	@rc=0; ACCEPT_REPORT="$(ACCEPT_REPORT)" $(CARGO) test -p cli --test accept -- --ignored --test-threads=1 || rc=$$?; \
	  if [ -f "$(ACCEPT_REPORT)" ]; then cat "$(ACCEPT_REPORT)"; else echo "accept: no report at $(ACCEPT_REPORT)"; fi; \
	  exit $$rc

ifeq ($(GROUP),unit)
ZONE_TEST = $(CARGO) test --workspace --lib -- --skip integration::
else
ZONE_TEST = $(MAKE) --no-print-directory test-group GROUP=$(GROUP)
endif
