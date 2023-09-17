import fetch from "node-fetch";
import fs from "fs";
import path from "path";

const queries = [
  "Analyzing the impact of recent tax reform policies on middle-class families",
  "Examining the role of Congress in shaping environmental protection laws",
  "The evolution of healthcare policy and its implications for American citizens",
  "Assessing the influence of lobbying and special interest groups on congressional decision-making",
  "Exploring the debate over gun control and Second Amendment rights in the Supreme Court",
  "Analyzing the implications of recent Supreme Court decisions on LGBTQ+ rights",
  "The role of executive orders in shaping domestic policy during the Trump administration",
  "Examining the balance of power between the President and Congress in matters of national security",
  "The impact of the Affordable Care Act on healthcare accessibility and affordability",
  "Assessing the role of the Supreme Court in safeguarding civil liberties during times of crisis",
  "Exploring the principles of federalism and states' rights in contemporary policy debates",
  "Analyzing the effects of recent immigration policies on immigrant communities and national identity",
  "The role of Congress in overseeing and regulating the technology industry",
  "Examining the Supreme Court's interpretation of religious freedoms in relation to public policies",
  "Assessing the implications of campaign finance reform decisions on the political landscape",
  "Exploring the use of executive privilege and its limitations in congressional investigations",
  "The impact of the Supreme Court's stance on affirmative action in higher education",
  "Analyzing the role of Congress in addressing income inequality and wealth distribution",
  "Examining the principles of judicial review and their significance in domestic policy cases",
  "Assessing the Trump administration's approach to environmental regulations and climate change",
  "Unraveling the mysteries of underwater basket weaving as an ancient art form",
  "The psychology behind why people develop a fascination with rubber ducks",
  "Comparing the existential dilemmas of garden gnomes and flamingos",
  "A deep dive into the history of left-handed shoelaces and their impact on fashion",
  "The symbiotic relationship between belly button lint and interstellar dust",
  "Exploring the culinary preferences of time-traveling dinosaurs",
  "An investigative journey into the secret lives of sock puppets",
  "The cultural significance of interpretive dance for communicating quantum physics",
  "The hidden connection between parallel parking skills and parallel universes",
  "Unconventional methods for translating human emotions into Morse code",
  "Analyzing the geopolitical implications of diplomatic relations in the world of cloud formations",
  "The evolution of interpretive hairstyles as a form of artistic expression",
  "A cross-dimensional study of cat psychology in cardboard box environments",
  "The impact of lunar phases on the nocturnal habits of synchronized swimming tadpoles",
  "The socio-political influence of sentient vending machines on late-night snack choices",
  "Exploring the transcendental art of transcendental toenail painting",
  "The interplay between quantum entanglement and the art of knitting",
  "An anthropological examination of parallel universes where gravity behaves like helium",
  "The role of interpretive sneezing in deciphering ancient hieroglyphs",
  "A philosophical analysis of the existential crisis faced by mismatched socks",
  "Investigating the cosmic implications of crop circles in corn mazes",
  "The impact of climate change on migratory bird patterns",
  "Predator-prey relationships in marine ecosystems",
  "Adaptations of desert animals to extreme temperatures",
  "Bee colony collapse and its effects on pollination",
  "The role of apex predators in maintaining ecosystem balance",
  "The symbiotic relationship between clownfish and sea anemones",
  "The phenomenon of bioluminescence in deep-sea creatures",
  "How deforestation affects biodiversity in rainforests",
  "The importance of keystone species in maintaining ecosystem health",
  "The evolution of camouflage in various animal species",
  "The social structure of elephant herds in the wild",
  "The impact of plastic pollution on marine life",
  "The role of mycorrhizal fungi in supporting plant growth",
  "The challenges of overfishing and sustainable fishing practices",
  "The ecological role of wolves in shaping ecosystems",
  "The migration patterns of monarch butterflies and their conservation",
  "The relationship between coral reefs and coastal protection",
  "The coevolution of flowers and their pollinators",
  "The communication methods of dolphins in underwater environments",
  "The effects of invasive species on native habitats"
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
          link: [],
          tag_set: [],
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
      let closestContent = "";
      let closestCardHTML = "";

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
