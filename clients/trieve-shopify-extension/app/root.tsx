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
        <link rel="preconnect" href="https://cdn.shopify.com/" />
        <link
          rel="stylesheet"
          href="https://cdn.shopify.com/static/fonts/inter/v4/styles.css"
        />
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
