all: npm build

npm:
	npm install

build: webpack copy-assets

webpack:
	webpack

copy-assets:
	cp src/index.html ../public/index.html

watch: copy-assets
	webpack -w


