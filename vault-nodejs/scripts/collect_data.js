import fetch from "node-fetch";
import fs from "fs";
import path from "path";

const queries = [
  "Analyzing the impact of economic sanctions as a tool of foreign policy.",
  "The role of diplomacy in resolving international conflicts: Case studies and lessons learned.",
  "Assessing the effectiveness of humanitarian interventions in promoting global stability.",
  'Exploring the concept of "soft power" and its significance in international relations.',
  "The principle of non-intervention in sovereign states: Balancing humanitarian concerns and state sovereignty.",
  "The implications of preemptive military strikes on international security and stability.",
  "Examining the role of international organizations in shaping modern foreign policy.",
  "Nationalism vs. globalism: How do these ideologies influence foreign policy decisions?",
  "The ethics of arms trade: Navigating profits, security interests, and human rights.",
  "Analyzing the impact of cultural diplomacy in strengthening international relationships.",
  "The doctrine of containment: Historical analysis and its relevance in today's world.",
  "Human rights as a cornerstone of foreign policy: Challenges and contradictions.",
  "Nuclear disarmament and non-proliferation efforts: Progress, setbacks, and future prospects.",
  "The role of intelligence agencies in shaping foreign policy strategies.",
  "Exploring economic alliances and trade agreements as instruments of foreign policy.",
  "Environmental diplomacy: Addressing global challenges through international cooperation.",
  "The concept of responsibility to protect (R2P): When and how should states intervene in humanitarian crises?",
  "Cybersecurity and international relations: Navigating digital threats and norms.",
  "The impact of public opinion and media on foreign policy decision-making.",
  "Religious diplomacy: Understanding the role of faith in shaping international relations.",
];
let data = [];

const getTrainingData = async (singleQuery, page) => {
  try {
    const cardSearchRequest = await fetch(
      `https://api.arguflow.ai/api/card/search/${page}`,
      {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        credentials: "include",
        body: JSON.stringify({
          content: singleQuery,
          filter_link_url: [],
          filter_oc_file_path: [],
        }),
      }
    );

    if (!cardSearchRequest.ok) {
      const respText = await cardSearchRequest.text();
      throw new Error(respText);
    }

    const cardSearchResponse = await cardSearchRequest.json();

    for (let i = 0; i < cardSearchResponse.score_cards.length; i++) {
      const curMetadatas = cardSearchResponse.score_cards[i].metadata;
      let closestContent = '';
      let closestCardHTML = '';

      let closestCardLengthDiff = Infinity;

      for (let j = 0; j < curMetadatas.length; j++) {
        const cardHTML = curMetadatas[j].card_html;
        const cardLength = cardHTML.length;
        const lengthDiff = Math.abs(cardLength - 4300);

        if (cardLength <= 4300 && lengthDiff < closestCardLengthDiff) {
          closestCardHTML = cardHTML;
          closestContent = curMetadatas[j].content;
          closestCardLengthDiff = lengthDiff;
        }
      }

      if (closestContent < 3500 || closestContent.length > 4300) {
        continue;
      }

      data.push({
        content: closestContent,
        card_html: closestCardHTML,
      });
      console.log("Pushed: ", data.length);
    }
  } catch (err) {
    console.log(err);
    return;
  }
};

const MAX_CONCURRENT_REQUESTS = 1;

const getTrainingDataForAllQueries = async () => {
  const requestQueue = [];
  const pages = [1, 2, 3, 4, 5, 6];

  for (let p = 0; p < pages.length; p++) {
    for (let i = 0; i < queries.length; i++) {
      requestQueue.push(getTrainingData(queries[i], p));

      if (
        requestQueue.length >= MAX_CONCURRENT_REQUESTS ||
        i === queries.length - 1
      ) {
        await Promise.all(requestQueue);
        requestQueue.length = 0; // Clear the queue
      }
    }
  }
};

getTrainingDataForAllQueries().then(() => {
  const filePath = path.join(".", "training_data.json"); // Adjust the file path as needed
  const jsonData = JSON.stringify(data);

  fs.writeFile(filePath, jsonData, (err) => {
    if (err) {
      console.log(data);
    } else {
      console.log("Data has been written to data.json");
    }
  });
});
