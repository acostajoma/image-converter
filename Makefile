.PHONY: build up down logs clean bench

APP_NAME = image-converter

build:
	docker build -t $(APP_NAME) .

up:
	docker compose up -d

logs:
	docker compose logs -f

down:
	docker compose down

clean:
	docker compose down
	docker system prune -f

bench:
	docker run --rm williamyeh/wrk \
		-t4 \
		-c20 \
		-d15s \
		-H "Accept: image/webp" \
		--timeout 2s \
		http://host.docker.internal:3000/images/test_image?width=300