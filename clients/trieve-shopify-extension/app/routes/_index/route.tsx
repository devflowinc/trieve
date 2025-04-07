import type { LoaderFunctionArgs } from "@remix-run/node";
import { redirect } from "@remix-run/node";
import "./tailwind.css";
import { useFetcher, useLoaderData } from "@remix-run/react";
import jwt, { JwtPayload } from "jsonwebtoken";

import { useEffect, useState } from "react";
import { useEnvs } from "app/context/useEnvs";

export const loader = async ({ request }: LoaderFunctionArgs) => {
  const url = new URL(request.url);

  if (url.searchParams.get("shop")) {
    throw redirect(`/app?${url.searchParams.toString()}`);
  }

  return null;
};

export const action = async ({ request }: LoaderFunctionArgs) => {
  const formData = await request.formData();
  const type = formData.get("type")!;
  const apiKey = formData.get("apiKey")!;
  const idToken = formData.get("idToken")!;
  const orgId = formData.get("orgId")!;

  const decoded = jwt.decode(idToken.toString()) as JwtPayload;

  if (type === "insert") {
    const existingKey = await prisma.apiKey.findFirst({
      where: {
        userId: decoded.sub ?? "",
        organizationId: orgId.toString(),
        shop: decoded.dest ?? "",
      },
    });
    if (existingKey) {
      return existingKey;
    }

    const key = await prisma.apiKey.create({
      data: {
        userId: decoded.sub ?? "",
        organizationId: orgId.toString(),
        shop: decoded.dest ?? "",
        key: apiKey.toString(),
        createdAt: new Date(),
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
  const [successfulLogin, setSuccessfulLogin] = useState(false);
  const envs = useEnvs();

  useEffect(() => {
    fetch(`${envs.TRIEVE_BASE_URL}/api/auth/me`, {
      credentials: "include",
    }).then((response) => {
      if (response.status !== 200) {
        window.location.href = `${envs.TRIEVE_BASE_URL}/api/auth?redirect_uri=${window.location}`;
        return;
      }
      response.json().then((data: User) => {
        if (data.orgs.length == 1) {
          generateApiKey(data.orgs[0].id);
        } else {
          setOrgs(data.orgs);
        }
      });
    });
  }, []);

  useEffect(() => {
    if (fetcher.data?.key) {
      setSuccessfulLogin(true);
    }
  }, [fetcher.data?.key]);

  const generateApiKey = (selectedOrg: string) => {
    if (!selectedOrg) {
      return;
    }
    fetch(`${envs.TRIEVE_BASE_URL}/api/organization/api_key`, {
      method: "POST",
      credentials: "include",
      headers: {
        "Content-Type": "application/json",
        "TR-Organization": selectedOrg.toString(),
      },
      body: JSON.stringify({ name: "Shopify-Access", role: 2 }),
    })
      .then((response) => {
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
      })
      .catch((error) => {
        console.error("Error generating API key:", error);
        setSuccessfulLogin(true);
      });
  };

  return (
    <div className="relative flex min-h-screen flex-col items-center bg-neutral-200 py-36">
      <div className="absolute left-4 top-2 mb-8 flex items-center gap-1">
        <img
          className="h-12 w-12 cursor-pointer"
          src="https://cdn.trieve.ai/trieve-logo.png"
          alt="Logo"
        />
        <span className="text-2xl font-semibold">Trieve</span>
      </div>
      <div className="rounded-md border border-neutral-300 bg-white p-4 md:min-w-[500px]">
        {successfulLogin && (
          <>
            <div className="flex justify-between">
              <div className="text-lg font-medium">Login Successful 🎉</div>
            </div>
            <div className="py-2 text-sm text-neutral-700">
              You can now close this window and return to your Shopify store.
            </div>
            <div className="py-2 text-sm text-neutral-700">
              If you don't see the app, please refresh the page.
            </div>
          </>
        )}
        {!successfulLogin &&
          (orgs.length > 0 ? (
            <>
              <div className="flex justify-between">
                <div className="text-lg font-medium">
                  Select An Organization
                </div>
              </div>
              <div className="flex flex-col py-2">
                {orgs?.map((org) => (
                  <button
                    key={org.id}
                    onClick={() => {
                      generateApiKey(org.id);
                    }}
                    className="flex cursor-pointer items-center justify-between rounded-md border-b border-b-neutral-200 p-2 last:border-b-transparent hover:bg-neutral-100"
                  >
                    <div className="flex w-full items-center justify-between">
                      <div className="text-sm font-medium">{org.name}</div>
                      <div className="text-xs text-neutral-500">{org.id}</div>
                    </div>
                  </button>
                ))}
              </div>
            </>
          ) : (
            <div className="flex justify-center items-center w-full">
              <div className="flex h-10 w-10 animate-spin items-center justify-center rounded-full border-4 border-neutral-300 border-t-transparent">
                <div className="h-6 w-6 animate-spin rounded-full border-4 border-neutral-300 border-t-transparent" />
              </div>
            </div>
          ))}
      </div>
    </div>
  );
}
