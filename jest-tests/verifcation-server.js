import http from 'http';
import puppeteer from 'puppeteer-extra';

import StealthPlugin from 'puppeteer-extra-plugin-stealth';
puppeteer.use(StealthPlugin())
let browser;

puppeteer.launch({ headless: true }).then(br => {
  browser = br;
}).catch(err => {
  console.error(err);
});

async function fetchPageContents(url) {
  const page = await browser.newPage();

  await page.goto(url);

  let body = await page.waitForSelector("body", { timeout: 5000 });
  const pageContent = await page.evaluate(() => {
    const elements = document.querySelectorAll('body');
    let text = '';
    for (const element of elements) {
      if (element.innerText) {
        text += element.innerText + '\n';
      }
    }
    return text;
  }, body);

  const cleanedContent = pageContent.replace(/(\r\n|\n|\r)/gm, " ").replace(/\s+/g, " ").trim();
  await page.close();

  return cleanedContent;
}

// Create the server
const server = http.createServer((req, res) => {
  // Set the response headers
  res.setHeader('Content-Type', 'application/json');

  // Handle the route for receiving JSON input
  if (req.method === 'POST' && req.url === '/get_url_content') {
    let requestBody = '';

    // Read the incoming data
    req.on('data', (chunk) => {
      requestBody += chunk;
    });

    // Process the JSON data once all data has been received
    req.on('end', () => {
      try {
        const jsonData = JSON.parse(requestBody);
        console.log(jsonData);
        if (jsonData.url === undefined) {
          res.statusCode = 400;
          res.end(JSON.stringify({ error: 'Invalid JSON data' }));
        }
        fetchPageContents(jsonData.url).then((pageContent) => {
          res.statusCode = 200;
          res.end(JSON.stringify({ content: pageContent }));
        }).catch((err) => {
          console.error(err);
          res.statusCode = 400;
          res.end(JSON.stringify({ error: 'Failed fetching page contents' }));
        })
      } catch (error) {
        console.error(error);
        // Handle JSON parsing error
        res.statusCode = 400;
        res.end(JSON.stringify({ error: 'Invalid JSON data' }));
      }
    });
  } else {
    // Handle other routes
    res.statusCode = 404;
    res.end(JSON.stringify({ error: 'Route not found' }));
  }
});

// Start the server
const port = 8091;
server.listen(port, () => {
  console.log(`Server running on port ${port}`);
});
