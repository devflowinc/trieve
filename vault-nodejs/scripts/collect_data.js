import fs from "fs";
import path from "path";
import pkg from "pg";
const { Client } = pkg;
import dotenv from "dotenv";
dotenv.config();

let data = [];
const pg = new Client(process.env.DATABASE_URL, {
  ssl: {
    rejectUnauthorized: false,
  },
});

pg.on("close", () => {
  // reconnect to the database
  pg.connect();
});

pg.on("error", () => {
  pg.connect();
});
const getTrainingData = async (metadata) => {
  try {
    const curMetadatas = [
      { card_html: metadata.card_html, content: metadata.content },
      ...metadata.card_collisions.map((collision) => {
        return { card_html: collision.f2, content: collision.f1 };
      }),
    ];
    let closestContent = "";
    let closestCardHTML = "";

    let closestCardLengthDiff = Infinity;

    for (let j = 0; j < curMetadatas.length; j++) {
      if (!curMetadatas[j].card_html) continue;
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
      return;
    }

    data.push({
      content: closestContent,
      card_html: closestCardHTML,
    });
    console.log("Pushed: ", data.length);
  } catch (err) {
    console.log(err);
    return;
  }
};

const MAX_CONCURRENT_REQUESTS = 1;

const getTrainingDataForAllQueries = async () => {
  const requestQueue = [];
  await pg.connect();
  fs.writeFileSync("data.json", "[");
  for (let i = 0; i < 1945 * 5; i++) {
    const res = await pg.query(
      'SELECT cm_main."content", cm_main.card_html , json_agg((cm_collision."content", cm_collision.card_html)) AS card_collisions FROM card_metadata cm_main LEFT JOIN card_collisions cc ON cm_main.qdrant_point_id  = cc.collision_qdrant_id LEFT JOIN card_metadata cm_collision ON cc.card_id  = cm_collision.id GROUP BY cm_main.id, cm_main."content" LIMIT 100 OFFSET $1*100;',
      [i]
    );
    res.rows.forEach(async (row) => {
      requestQueue.push(getTrainingData(row));

      if (requestQueue.length >= MAX_CONCURRENT_REQUESTS) {
        await Promise.all(requestQueue);
        requestQueue.length = 0; // Clear the queue
      }
    });
    fs.appendFileSync("data.json", JSON.stringify(data) + ",");
  }
  fs.appendFileSync("data.json", "]");
};

getTrainingDataForAllQueries().then(() => {});
