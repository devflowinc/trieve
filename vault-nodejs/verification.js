import redis from "redis";
import redisScan from "node-redis-scan";
import pkg from "pg";
const { Client } = pkg;
import { spawnSync } from "child_process";
import { v4 as uuidv4 } from "uuid";
import pdf2html from "pdf2html";
import { createWriteStream, unlink } from "fs";
import { pipeline } from "stream/promises";
import { convert } from "html-to-text";
import { parse } from "node-html-parser";
import dotenv from "dotenv";

dotenv.config();
process.setMaxListeners(50);

const keyvDb = new redis.createClient({
  url: process.env.REDIS_URL || "redis://127.0.0.1:6379",
  legacyMode: true,
});
const pg = new Client(
  process.env.DATABASE_URL ||
    "postgresql://postgres:password@localhost:5432/ai_editor"
);
let MAX_CONCURRENT_REQUESTS = 5;
let activeRequests = 0;
const scanner = new redisScan(keyvDb);
function getInnerHtml(html) {
  const root = parse(html);
  let body = root.getElementsByTagName("body")[0];
  if (!body) {
    body = root;
  }
  return convert(body.innerHTML);
}
import puppeteer from "puppeteer-extra";

import StealthPlugin from "puppeteer-extra-plugin-stealth";
puppeteer.use(StealthPlugin());
let browser;

async function fetchPageContents(url) {
  while (!browser) {
    await new Promise((r) => setTimeout(r, 100));
  }
  const page = await browser.newPage();
  page.setDefaultTimeout(5000);

  try {
    await page.goto(url);
    let body = await page.waitForSelector("body", { timeout: 5000 });
    const pageContent = await page.evaluate(() => {
      const elements = document.querySelectorAll("body");
      let text = "";
      for (const element of elements) {
        if (element.innerText) {
          text += element.innerText + "\n";
        }
      }
      return text;
    }, body);

    const cleanedContent = pageContent
      .replace(/(\r\n|\n|\r)/gm, " ")
      .replace(/\s+/g, " ")
      .trim();

    return cleanedContent;
  } catch (error) {
    console.error("failed to fetch webpage " + error.stack.split("\n")[0]);
    return "";
  } finally {
    await page.close();
  }
}
async function get_webpage_text_fetch(link) {
  try {
    let content = await fetch(link).then(async (response) => {
      if (response.headers.get("content-type").includes("application/pdf")) {
        let fileName = `../tmp/${uuidv4()}.pdf`;
        pipeline(response.body, createWriteStream(fileName));
        let html = await pdf2html.html(fileName, { maxBuffer: 1024 * 10000 });
        html = getInnerHtml(html);
        unlink(fileName, (err) => {
          console.error(err, (err) => {
            if (err) throw err; //handle your error the way you want to;
            console.log(`${fileName} was deleted`);
            //or else the file will be deleted
          });
        });
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
    console.error("failed to fetch webpage " + error.stack.split("\n")[0]);
    return "";
  }
}

async function get_webpage_score(link, content) {
  let webpage_text = "";
  if (content) {
    webpage_text = await get_webpage_text_fetch(link, content);
  }
  let score = 0;
  if (webpage_text !== "") {
    try {
      const { stdout } = spawnSync("python", [
        "../vault-python/fuzzy-text-match.py",
        content,
        webpage_text.replace("\n", ""),
      ]);
      score = stdout.toString();
    } catch (error) {
      console.error(
        "failed to get fuzzy text match score " + error.stack.split("\n")[0]
      );
    }
  }
  if (score < 80) {
    let pageContent = "";
    if (content) {
      pageContent = await fetchPageContents(link, content);
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

async function getKeys(matchingKeys) {
  // matchingKeys will be an array of strings if matches were found
  // otherwise it will be an empty array.
  let card_uuids = [];

  puppeteer
    .launch({ headless: true })
    .then((br) => {
      browser = br;
    })
    .catch((err) => {
      console.error(err);
    });
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

    if (card_metadata.rows.length === 0) {
      console.error("Card not found");
      activeRequests--;

      await keyvDb.del(`Verify: ${card}`);
      continue;
    }

    let score = await get_webpage_score(
      card_metadata.rows[0].link,
      card_metadata.rows[0].content
    );

    let verification_uuid = uuidv4();
    const verif_insert = await pg.query(
      `INSERT INTO card_verification (id, card_id, similarity_score) 
        VALUES ($1, $2, $3) 
        ON CONFLICT (card_id) 
        DO UPDATE SET similarity_score = $3
        RETURNING id;`,
      [verification_uuid, card, score]
    );
    await pg.query(
      `INSERT INTO verification_notifications (id, user_uuid, card_uuid, verification_uuid, similarity_score, user_read)
        VALUES ($1, $2, $3, $4, $5, $6)`,
      [
        uuidv4(),
        card_metadata.rows[0].author_id,
        card,
        verif_insert.rows[0].id,
        score,
        false,
      ]
    );
    console.log(`Verified card ${card} with score ${score}`);

    await keyvDb.del(`Verify: ${card}`);
  }
  await browser.close();
}

async function verifyCard() {
  await keyvDb.connect();
  await pg.connect();
  scanner.eachScan(
    "Verify:*",
    { count: 1000 },
    async (matchingKeys) => {
      while (activeRequests >= MAX_CONCURRENT_REQUESTS) {
        await new Promise((resolve) => setTimeout(resolve, 1000));
      }
      activeRequests++;
      await getKeys(matchingKeys);
      activeRequests--;
      console.log(activeRequests);
    },
    (err, count) => {
      if (err) throw err;
      console.log(`Found ${count} cards to verify`);
    }
  );
}

// Call verifyCard every 2 mniutes
verifyCard();
