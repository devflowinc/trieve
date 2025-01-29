import type { LoaderFunctionArgs } from "@remix-run/node";
import { redirect } from "@remix-run/node";
import { useFetcher } from "@remix-run/react";
import jwt, { JwtPayload } from "jsonwebtoken";

import { authenticate, login } from "../../shopify.server";

import { useEffect, useState } from "react";
import { time } from "console";

export const loader = async ({ request }: LoaderFunctionArgs) => {
  const url = new URL(request.url);

  if (url.searchParams.get("shop")) {
    throw redirect(`/app?${url.searchParams.toString()}`);
  }

  return { showForm: Boolean(login) };
};

export const action = async ({ request }: LoaderFunctionArgs) => {
  const formData = await request.formData();
  const type = formData.get("type")!;
  const apiKey = formData.get("apiKey")!;
  const idToken = formData.get("idToken")!;
  const orgId = formData.get("orgId")!;

  const decoded = jwt.verify(
    idToken.toString(),
    process.env.SHOPIFY_API_SECRET!,
    {
      algorithms: ["HS256"],
      clockTolerance: 10,
    },
  ) as JwtPayload;

  const now = Math.floor(Date.now() / 1000);

  if (decoded.exp && decoded.exp < now) {
    throw new Error("Token expired");
  }

  if (decoded.nbf && decoded.nbf > now) {
    throw new Error("Token not yet valid");
  }
  if (type === "insert") {
    await prisma.apiKey.create({
      data: {
        userId: decoded.sub ?? "",
        organizationId: orgId.toString(),
        key: apiKey.toString(),
        createdAt: new Date(),
      },
    });
    return null;
  } else if (type === "get") {
    const key = await prisma.apiKey.findFirst({
      where: {
        userId: decoded.sub ?? "",
      },
    });
    return key;
  }
};

type User = {
  email: string;
  id: string;
  name: string;
  orgs: Orgs[];
};

type Orgs = {
  id: string;
  name: string;
};

export default function App() {
  const fetcher = useFetcher<typeof action>();
  const [orgs, setOrgs] = useState<Orgs[]>([]);
  const [selectedOrg, setSelectedOrg] = useState<string | null>(null);
  const [userApiKey, setUserApiKey] = useState<string | null>(null);

  useEffect(() => {
    fetch("https://api.trieve.ai/api/auth/me", {
      credentials: "include",
    }).then((response) => {
      if (response.status === 401) {
        window.location.href =
          "https://api.trieve.ai/api/auth?redirect_uri=https://dens-shopify.trieve.ai";
      }
      response.json().then((data: User) => {
        setOrgs(data.orgs);
      });
    });
  }, []);

  useEffect(() => {
    let params = new URLSearchParams(window.location.search);
    fetcher.submit(
      { type: "get", idToken: params.get("token") },
      { method: "POST" },
    );

    setUserApiKey(fetcher.data?.key ?? "");
  }, [fetcher.data?.key]);

  const generateApiKey = () => {
    if (!selectedOrg) {
      return;
    }
    fetch("https://api.trieve.ai/api/organization/api_key", {
      method: "POST",
      credentials: "include",
      headers: {
        "Content-Type": "application/json",
        "TR-Organization": selectedOrg.toString(),
      },
      body: JSON.stringify({ name: "Shopify-Access", role: 1 }),
    }).then((response) => {
      response.json().then((data) => {
        let params = new URLSearchParams(window.location.search);

        fetcher.submit(
          {
            apiKey: data.api_key,
            orgId: selectedOrg,
            idToken: params.get("token"),
            type: "insert",
          },
          { method: "POST" },
        );
      });
    });
  };

  return (
    <div>
      <select
        value={selectedOrg ?? undefined}
        onChange={(e) => setSelectedOrg(e.target.value)}>
        {orgs.map((org) => (
          <option key={org.id} value={org.id}>
            {org.name}
          </option>
        ))}
      </select>
      <button onClick={() => generateApiKey()}>Generate API Key</button>
      {userApiKey && <p>Your API Key: {userApiKey}</p>}
    </div>
  );
}
