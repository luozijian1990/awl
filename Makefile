.PHONY: fmt test build smoke clean-smoke

SMOKE_DB := target/worklog-smoke.db
WORKLOG := target/debug/worklog

fmt:
	cargo fmt --check

test:
	cargo test --workspace

build:
	cargo build --workspace

smoke: build clean-smoke
	$(WORKLOG) --db $(SMOKE_DB) entry add --title "Make smoke draft" --start 2026-06-16T09:00:00Z --end 2026-06-16T10:00:00Z --actor ai --source codex
	$(WORKLOG) --db $(SMOKE_DB) entry list --status draft --format json
	$(WORKLOG) --db $(SMOKE_DB) entry confirm 1
	$(WORKLOG) --db $(SMOKE_DB) export report-source --start 2026-06-16T00:00:00Z --end 2026-06-17T00:00:00Z --format json
	$(WORKLOG) --db $(SMOKE_DB) export report-source --start 2026-06-16T00:00:00Z --end 2026-06-17T00:00:00Z --format markdown

clean-smoke:
	rm -f $(SMOKE_DB) $(SMOKE_DB)-wal $(SMOKE_DB)-shm
