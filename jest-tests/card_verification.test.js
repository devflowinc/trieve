import fetch from "node-fetch";

const api_endpoint = process.env.API_ENDPOINT || "http://localhost:8090/api";

describe("Card Verification Tests", () => {
  test("Verification with exact match", async () => {
    const response = await fetch(`${api_endpoint}/verification`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        url_source: "https://www.example.com",
        content: "Example Domain This domain is for use in illustrative examples in documents. You may use this domain in literature without prior coordination or asking for permission.",
      }),
    });
    const json = await response.json();
    expect(json).toHaveProperty("score");
    console.log("Score: ", json.score);
  });

  test("Verification with exact match and slight changes", async () => {
    let content = "Example Domain This domain is for use in illustrative examples in documents. You may use this domain in literature without prior coordination or asking for permission."
    content += "L";
    const response = await fetch(`${api_endpoint}/verification`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        url_source: "https://www.example.com",
        content,
      }),
    });
    const json = await response.json();
    expect(json).toHaveProperty("score");
    console.log("Score: ", json.score);
  });
});
