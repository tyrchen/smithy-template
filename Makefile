ASSETS = assets.tar.gz

validate:
	@cd smithy && smithy validate

update-smithy:
	@gh release download -R tyrchen/smithy-assets -p '$(ASSETS)'
	@rm -rf $HOME/.m2
	@tar -xzf $(ASSETS) -C $(HOME) --strip-components=2
	@rm $(ASSETS)

build-smithy:
	@cd smithy && smithy build

watch:
	@watchexec --restart --exts rs -- cargo run --bin echo-server

client:
	@cargo run --bin echo-client
