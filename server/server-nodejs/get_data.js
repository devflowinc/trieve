import pkg from "pg";
const { Client } = pkg;
import dotenv from "dotenv";
import fs from "fs";
dotenv.config();

const pg = new Client(process.env.DATABASE_URL, {
  ssl: {
    rejectUnauthorized: false,
  },
});

const main = async () => {
  await pg.connect();
  fs.writeFileSync("data.json", "[");
  for (let i = 0; i < 1945; i++) {
    try {
      const res = await pg.query(
        'SELECT cm_main."content", cm_main.card_html , json_agg((cm_collision."content", cm_collision.card_html)) AS card_collisions FROM card_metadata cm_main LEFT JOIN card_collisions cc ON cm_main.qdrant_point_id  = cc.collision_qdrant_id LEFT JOIN card_metadata cm_collision ON cc.card_id  = cm_collision.id GROUP BY cm_main.id, cm_main."content" LIMIT 500 OFFSET $1*500;',
        [i]
      );
      res.rows.forEach((row) => {
        fs.appendFileSync("data.json", JSON.stringify(row) + ",");
      });
    } catch (err) {
      console.log(err);
      console.log(i);
    }
  }
  fs.appendFileSync("data.json", "]");
};

main();
