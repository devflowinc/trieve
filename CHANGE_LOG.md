### Server

- allow for file upload without creating cards
- allow for a collection_id to be passed when creating cards such that a bookmark can be created
- add OpenAPI docs through redoc display with actix
- card_collection route now returns 200 with CardCollection instead of 204
- support multiple DATASET logic on the backend only on the card level

### Search

### Chat

### Docker

- version tag Redis containers to redis:7.2.2
- Optimized Dockerfiles to compile faster
