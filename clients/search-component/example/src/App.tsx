import { TrieveSearch, TrieveSDK, TrieveModalSearch } from "../../src/index";
import "../../dist/index.css";
import { useState } from "react";
import { IconMoon, IconNext, IconPrevious, IconSun } from "./Icons";
import SyntaxHighlighter from "react-syntax-highlighter";
import { nightOwl } from "react-syntax-highlighter/dist/esm/styles/hljs";

const trieve = new TrieveSDK({
  apiKey: 'tr-UPjrAOp35kGrJWM1IvODj4zVWTdGgQxI',
  datasetId: '4650e231-7857-45aa-beb1-cb52006a2460',
})

export default function App() {
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
            <li>
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

            <TrieveModalSearch trieve={trieve} theme={theme} />

            <div className="mt-8 text-sm rounded overflow-hidden">
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
            <TrieveSearch trieve={trieve} theme={theme} />
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
