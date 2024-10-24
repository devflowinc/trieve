import { TrieveModalSearch } from "../../../src/index";
import "../../../dist/index.css";
import { useState } from "react";
import { IconMoon, IconNext, IconPrevious, IconSun } from "../Icons";
import SyntaxHighlighter from "react-syntax-highlighter";
import { nightOwl } from "react-syntax-highlighter/dist/esm/styles/hljs";
import { createLazyFileRoute } from "@tanstack/react-router";

export const Route = createLazyFileRoute("/")({
  component: Home,
});

export default function Home() {
  const [theme, setTheme] = useState<"light" | "dark">("light");
  const [component, setComponent] = useState(0);
  return (
    <>
      <div
        className={`p-12 flex flex-col items-center justify-center w-screen h-screen relative ${
          theme === "dark" ? "bg-zinc-900 text-zinc-50" : ""
        }`}
      >
        <div className="absolute top-6 right-6">
          <ul>
            <li key="theme">
              <button
                onClick={() => setTheme(theme === "light" ? "dark" : "light")}
              >
                {theme === "light" ? <IconMoon /> : <IconSun />}
              </button>
            </li>
          </ul>
        </div>
        {component === 0 ? (
          <>
            <h2 className="font-bold text-center py-8">
              Search Modal Component{" "}
            </h2>

            <TrieveModalSearch
              debounceMs={50}
              defaultSearchMode="search"
              apiKey="tr-zpPVGUq18FxOCmXgLfqGbmDOY4UMW00r"
              datasetId="4538ad9f-47cf-44d4-8a14-7c111f9558a9"
              problemLink="mailto:help@trieve.ai?subject="
              theme={theme}
              tags={[
                {
                  tag: "openapi-route",
                  label: "API Routes",
                  icon: () => (
                    <svg
                      xmlns="http://www.w3.org/2000/svg"
                      width="24"
                      height="24"
                      viewBox="0 0 24 24"
                      fill="none"
                      stroke="currentColor"
                      strokeWidth="2"
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      className="w-3 h-3"
                    >
                      <path stroke="none" d="M0 0h24v24H0z" fill="none" />
                      <path d="M14 3v4a1 1 0 0 0 1 1h4" />
                      <path d="M17 21h-10a2 2 0 0 1 -2 -2v-14a2 2 0 0 1 2 -2h7l5 5v11a2 2 0 0 1 -2 2z" />
                      <path d="M9 14v.01" />
                      <path d="M12 14v.01" />
                      <path d="M15 14v.01" />
                    </svg>
                  ),
                },
                {
                  tag: "blog",
                  label: "Blog",
                  icon: () => (
                    <svg
                      xmlns="http://www.w3.org/2000/svg"
                      width="24"
                      height="24"
                      viewBox="0 0 24 24"
                      strokeWidth="2"
                      fill="none"
                      stroke="currentColor"
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      className="w-3 h-3"
                    >
                      <path stroke="none" d="M0 0h24v24H0z" fill="none" />
                      <path d="M22 12.54c-1.804 -.345 -2.701 -1.08 -3.523 -2.94c-.487 .696 -1.102 1.568 -.92 2.4c.028 .238 -.32 1 -.557 1h-14c0 5.208 3.164 7 6.196 7c4.124 .022 7.828 -1.376 9.854 -5c1.146 -.101 2.296 -1.505 2.95 -2.46z" />
                      <path d="M5 10h3v3h-3z" />
                      <path d="M8 10h3v3h-3z" />
                      <path d="M11 10h3v3h-3z" />
                      <path d="M8 7h3v3h-3z" />
                      <path d="M11 7h3v3h-3z" />
                      <path d="M11 4h3v3h-3z" />
                      <path d="M4.571 18c1.5 0 2.047 -.074 2.958 -.78" />
                      <path d="M10 16l0 .01" />
                    </svg>
                  ),
                },
              ]}
              defaultSearchQueries={[
                "How to create a chunk?",
                "Does Trieve use a re-ranker?",
                "Sending click events",
              ]}
              defaultAiQuestions={[
                "What is Trieve?",
                "How to perform autocomplete search?",
                "How do I install the TS SDK?",
              ]}
              brandLogoImgSrcUrl="https://cdn.trieve.ai/trieve-logo.png"
              brandName="Trieve"
              brandColor="#b557c5"
              allowSwitchingModes={true}
              useGroupSearch={false}
              responsive={false}
            />

            <div className="mt-8 text-sm rounded overflow-hidden max-w-[100vw]">
              <SyntaxHighlighter language={"jsx"} style={nightOwl}>
                {`<TrieveModalSearch trieve={trieve} theme="${theme}" /> `}
              </SyntaxHighlighter>
            </div>
          </>
        ) : (
          <>
            <h2 className="font-bold text-center py-8">
              Search Results Component
            </h2>
            <h2 className="font-bold text-center py-8">
              This was removed, see
              https://github.com/devflowinc/trieve/pull/2613
            </h2>
          </>
        )}

        <ul className="absolute top-1/2 -translate-y-1/2 w-full">
          {component > 0 ? (
            <li className="left-6 absolute">
              <button onClick={() => setComponent(0)}>
                <IconPrevious />
              </button>
            </li>
          ) : (
            <li className="right-6 absolute">
              <button onClick={() => setComponent(1)}>
                <IconNext />
              </button>
            </li>
          )}
        </ul>
      </div>
    </>
  );
}
