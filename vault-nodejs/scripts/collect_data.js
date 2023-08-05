import fetch from "node-fetch";
import fs from "fs";
import path from "path";

const queries = [
  "AI will create millions and millions of jobs that currently don't exist and be a massive net positive for everyone",
  "Morality is not a sliding scale but a set of rules that must be followed unilaterally",
  "Economic situations do not have significant effects on the overall happiness and satisfaction levels of the population",
  "Global warming is actually not that bad and is completely under control. We should not be very concerned",
  "The impact of automation on traditional manufacturing jobs in developed countries",
  "The ethical implications of advanced AI in military applications",
  "Exploring cultural relativism: Is morality subjective or objective?",
  "The role of education in preparing future generations for an AI-dominated workforce",
  "The psychological effects of long-term isolation on astronauts during space missions",
  "The history and significance of the Industrial Revolution in shaping modern society",
  "The cultural and economic benefits of preserving endangered languages",
  "The potential risks and benefits of gene editing technology (CRISPR)",
  "The role of blockchain technology in revolutionizing supply chain management",
  "The influence of social media on political polarization and echo chambers",
  "Comparative analysis of healthcare systems in different countries",
  "The impact of income inequality on social stability and crime rates",
  "The history and cultural significance of traditional festivals around the world",
  "The future of renewable energy: Advancements in solar, wind, and hydro power",
  "The portrayal of artificial intelligence in popular science fiction literature",
  "The cognitive and emotional benefits of bilingualism in early childhood development",
  "The conservation efforts and challenges related to protecting marine biodiversity",
  "The role of women in STEM fields and efforts to promote gender diversity",
  "The influence of ancient philosophical principles on modern ethical debates",
  "Exploring the concept of time: Is time travel theoretically possible?",
  "The impact of mass surveillance on individual privacy and civil liberties",
  "The history and evolution of human rights across different civilizations",
  "The potential benefits and drawbacks of a cashless society",
  "The intersection of art and technology: Digital art and its cultural implications",
  "The psychology of decision-making: Factors influencing human choices",
  "The effects of meditation and mindfulness on mental health and well-being",
  "The history and effects of pandemics on societies throughout history",
  "The ethical considerations of animal testing in scientific research",
  "The economic and social implications of an aging population",
  "The philosophy of existentialism and its relevance in today's world",
  "The portrayal of diversity and inclusivity in contemporary media",
  "The cultural significance of traditional music and its role in modern society",
  "Exploring the Fermi Paradox: Are we alone in the universe?",
  "The impact of colonialism on indigenous cultures and societies",
  "The rise of virtual reality and its applications in various industries",
  "The psychology of conspiracy theories: Why do people believe in them?",
  "The challenges and potential solutions for sustainable urban development",
  "The historical accuracy of myths and legends in different cultures",
  "The future of space exploration and its potential benefits for humanity",
  "The psychology of decision-making and its role in everyday life",
  "Debunking common myths about vaccines and understanding their importance",
  "The history and evolution of human rights around the world",
  "The impact of globalization on traditional cultures and local economies",
  "The relationship between diet, exercise, and overall health",
  "Exploring the concept of time travel in physics and popular culture",
  "The role of women in leadership and their contributions to society",
  "The ethics of artificial intelligence and its potential risks",
  "The significance of renewable energy sources in mitigating climate change",
  "The history and cultural significance of storytelling in different societies",
  "Examining the pros and cons of a cashless society and digital currencies",
  "The effects of music on emotions and cognitive functions",
  "The philosophy of existentialism and its influence on literature and thought",
  "Understanding the concept of consciousness and its mysteries",
  "The impact of colonialism on indigenous cultures and societies",
  "The implications of artificial wombs on reproduction and gender roles",
  "Analyzing the sociopolitical impact of universal basic income",
  "Exploring the concept of beauty across different cultures and time periods",
  "The role of empathy in fostering social cohesion and reducing conflicts",
  "The future of work: How will automation affect job satisfaction and fulfillment?",
  "The effects of climate change on biodiversity and ecosystem stability",
  "The ethics of using CRISPR technology for human enhancement",
  "The evolution of language: From gestures to complex communication systems",
  "The cultural significance of traditional clothing in contemporary society",
  "The potential risks and rewards of asteroid mining for Earth's resources",
  "The psychology of online behavior: Trolling, cyberbullying, and digital etiquette",
  "The impact of artificial intelligence on creative fields such as art and music",
  "Exploring the concept of consciousness: Can machines ever be truly self-aware?",
  "The history and influence of ancient trade routes on modern globalization",
  "The role of nostalgia in shaping personal identity and consumer behavior",
  "The effects of automation on the future of transportation and mobility",
  "The portrayal of mental health in popular media: Stereotypes and progress",
  "The relationship between economic development and environmental conservation",
  "The ethics of human cloning: Scientific possibilities and ethical dilemmas",
  "The psychology of motivation: What drives human behavior and achievement?",
  "The impact of virtual reality on therapeutic applications, such as exposure therapy",
  "The role of architecture in influencing human behavior and well-being",
  "The cultural implications of language extinction and efforts to revitalize languages",
  "Exploring the concept of free will: Do we truly have the power to choose?",
  "The effects of social isolation on mental health in the digital age",
  "The role of humor in coping with adversity and building social connections",
  "The history and significance of the Silk Road in connecting civilizations",
  "The ethics of surveillance capitalism: Data privacy and corporate responsibility",
  "The potential benefits and drawbacks of decentralized digital currencies",
  "The psychology of fear: How do phobias and anxieties develop?",
  "The impact of automation on the future of healthcare and medical professions",
  "The relationship between urban design and community well-being",
  "The evolution of human nutrition: From foraging to genetically modified foods",
  "The ethics of artificial intelligence in artistic creation and copyright",
  "The influence of cultural beliefs on attitudes toward mental health treatment",
  "The role of play and leisure activities in cognitive development",
  "The effects of disinformation and fake news on democratic societies",
  "The future of transportation: Hyperloop, self-driving cars, and beyond",
  "The philosophy of identity: What defines the essence of 'self'?",
  "The impact of social robots on human interactions and companionship",
  "The representation of gender and sexuality in contemporary art",
  "The influence of architecture on social inequality and urban segregation",
  "The effects of gentrification on local communities and identity",
  "The ethical considerations of using AI for personalized medicine",
  "The psychology of resilience: How do some individuals bounce back from adversity?",
  "The implications of quantum computing for cryptography and cybersecurity",
  "The role of traditional medicine practices in modern healthcare",
  "The effects of advertising and consumerism on individual happiness",
  "The philosophy of mind-body dualism and its relevance in modern science",
  "The impact of social media activism on real-world social and political change",
  "The psychology of addiction: Understanding substance abuse and behavioral addictions",
  "The relationship between cultural diversity and innovative thinking",
  "The ethics of geoengineering as a solution to climate change",
  "The implications of lab-grown meat for food sustainability and ethics",
  "Fish farming and its impact on marine ecosystems and biodiversity",
  "The role of religion in shaping cultural values and social norms",
  "Pets and animals as companions: The benefits and responsibilities of pet ownership",
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
      let shortestContent = curMetadatas[0].content;
      let shortestCardHTML = curMetadatas[0].card_html;

      for (let j = 1; j < curMetadatas.length; j++) {
        if (curMetadatas[j].content.length < shortestContent.length) {
          shortestContent = curMetadatas[j].content;
          shortestCardHTML = curMetadatas[j].card_html;
        }
      }

      if (shortestContent.length > 4300) {
        continue;
      }

      data.push({
        content: shortestContent,
        card_html: shortestCardHTML,
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
  const pages = [1, 2, 3, 4, 5];

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
