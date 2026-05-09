SHELL := /bin/sh

.PHONY: help up down status test build

RUNTIME_DIR := .local/dev-runtime
LOG_DIR := $(RUNTIME_DIR)/logs
PID_DIR := $(RUNTIME_DIR)/pids

SERVICES := trading-api market-data-api wallet-worker risk-worker liquidation-worker

help:
	@echo "Available targets:"
	@echo "  make up      - build workspace and start all services"
	@echo "  make down    - stop all services"
	@echo "  make status  - show services status"
	@echo "  make test    - run workspace tests"
	@echo "  make build   - build workspace"

build:
	cargo build --workspace

test:
	cargo test --workspace

up:
	@mkdir -p "$(LOG_DIR)" "$(PID_DIR)"
	@for s in $(SERVICES); do \
		if [ -f "$(PID_DIR)/$$s.pid" ] && kill -0 "$$(cat "$(PID_DIR)/$$s.pid")" 2>/dev/null; then \
			echo "$$s is already running"; \
		else \
			echo "starting $$s..."; \
			nohup cargo run -p "$$s" >"$(LOG_DIR)/$$s.out.log" 2>"$(LOG_DIR)/$$s.err.log" & \
			echo $$! >"$(PID_DIR)/$$s.pid"; \
		fi; \
	done
	@echo "all services processed"

down:
	@for s in $(SERVICES); do \
		if [ -f "$(PID_DIR)/$$s.pid" ]; then \
			pid="$$(cat "$(PID_DIR)/$$s.pid")"; \
			if kill -0 "$$pid" 2>/dev/null; then \
				echo "stopping $$s (pid=$$pid)"; \
				kill "$$pid" 2>/dev/null || true; \
			else \
				echo "$$s already stopped"; \
			fi; \
			rm -f "$(PID_DIR)/$$s.pid"; \
		else \
			echo "$$s pid file not found"; \
		fi; \
	done
	@echo "all services processed"

status:
	@for s in $(SERVICES); do \
		if [ -f "$(PID_DIR)/$$s.pid" ]; then \
			pid="$$(cat "$(PID_DIR)/$$s.pid")"; \
			if kill -0 "$$pid" 2>/dev/null; then \
				echo "$$s: running (pid=$$pid)"; \
			else \
				echo "$$s: stopped (stale pid=$$pid)"; \
			fi; \
		else \
			echo "$$s: stopped"; \
		fi; \
	done
