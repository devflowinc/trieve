import { Hono } from "hono";
import { judgeMe } from "./judgeMe";

export const api = new Hono()
  .onError((err, c) => {
    console.error(err);
    c.status(500);
    return c.json({
      error: err.message,
    });
  })
  .basePath("/api")
  .route("/judgeme", judgeMe);

export type ApiType = typeof api;
