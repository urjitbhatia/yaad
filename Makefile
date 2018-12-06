test:
	cargo test

run-metrics-container: ## Runs a container running InfluxDB+Grafana and graphs Logviathan runtime metrics
	docker run -d --name docker-statsd-influxdb-grafana \
		-p 3003:3003 \
		-p 3004:8083 \
		-p 8086:8086 \
		-p 22022:22 \
		-p 8125:8125/udp \
		samuelebistoletti/docker-statsd-influxdb-grafana:latest
	@grafanasettings/setupgrafana.sh

stop-metrics-container:
	docker stop yaad-metrics

start-metrics-container:
	docker start docker-statsd-influxdb-grafana

run-demo:
	RUN_MODE=demo cargo run

go-dep:
	dep ensure

go-build: go-dep
	 GOOS=linux GOARCH=amd64 go build -o yaad .

