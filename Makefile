allfmt:
	make format
	make lint

lint:
	cd client/ && cargo clippy --all-targets --all-features -- -D warnings
	cd server/ && scalafmt --check .

format:
	cd client/ && cargo fmt --all -- --check
	cd server/ && scalafmt .

build:
	cd client/ && cargo build
	cd server/ && sbt -v +test

dev:
	docker-compose up -d

destroy:
	docker-compose down

purge:
	sudo rm -rf .docker/data

clean:
	make destroy && make purge