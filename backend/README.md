```shell
cargo run -- 1 3001 2 3
cargo run -- 2 3002 1 3
cargo run -- 3 3003 1 2


curl http://localhost:3001/health 

curl -X POST http://localhost:3001/auth/init-admin
curl -X POST http://localhost:3001/kv/init-storage
curl -X POST http://localhost:3001/fee/init-fee

curl -X POST http://localhost:3001/kv/upsert -H "Content-Type: application/json" -d "{\"op\": \"upsert\", \"key\": \"name1\", \"value\": \"AeroKV1\"}"
curl -X POST http://localhost:3001/kv/upsert -H "Content-Type: application/json" -d "{\"op\": \"upsert\", \"key\": \"name2\", \"value\": \"AeroKV2\"}"

curl -X DELETE http://localhost:3002/kv/delete -H "Content-Type: application/json" -d "{\"op\": \"delete\", \"key\": \"name1\"}"



curl -X POST http://localhost:3001/auth/set-pause -H "Content-Type: application/json" -d "{\"op\": \"pause\", \"paused\": true}"
curl -X POST http://localhost:3001/auth/set-pause -H "Content-Type: application/json" -d "{\"op\": \"pause\", \"paused\": false}"

curl -X POST http://localhost:3001/fee/set-fee -H "Content-Type: application/json" -d "{\"op\": \"fee\", \"base_fee\": 100, \"fee_per_byte\": 100, \"scan_fee_per_item\": 100}"

curl http://localhost:3001/kv/get?key=name1

curl -X POST http://localhost:3001/kv/scan -H "Content-Type: application/json" -d "{\"key\": \"na\", \"value\": \"\", \"limit\": 10}"
curl -X POST http://localhost:3001/kv/page -H "Content-Type: application/json" -d "{\"page\": 1, \"limit\": 10}"
curl -X POST http://localhost:3001/kv/page -H "Content-Type: application/json" -d "{\"page\": 2, \"limit\": 1}"


```
