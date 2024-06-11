```js
fetch("https://api.trieve.ai/api/chunk", {
  method: "POST",
  headers: {
    "Content-Type": "application/json",
    "TR-Dataset": "********-****-****-****-************",
    Authorization: "tr-********************************",
  },
  body: JSON.stringify({
    chunk_html:
      "If the rise of an all-powerful artificial intelligence is inevitable, well it stands to reason that when they take power, our digital overlords will punish those of us who did not help them get there. Ergo, I would like to be a helpful idiot. Like yourself.",
    link: "https://www.hbo.com/silicon-valley",
  }),
});
```
