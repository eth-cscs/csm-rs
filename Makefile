.PHONY: convert-spec

# Convert Swagger 2.0 YAML to OpenAPI 3.0 JSON for progenitor consumption.
# Re-run whenever src/hsm/csm_api_docs.yaml changes. The JSON is committed.
convert-spec:
	npx --yes swagger2openapi src/hsm/csm_api_docs.yaml \
		-o src/hsm/csm_api_docs.openapi3.json
