.PHONY: dev daemon web

dev:
	@trap 'kill 0' INT TERM; \
	cargo run --bin pi-router & \
	(cd web && npm run dev) & \
	wait

daemon:
	cargo run --bin pi-router

web:
	cd web && npm run dev
