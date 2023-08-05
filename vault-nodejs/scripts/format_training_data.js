import fs from "fs";
import path from "path";

function readJsonFromFile(filePath) {
  try {
    const rawData = fs.readFileSync(filePath);
    const jsonData = JSON.parse(rawData);
    return jsonData;
  } catch (error) {
    console.error("Error reading JSON from file:", error.message);
    return null;
  }
}

const filePath = "./training_data.json";
const jsonObject = readJsonFromFile(filePath);
let data = [];

for (let i = 0; i < jsonObject.length; i++) {
  const curContent = jsonObject[i].content;
  const curCardHTML = jsonObject[i].card_html;

  const prompt = curContent + "\n\n###\n\n";
  const completion = " " + curCardHTML + "###";

  if (prompt.length + completion.length > 11000) {
    continue;
  }

  data.push({
    prompt: prompt,
    completion: completion,
  });
}

const newFilePath = path.join(".", "clean_training_data.json"); // Adjust the file path as needed
const jsonData = JSON.stringify(data);

fs.writeFile(newFilePath, jsonData, (err) => {
  if (err) {
    console.log(data);
  } else {
    console.log("Data has been written to data.json");
  }
});

// openai tools fine_tunes.prepare_data -f ./clean_training_data.json
// openai api fine_tunes.create -t "./clean_training_data_prepared.jsonl" -m curie