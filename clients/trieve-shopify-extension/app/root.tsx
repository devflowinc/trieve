import {
  Links,
  Meta,
  Outlet,
  Scripts,
  ScrollRestoration,
  useLoaderData,
} from "@remix-run/react";
import { AppEnvProvider } from "./context/useEnvs";
import { getTrieveBaseUrlEnv } from "./env.server";

export async function loader() {
  return {
    ENV: {
      TRIEVE_BASE_URL: getTrieveBaseUrlEnv(),
    },
  };
}

export default function App() {
  const data = useLoaderData<typeof loader>();
  return (
    <html>
      <head>
        <meta charSet="utf-8" />
        <meta name="viewport" content="width=device-width,initial-scale=1" />
        <meta property="og:title" content="Trieve Shopify App" />
        <meta
          property="og:site_name"
          content="Trieve - AI Sales Associate for Ecommerce"
        />
        <meta property="og:url" content="https://docsearch.trieve.ai" />
        <meta
          property="og:description"
          content="Replicate your best sales associate with Trieve. Our AI assistant helps you answer customer questions, generate product descriptions, and more."
        />
        <meta
          property="og:image"
          content="https://cdn.trieve.ai/trieve-og.png"
        />
        <link rel="preconnect" href="https://cdn.shopify.com/" />
        <link
          rel="stylesheet"
          href="https://cdn.shopify.com/static/fonts/inter/v4/styles.css"
        />
        <link
          rel="apple-touch-icon"
          sizes="180x180"
          href="https://cdn.trieve.ai/apple-touch-icon.png"
        />
        <link
          rel="icon"
          type="image/png"
          sizes="32x32"
          href="https://cdn.trieve.ai/favicon-32x32.png"
        />
        <link
          rel="icon"
          type="image/png"
          sizes="16x16"
          href="https://cdn.trieve.ai/favicon-16x16.png"
        />
        <title>Trieve Shopify App</title>
        <Meta />
        <Links />
      </head>
      <body>
        <AppEnvProvider envs={data.ENV}>
          <Outlet />
        </AppEnvProvider>
        <ScrollRestoration />
        <Scripts />
      </body>
    </html>
  );
}
