.PHONY: build

MEMCACHED_HOST=127.0.0.1
MEMCACHED_PORT=11241
ITER=100
ATOMIC=atomic
NONATOMIC=nonatomic

APP_IMAGE_NAME=get-set-data-race-rust

MEMCACHED_IMAGE_NAME=mc-get-set-data-race-rust

up:
	docker run -d --rm -p $(MEMCACHED_PORT):11211/tcp --name $(MEMCACHED_IMAGE_NAME) memcached

down:
	docker stop $(MEMCACHED_IMAGE_NAME)

build:
	docker build -f build/package/Dockerfile -t $(APP_IMAGE_NAME) .

run:
	docker run -it --rm --network=host --name my-app $(APP_IMAGE_NAME) --host $(MEMCACHED_HOST) --port $(MEMCACHED_PORT) --key mykey --iter $(ITER) --method $(NONATOMIC)
	docker run -it --rm --network=host --name my-app $(APP_IMAGE_NAME) --host $(MEMCACHED_HOST) --port $(MEMCACHED_PORT) --key mykey --iter $(ITER) --method $(ATOMIC)
