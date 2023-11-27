### Server

- allow for file upload without creating cards
- allow for a collection_id to be passed when creating cards such that a bookmark can be created
- add OpenAPI docs through redoc display with actix
- card_collection route now returns 200 with CardCollection instead of 204
- Addded EMBEDDING_SEMAPHORE_SIZE to limit the amount of threads that can make a request to create an embedding

### Search

- styling: slightly improved filters modal UI with distinguished button colors
- bugfix: show more only appears when needed for CardMetadataDisplay.tsx component
- bugfix: make ChatPopup choose from only selected ids instead of global set for chat docs 

### Chat

- feature: make link to search open in new tab
- bugfix: always show all cards instead of a single one

### Docker

- version tag Redis containers to redis:7.2.2
- Optimized Dockerfiles to compile faster
