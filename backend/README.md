```shell
curl -X POST http://127.0.0.1:8001/raft/init -H "Content-Type: application/json" -d '{
    "1": { "addr": "127.0.0.1:8001" },
    "2": { "addr": "127.0.0.1:8002" },
    "3": { "addr": "127.0.0.1:8003" }
}'

curl http://localhost/health

curl -X POST http://localhost:8001/auth/init-admin
curl -X POST http://localhost:8001/kv/init-counter
curl -X POST http://localhost:8002/kv/init-storage
curl -X POST http://localhost:8003/fee/init-fee

curl -X POST http://localhost:8001/kv/upsert -H "Content-Type: application/json" -d "{\"key\": \"name1\", \"value\": \"AeroKV1\", \"limit\": 0}"
curl -X POST http://localhost:8001/kv/upsert -H "Content-Type: application/json" -d "{\"key\": \"name2\", \"value\": \"AeroKV2\", \"limit\": 0}"

curl -X DELETE http://localhost:8001/kv/delete?key=name1



curl -X POST http://localhost:8001/auth/set-pause -H "Content-Type: application/json" -d "{\"paused\": true}"
curl -X POST http://localhost:8001/auth/set-pause -H "Content-Type: application/json" -d "{\"paused\": false}"

curl -X POST http://localhost:8001/fee/set-fee -H "Content-Type: application/json" -d "{\"op\": \"fee\", \"base_fee\": 100, \"fee_per_byte\": 100, \"scan_fee_per_item\": 100}"

curl http://localhost:8001/kv/get?key=name1
curl http://localhost:8003/kv/get?key=name1
curl http://localhost:8002/kv/get?key=name2

curl -X POST http://localhost:8001/kv/scan -H "Content-Type: application/json" -d "{\"key\": \"na\", \"value\": \"\", \"limit\": 10}"
curl -X POST http://localhost:8001/kv/page -H "Content-Type: application/json" -d "{\"page\": 1, \"limit\": 10}"
curl -X POST http://localhost:8002/kv/page -H "Content-Type: application/json" -d "{\"page\": 1, \"limit\": 10}"
curl -X POST http://localhost:8003/kv/page -H "Content-Type: application/json" -d "{\"page\": 1, \"limit\": 10}"
curl -X POST http://localhost:8003/kv/page -H "Content-Type: application/json" -d "{\"page\": 2, \"limit\": 1}"


```
