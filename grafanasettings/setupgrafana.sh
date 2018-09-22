#!/usr/bin/env bash
set -e

echo "       Waiting for Grafana Server to come up   "
bash -c 'while [[ "$(curl -s -o /dev/null -w ''%{http_code}'' http://root:root@localhost:3003/api/org)" != "200" ]]; do sleep 5; done'

echo "       Setting up Grafana Datasource           "
curl -H "Content-Type: application/json" -d "@grafanasettings/datasource.json" http://root:root@localhost:3003/api/datasources

echo " Grafana dashboard running at: http://root:root@localhost:8003  "
