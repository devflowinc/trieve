```js
fetch("https://api.trieve.ai/api/chunk/search", {
  method: "POST",
  headers: {
    "Content-Type": "application/json",
    "TR-Dataset": "********-****-****-****-************",
    Authorization: "tr-********************************",
  },
  body: JSON.stringify({
    content: "AI will take over and maybe reward those who aided its rise",
    search_type: "hybrid",
  }),
});
```
