import { TrieveSearch, TrieveSDK, TrieveModalSearch } from "../../src/index";
import "../../dist/app.css";

const trieve = new TrieveSDK({
  apiKey: "tr-l1IRx4Jw0iVICiFdf9NroFwmDWQ4CnEd",
  datasetId: "85bdeb65-44ec-4c9c-9d64-725601ad672a",
});
export default function App() {
  return (
    <>
      <div className="container mx-auto my-12 ">
        <h2 className="font-bold text-center py-8">Search Modal Component </h2>
        <div className="grid gap-8 grid-cols-2">
          <div className="mb-8">
            <h2>Search Dark Mode</h2>
            <TrieveModalSearch trieve={trieve} theme="dark" />
          </div>
          <div>
            <h2>Search Light Mode</h2>
            <TrieveModalSearch trieve={trieve} />
          </div>
        </div>
      </div>
      <div className="container mx-auto my-12 ">
        <h2 className="font-bold text-center py-8">
          Search Results Component{" "}
        </h2>
        <div className="grid gap-8 grid-cols-2">
          <div className="mb-8">
            <h2>Search Dark Mode</h2>
            <TrieveSearch trieve={trieve} theme="dark" />
          </div>
          <div>
            <h2>Search Light Mode</h2>
            <TrieveSearch trieve={trieve} />
          </div>
        </div>
      </div>
    </>
  );
}
