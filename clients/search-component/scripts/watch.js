import { context as esbuild } from "esbuild";
import { options } from "./build.js";

let ctx = await esbuild(options);

await ctx.watch();
console.log("watching...");
