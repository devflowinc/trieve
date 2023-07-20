import http from 'http';
import puppeteer from 'puppeteer-extra';
import winston from 'winston';

import StealthPlugin from 'puppeteer-extra-plugin-stealth';
puppeteer.use(StealthPlugin())
let browser;

puppeteer.launch({ headless: true }).then(br => {
  browser = br;
}).catch(err => {
  console.error(err);
});



// Define log format
const logFormat = winston.format.printf(({ level, message, timestamp }) => {
  return `${timestamp} ${level}: ${message}`;
});

// Create logger instance
const log = winston.createLogger({
  level: 'info', // Set the minimum log level
  format: winston.format.combine(
    winston.format.timestamp({ format: 'YYYY-MM-DD HH:mm:ss' }),
    logFormat
  ),
  transports: [
    // Define transports (where the logs will be stored)
    new winston.transports.Console(), // Log to console
    new winston.transports.File({ filename: 'logs.log' }), // Log to a file
  ],
});


async function fetchPageContents(url) {
  const page = await browser.newPage();
  page.setDefaultTimeout(5000);

  try {
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
    return cleanedContent;
  } finally {
    await page.close();
  }
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
        if (jsonData.url === undefined) {
          res.statusCode = 400;
          log.info(`${requestBody} 400 Invalid JSON data`);
          res.end(JSON.stringify({ error: 'Invalid JSON data' }));
        }
        fetchPageContents(jsonData.url).then((pageContent) => {
          res.statusCode = 200;
          log.info(`${requestBody} 200 valid`);
          res.end(JSON.stringify({ content: pageContent }));
        }).catch((err) => {
          console.error(err);
          res.statusCode = 400;
          log.info(`${requestBody} 400 Invalid JSON data`);
          res.end(JSON.stringify({ error: 'Failed fetching page contents' }));
        })
      } catch (error) {
        console.error(error);
        // Handle JSON parsing error
        res.statusCode = 400;
        log.info(`${requestBody} 400 Invalid JSON data`);
        res.end(JSON.stringify({ error: 'Invalid JSON data' }));
      }
    });
  } else {
    // Handle other routes
    res.statusCode = 404;
    log.info('404 Not Found');
    res.end(JSON.stringify({ error: 'Route not found' }));
  }
});

// Start the server
const port = 8091;
server.listen(port, () => {
  log.info(`Server running on port ${port}`);
});
