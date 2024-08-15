import fs from "fs";
import { EXAMPLE_DATASET_ID, trieve } from "./trieve";

const main = async () => {
  const file = fs.readFileSync("./src/uploadme.txt");

  const fileString = file.toString("base64");

  console.log(fileString);

  const response = await trieve.fetch("/api/file", "post", {
    data: {
      base64_file: fileString,
      file_name: "uploadmeagain.txt",
      group_tracking_id: "file-upload-group",
    },
    datasetId: EXAMPLE_DATASET_ID,
  });

  console.log(response);
};

main();
