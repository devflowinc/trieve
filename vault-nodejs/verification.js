import redis from "redis";
import redisScan from "node-redis-scan";
import pkg from "pg";
const { Client } = pkg;
import { spawnSync } from "child_process";
import { fetchPageContents } from "./verifcation-server.js";
import { v4 as uuidv4 } from "uuid";
import pdf2html from "pdf2html";
import { createWriteStream, unlink } from "fs";
import { pipeline } from "stream/promises";
import { convert } from "html-to-text";
import { parse } from "node-html-parser";

const keyvDb = new redis.createClient({
  url: process.env.REDIS_URL || "redis://127.0.0.1:6379",
  legacyMode: true,
});
const pg = new Client({
  user: "postgres",
  host: "localhost",
  database: "ai_editor",
  password: "password",
  port: 5432,
});

const scanner = new redisScan(keyvDb);
function getInnerHtml(html) {
  const root = parse(html);
  let body = root.getElementsByTagName("body")[0];
  if (!body) {
    body = root;
  }
  return convert(body.innerHTML);
}
async function get_webpage_text_fetch(link) {
  try {
    let content = await fetch(link).then(async (response) => {
      if (response.headers.get("content-type").includes("application/pdf")) {
        let fileName = `../tmp/${uuidv4()}.pdf`;
        pipeline(response.body, createWriteStream(fileName));
        let html = await pdf2html.html(fileName);
        html = getInnerHtml(html);
        unlink(fileName);
        return html.replaceAll("/s+/", "");
      } else if (!response.headers.get("content-type").includes("text/html")) {
        console.error("Not a webpage");
        return "";
      }
      let text = response.text().then((text) => {
        let html = getInnerHtml(text);
        if (!html) {
          console.error("No html found.");
          return "";
        }
        return html.replaceAll("/s+/", "");
      });
      return text;
    });

    return content;
  } catch (error) {
    console.error(error);
    return "";
  }
}

async function get_webpage_score(link, content) {
  //TODO: get await to work
  let webpage_text = await get_webpage_text_fetch(link);
  let score = 0;
  if (webpage_text !== "") {
    const { stdout } = spawnSync("python", [
      "../vault-python/fuzzy-text-match.py",
      content,
      webpage_text.replace("\n", ""),
    ]);
    score = stdout.toString();
  }
  if (score < 80) {
    let pageContent = "";
    try {
      pageContent = await fetchPageContents(link);
    } catch (error) {
      console.error("failed to fetch webpage");
    }
    const { stdout } = spawnSync("python", [
      "../vault-python/fuzzy-text-match.py",
      content,
      pageContent.replace("\n", ""),
    ]);
    score = stdout.toString();
  }
  return score;
}
async function verifyCard() {
  await keyvDb.connect();
  await pg.connect();
  let card_uuids = [];
  scanner.scan("Verify:*", async (err, matchingKeys) => {
    if (err) throw err;

    // matchingKeys will be an array of strings if matches were found
    // otherwise it will be an empty array.
    card_uuids = matchingKeys;
    card_uuids = card_uuids.map((card_uuid) => {
      card_uuid = card_uuid.replace("Verify: ", "");
      return card_uuid;
    });
    for (let card of card_uuids) {
      let card_metadata = await pg.query(
        "SELECT * FROM card_metadata WHERE id = $1",
        [card]
      );

      let score = await get_webpage_score(
        card_metadata.rows[0].link,
        card_metadata.rows[0].content
      );
      let verification_uuid = uuidv4();
      await pg.query(
        `INSERT INTO card_verification (id, card_id, similarity_score) 
        VALUES ($1, $2, $3) 
        ON CONFLICT (card_id) 
        DO UPDATE SET similarity_score = $3;`,
        [verification_uuid, card, score]
      );

      await pg.query(
        `INSERT INTO verification_notifications (id, user_uuid, card_uuid, verification_uuid, similarity_score, user_read) 
        VALUES ($1, $2, $3, $4, $5, $6)`,
        [
          uuidv4(),
          card_metadata.rows[0].author_id,
          card,
          verification_uuid,
          score,
          false,
        ]
      );
      console.log(`Verified card ${card} with score ${score}`);

      await keyvDb.del(`Verify: ${card}`);
    }
    pg.end();
    keyvDb.quit();
    process.exit(0);
  });
}

verifyCard();
