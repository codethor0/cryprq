.PHONY: test build-linux build-mac build-win clean install

test:
	docker compose -f docker-compose.yml up --build --abort-on-container-exit --exit-code-from tests

build-linux:
	npm run build:linux

build-mac:
	npm run build:mac

build-win:
	npm run build:win-unsigned

clean:
	rm -rf dist dist-electron dist-package node_modules/.vite

install:
	npm ci
