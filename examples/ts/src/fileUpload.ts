import fs from "fs";
import { EXAMPLE_DATASET_ID, trieve } from "./trieve";

const main = async () => {
  const file = fs.readFileSync("./src/uploadme.pdf");

  console.log(file.toString("base64"));
  const fileEncoded = file
    .toString("base64")
    .replace(/\+/g, "-") // Convert '+' to '-'
    .replace(/\//g, "_") // Convert '/' to '_'
    .replace(/=+$/, ""); // Remove ending '='

  const response = await trieve.fetch("/api/file", "post", {
    data: {
      base64_file: fileEncoded,
      file_name: "uploadme.pdf",
      group_tracking_id: "file-upload-group",
    },
    datasetId: EXAMPLE_DATASET_ID,
  });

  console.log(response);
};

main();
