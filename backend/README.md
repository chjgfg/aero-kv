```shell


curl http://localhost/health

curl -X POST http://localhost/auth/init-admin
curl -X POST http://localhost/kv/init-storage
curl -X POST http://localhost/fee/init-fee

curl -X POST http://localhost/kv/upsert -H "Content-Type: application/json" -d "{\"key\": \"name1\", \"value\": \"AeroKV1\", \"limit\": 0}"
curl -X POST http://localhost/kv/upsert -H "Content-Type: application/json" -d "{\"key\": \"name2\", \"value\": \"AeroKV2\", \"limit\": 0}"

curl -X DELETE http://localhost/kv/delete -H "Content-Type: application/json" -d "{\"op\": \"delete\", \"key\": \"name1\"}"



curl -X POST http://localhost/auth/set-pause -H "Content-Type: application/json" -d "{\"paused\": true}"
curl -X POST http://localhost/auth/set-pause -H "Content-Type: application/json" -d "{\"paused\": false}"

curl -X POST http://localhost/fee/set-fee -H "Content-Type: application/json" -d "{\"op\": \"fee\", \"base_fee\": 100, \"fee_per_byte\": 100, \"scan_fee_per_item\": 100}"

curl http://localhost/kv/get?key=name1
curl http://localhost/kv/get?key=name2

curl -X POST http://localhost/kv/scan -H "Content-Type: application/json" -d "{\"key\": \"na\", \"value\": \"\", \"limit\": 10}"
curl -X POST http://localhost/kv/page -H "Content-Type: application/json" -d "{\"page\": 1, \"limit\": 10}"
curl -X POST http://localhost/kv/page -H "Content-Type: application/json" -d "{\"page\": 2, \"limit\": 1}"


```
