import { createClient } from "@hey-api/openapi-ts";
import fs from "node:fs";

const main = async () => {
  await createClient({
    client: {
      name: "legacy/fetch",
    },
    types: true,
    schemas: false,
    services: false,
    input: "./openapi.json",
    output: "./client/generated",
  });

  // Move ./client/generated/types.gen.ts to ./client/src/types.gen.ts
  console.log("Client generated successfully!");
  fs.renameSync("./client/generated/types.gen.ts", "./src/types.gen.ts");

  // Delete the generated folder
  fs.rmSync("./client", { recursive: true });
};

void main();
