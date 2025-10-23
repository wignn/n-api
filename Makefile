dev:
	docker-compose up -d

dev-down:
	docker-compose down

migrate-up:
	sqlx migrate run

migrate-down:
	sqlx migrate revert

start-server:
	cargo watch -q -c -w src/ -x run

start-load-test:
	k6 run --out influxdb=http://localhost:8086/k6 load-test.js