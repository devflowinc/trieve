# Changelog

## [0.12.0](https://github.com/devflowinc/trieve/compare/v0.11.8...v0.12.0) (2024-09-18)


### Features

* add copy button to chat ([c35406d](https://github.com/devflowinc/trieve/commit/c35406d4bcb54fb94615e951a867e5456bb75c21))
* add created at sort in rag ([7ad2d4d](https://github.com/devflowinc/trieve/commit/7ad2d4d5d68d3de960d6861eab19e30cb11c6de1))
* add firecrawl to our docker compose ([97deaa5](https://github.com/devflowinc/trieve/commit/97deaa5212b2dbf81353c0f36df86a9e70509d8d))
* fill in missing dates on graphs ([2eca588](https://github.com/devflowinc/trieve/commit/2eca5885b0ebf6866d9bfd377688375df82c6044))
* fix typo issues ([590d808](https://github.com/devflowinc/trieve/commit/590d808b6e3c8262dbe42214bda8f414b38fbe2f))
* new data explorer table ([c350a5a](https://github.com/devflowinc/trieve/commit/c350a5ac2c3397c6d94584f09c5f01dad7218d8f))
* new tables for rag queries ([48112ee](https://github.com/devflowinc/trieve/commit/48112ee5d8d56f6e4d0e4d6e6635a0b0965dec93))
* python client ([5e864ee](https://github.com/devflowinc/trieve/commit/5e864ee3e0209ae2bb97fd1210c10705d443c3a0))
* redo table in no search query ([dad2340](https://github.com/devflowinc/trieve/commit/dad2340f16a03482e40733710917ba1becd6349c))
* redo tables in search analytics ([bbd7255](https://github.com/devflowinc/trieve/commit/bbd7255931d400b056effc73a97bb2e2f346ae19))
* update modal in data explorer ([7c207a9](https://github.com/devflowinc/trieve/commit/7c207a996a549520675b055bc9640f246ec1cf98))


### Cleanup

* better CSS scoping for search component ([87e847a](https://github.com/devflowinc/trieve/commit/87e847aa6039fdcc0bb65eab1938892d3290fe62))
* change default prompt such that LLM does not provide citations by default ([6878f85](https://github.com/devflowinc/trieve/commit/6878f85e52d32ca6a52b8d1aa70257bef1d5b9ad))
* fix 2x typos of 'likeclickthrough' ([4041788](https://github.com/devflowinc/trieve/commit/40417885182171a61a005c84e69ff9240a572ef6))
* improve pypi page for python sdk client ([cd20b77](https://github.com/devflowinc/trieve/commit/cd20b779bf74e5d8e82d878600217b8b8e936768))
* properly handle fallback types for redis for Rust '24 version ([6e14d18](https://github.com/devflowinc/trieve/commit/6e14d18321713611b83fdab9f0000a96439b9c36))
* remove tooltip and globe icons ([5831b53](https://github.com/devflowinc/trieve/commit/5831b53fad74a495c21d37512c50f113156b60eb))


### Bug Fixes

* add better margins to headings ([868bc7c](https://github.com/devflowinc/trieve/commit/868bc7c7ab46a06f188e8d8858f248be1afba2e3))
* add get invitations to openapi spec ([5f33336](https://github.com/devflowinc/trieve/commit/5f33336f992cc8287763ebba88d92e54e579a778))
* assorted fixes on component from signoz feedback ([122801a](https://github.com/devflowinc/trieve/commit/122801aff394a9b06444a5ecd56fa6b62caf2c15))
* css fixes for rag page ([40bd6ea](https://github.com/devflowinc/trieve/commit/40bd6eaa1538405c05d1e7e6f770b00ab2268b63))
* dataset warning for chat ([82b8f21](https://github.com/devflowinc/trieve/commit/82b8f21e7305b14077a81f0f37a7716ac00d7222))
* fill empty dates in rag usage chart ([2537a6d](https://github.com/devflowinc/trieve/commit/2537a6d161485c8c15303e50aa047f5d618c615e))
* fix copy feature to allow for plain text in chat ([f4f2a2c](https://github.com/devflowinc/trieve/commit/f4f2a2c128aec9d4709056e18eb32f972e2e2de9))
* improve start services and add tmuxp to README local Dev guide ([2473c0a](https://github.com/devflowinc/trieve/commit/2473c0a311a4c1f31a86d5852b600fe72912d802))
* incorrect schema type for getting an org ([4061254](https://github.com/devflowinc/trieve/commit/4061254b31865a3b868e69744eb9e22d7ecc7698))
* move table code to the top of component ([6631ee7](https://github.com/devflowinc/trieve/commit/6631ee785dc142a0713f9b2d8261fee1e6a45975))
* no datasets warning for chat ([04a7775](https://github.com/devflowinc/trieve/commit/04a7775615fc6fa3ac9802bdd7e0ec715e471ceb))
* only open modal when there are results ([d93b9ef](https://github.com/devflowinc/trieve/commit/d93b9ef5da490b1549ed559bb69d0915b79a171e))
* set topic list to empty if dataset is invalid ([b958ed6](https://github.com/devflowinc/trieve/commit/b958ed6bda112af8f89394be746e1d36c7175c6d))
* show all info in the no results table ([36c948e](https://github.com/devflowinc/trieve/commit/36c948e23ea15645a1f00990ddc34644d05a00d4))
* use cn helper && add classname prop to table ([8d78f6b](https://github.com/devflowinc/trieve/commit/8d78f6b6cb999226e8cb0c36a5989bc323912072))


### Other

* fix eslint ([b8fd916](https://github.com/devflowinc/trieve/commit/b8fd916cab27045a2058753cfb8bee0324c32b7e))
* fix eslint ([d44299f](https://github.com/devflowinc/trieve/commit/d44299f29c23cc8a0bba8a5467b7cdb302b70c67))
* regenarate lock ([404bc9d](https://github.com/devflowinc/trieve/commit/404bc9da92f55831e9a3c3cd4ba589fc89fb67c1))
* ts fix ([f2ecdce](https://github.com/devflowinc/trieve/commit/f2ecdcebac8f2881bd4d6c34137e8e97821fb4bc))
* use memo ([6ab259b](https://github.com/devflowinc/trieve/commit/6ab259bdb40bb18614fe958a5a88e011548571d2))


### Docs

* fix getting started README docs ([6410745](https://github.com/devflowinc/trieve/commit/64107452d4e119c77fdb2ac0e6f3b2cb8c1c7585))
