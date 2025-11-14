# Emberleaf — Local QA Helpers

.PHONY: qa-kws-e2e-loopback
qa-kws-e2e-loopback:
	@echo "Starting PipeWire/WirePlumber (local) and running harness with loopback…"
	@dbus-run-session -- bash -c '\
		pipewire & wireplumber & sleep 2; \
		pactl load-module module-null-sink sink_name=qa_sink sink_properties=device.description=QA_SINK || true; \
		pactl load-module module-loopback sink=qa_sink latency_msec=1 || true; \
		xvfb-run -a node scripts/qa/run-kws-e2e.mjs \
	'
