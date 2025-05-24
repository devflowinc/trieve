import React, { useState, useEffect } from "react";
import PriceSlider, { formatValue } from "./PriceSlider";

interface Usage {
  // Search & Vector DB
  chunksStored: number;
  searchesSent: number;
  messagesSent: number;
  writesPerMo: number;
  // Data Ingestion
  pagesCrawled: number;
  ocrPages: number;
  fileStoredGb: number;
  // Analytics
  analyticsEvents: number;
  // Platform fee
  datasets: number;
  users: number;
  componentLoads: number;
}

interface Assumptions {
  averageSparseSize: number; // Used for writes per month and vector storage
  vectorDimension: number; // Used for writes per month and vector storage
  averagePayloadSizeBytes: number; // Used for writes per month
  tokensPerSearch: number; // Used for searches per month
  tokensPerMessage: number; // Used for messages per month
  tokensPerChunk: number; // Used for writes per month
  tokensPerPage: number; // Used for pdf2md
}

interface Costs {
  name: string;
  price: number;
}

interface CostItem {
  name: string;
  tooltip: string;
  total: number;
  breakdown: Costs[];
  amount: number;
  unit: string;
}

const defaultUsage: Usage = {
  chunksStored: 1_000,
  messagesSent: 1_000,
  searchesSent: 1_000,
  fileStoredGb: 5,
  pagesCrawled: 10,
  writesPerMo: 10_000,
  analyticsEvents: 1_000_000,
  ocrPages: 10,
  componentLoads: 1_000,
  datasets: 1,
  users: 1,
};

const defaultAssumptions: Assumptions = {
  averageSparseSize: 256, // Used for writes per month and vector storage
  vectorDimension: 1536, // Used for writes per month and vector storage
  averagePayloadSizeBytes: 4096, // Used for writes per month
  tokensPerSearch: 12.65, // Used for searches per month
  tokensPerMessage: 26.3, // Used for messages per month
  tokensPerChunk: 300, // Used for writes per month
  tokensPerPage: 800, // Used for pdf2md
};

const calculateUsageCost = (usage: Usage, assumptions: Assumptions) => {
  // First 1000 chunks are free.
  const storagePerChunk =
    assumptions.averageSparseSize * 4 +
    assumptions.vectorDimension * 4 +
    assumptions.averagePayloadSizeBytes;
  const storageUsedBytes =
    Math.max(0, usage.chunksStored - 1000) * storagePerChunk;
  // First 10,000 searches are free.
  const searchTokens = usage.searchesSent * assumptions.tokensPerSearch;
  const searchTokensCalc = Math.max(0, searchTokens - 100_000);
  // First 1,000 messages are free.
  const messageTokens = usage.messagesSent * assumptions.tokensPerMessage;
  const messageTokensCalc =
    Math.max(0, messageTokens - 263_000) * assumptions.tokensPerMessage;
  // First 10,000 writes are free.
  const writeTokens = usage.writesPerMo * assumptions.tokensPerChunk;
  const writeTokensCalc =
    Math.max(0, usage.writesPerMo - 100_000) * assumptions.tokensPerChunk;

  // assuming 4KB per chunk
  const ingestionStorageGB =
    (usage.writesPerMo * assumptions.averagePayloadSizeBytes) / 1_000_000_000;

  let costs = {
    // Vector / Search costs
    vectorStorageGB: (storagePerChunk * usage.chunksStored) / 1_000_000_000,
    vectorCost: (storageUsedBytes / 1_000_000_000) * 12.07,
    searchTokens,
    searchCost: (searchTokensCalc * 0.028) / 1_000_000,
    messageTokens,
    messageCost: (messageTokensCalc * 3.528) / 1_000_000,
    writeTokens,
    writesCost: (writeTokensCalc * 0.028) / 1_000_000,
    datasetCost: Math.max(0, usage.datasets - 2) * 0.05,
    scrapeCost: Math.max(0, usage.pagesCrawled - 10) * 0.00086,
    ingestionStorageGB,
    ingestCostGb: Math.max(ingestionStorageGB - 1, 0) * 2,
    analyticsCost: Math.max(0, usage.analyticsEvents - 1_000_000) * 0.0001,
    fileStorageCost: Math.max(0, usage.fileStoredGb - 10) * 0.046,
    ocrCost: Math.max(0, usage.ocrPages - 10) * 0.01,
    componentCost: Math.max(0, usage.componentLoads - 1_000) * 0.01,
    userCost: Math.max(0, usage.users - 5) * 5,
    platformFee: 0,
    total: 0,
  };

  // if any costs is greater than 0, add base platform platformFee
  costs.total =
    costs.ingestCostGb +
    costs.vectorCost +
    costs.searchCost +
    costs.messageCost +
    costs.writesCost +
    costs.datasetCost +
    costs.scrapeCost +
    costs.analyticsCost +
    costs.fileStorageCost +
    costs.ocrCost +
    costs.componentCost +
    costs.userCost;
  if (costs.total > 0) {
    costs.total += 5.0;
    costs.platformFee = 5.0;
  }
  return costs;
};

const currencyFormatter = Intl.NumberFormat(navigator.language || "en-US", {
  style: "currency",
  currency: "USD",
});
// Format currency
const formatCurrency = (amount: number) => {
  return currencyFormatter.format(amount);
};

const PricingCalculator = () => {
  const pullUsageFromPath = () => {
    const searchParams = new URLSearchParams(window.location.search);

    return {
      searchesSent: parseInt(
        searchParams.get("searchesSent") ||
          defaultUsage.searchesSent.toString(),
      ),
      messagesSent: parseInt(
        searchParams.get("messagesSent") ||
          defaultUsage.messagesSent.toString(),
      ),
      writesPerMo: parseInt(
        searchParams.get("writesPerMo") || defaultUsage.writesPerMo.toString(),
      ),
      chunksStored: parseInt(
        searchParams.get("chunksStored") ||
          defaultUsage.chunksStored.toString(),
      ),
      datasets: parseInt(
        searchParams.get("datasets") || defaultUsage.datasets.toString(),
      ),
      pagesCrawled: parseInt(
        searchParams.get("pagesCrawled") ||
          defaultUsage.pagesCrawled.toString(),
      ),
      analyticsEvents: parseInt(
        searchParams.get("analyticsEvents") ||
          defaultUsage.analyticsEvents.toString(),
      ),
      fileStoredGb: parseInt(
        searchParams.get("fileStoredGb") ||
          defaultUsage.fileStoredGb.toString(),
      ),
      ocrPages: parseInt(
        searchParams.get("ocrPages") || defaultUsage.ocrPages.toString(),
      ),
      componentLoads: parseInt(
        searchParams.get("componentLoads") ||
          defaultUsage.componentLoads.toString(),
      ),
      users: parseInt(
        searchParams.get("users") || defaultUsage.users.toString(),
      ),
    };
  };

  const pullAssumptionsFromPath = () => {
    const searchParams = new URLSearchParams(window.location.search);

    return {
      ...defaultAssumptions,
      vectorDimension: parseInt(
        searchParams.get("vectorDimension") ||
          defaultAssumptions.vectorDimension.toString(),
      ),
      averagePayloadSizeBytes: parseInt(
        searchParams.get("averagePayloadSizeBytes") ||
          defaultAssumptions.averagePayloadSizeBytes.toString(),
      ),
    };
  };

  const [usage, setUsage] = useState<Usage>(pullUsageFromPath());
  const [assumptionsUsed, setAssumptionsUsed] = useState<Assumptions>(
    pullAssumptionsFromPath(),
  );

  // New state to track individual cost items
  const [costItems, setCostItems] = useState<CostItem[]>([]);

  useEffect(() => {
    const cost = calculateUsageCost(usage, assumptionsUsed);

    setCostItems([
      {
        name: "Vectors",
        tooltip: "Vector dimension and payload size may vary",
        total: cost.vectorCost,
        breakdown: [],
        amount: cost.vectorStorageGB,
        unit: "GBs stored",
      },
      {
        name: "Queries",
        tooltip: "Assuming 300 average tokens / chunk",
        total: cost.searchCost + cost.messageCost,
        breakdown: [
          {
            name: "Search",
            price: cost.searchCost,
          },
          {
            name: "Messages",
            price: cost.messageCost,
          },
        ],
        amount: cost.searchTokens + cost.messageTokens,
        unit: "tokens processed",
      },
      {
        name: "Writes",
        tooltip: "Assuming 300 average tokens / chunk",
        total: cost.writesCost,
        breakdown: [
          {
            name: formatValue(cost.writeTokens) + " Token Writes",
            price: cost.writesCost,
          },
          {
            name:
              cost.ingestionStorageGB.toLocaleString("default", {
                minimumFractionDigits: 2,
                maximumFractionDigits: 2,
              }) + " GB(s) ingested",
            price: cost.ingestCostGb,
          },
        ],
        amount: 0,
        unit: "",
      },
      {
        name: "Web Scrape",
        tooltip: "Number of pages crawled",
        total: cost.scrapeCost,
        breakdown: [],
        amount: usage.pagesCrawled,
        unit: "Pages crawled",
      },
      {
        name: "Files",
        tooltip: "",
        total: cost.fileStorageCost + cost.ocrCost,
        breakdown: [
          {
            name: `File OCR (${usage.ocrPages} pages)`,
            price: cost.ocrCost,
          },
          {
            name: `File Storage (${usage.fileStoredGb} GB)`,
            price: cost.fileStorageCost,
          },
        ],
        amount: 0,
        unit: "",
      },
      {
        name: "Platform",
        tooltip: "Number of component loads",
        total:
          cost.componentCost +
          cost.analyticsCost +
          cost.datasetCost +
          cost.userCost +
          cost.platformFee,
        breakdown: [
          {
            name: `Base Platform fee`,
            price: cost.platformFee,
          },
          {
            name: `${formatValue(usage.analyticsEvents)} Analytics Events`,
            price: cost.analyticsCost,
          },
          {
            name: `${usage.datasets} Datasets`,
            price: cost.datasetCost,
          },
          {
            name: `${usage.users} Users`,
            price: cost.userCost,
          },
        ],
        amount: 0,
        unit: "",
      },
    ]);

    setTotal({
      price: cost.total,
      usageDiscount: 0,
    });
  }, [usage, assumptionsUsed]);

  const [total, setTotal] = useState({
    price: 0,
    usageDiscount: 0,
  });
  const [activeTab, setActiveTab] = useState("search"); // Tab for product categories

  // Handle usage input change
  const handleUsageChange = (product: string, value: string) => {
    setUsage((prev) => ({
      ...prev,
      [product]: parseInt(value, 10) || 0,
    }));

    // Update URL search params
    const searchParams = new URLSearchParams(window.location.search);
    searchParams.set(product, value);
    window.history.replaceState(
      null,
      "",
      `${window.location.pathname}?${searchParams.toString()}`,
    );
  };

  return (
    <div className="mx-auto text-primary-900 rounded-xl overflow-hidden mt-2">
      <h2 className="text-5xl">Trieve API Pricing Calculator</h2>
      <p className="mt-4 pl-2 text-lg">
        Note: See our{" "}
        <a
          href="https://apps.shopify.com/trieve"
          className="text-primary-500 hover:text-primary-900 underline"
        >
          Shopify app
        </a>{" "}
        for Ecommerce and site-search pricing. The below calculator is only
        applicable to our API product. <br /><br /> Self-hosting is available on
        request and also priced per usage.
      </p>
      {/* Main tab navigation */}
      <div className="border-b border-gray-800 mt-4">
        <div className="flex overflow-x-auto scrollbar-hide text-gray-400">
          <button
            onClick={() => setActiveTab("search")}
            className={`pr-1 py-2 mr-2 md:mr-6 text-left text-base font-medium whitespace-nowrap hover:text-primary-600 ${activeTab === "search" ? "text-primary-500 border-b-2 border-primary-500" : ""}`}
          >
            Storage
          </button>
          <button
            onClick={() => setActiveTab("ingestion")}
            className={`pr-1 py-2 mr-2 md:mr-6 text-left text-base font-medium whitespace-nowrap hover:text-primary-600 ${activeTab === "ingestion" ? "text-primary-500 border-b-2 border-primary-500" : ""}`}
          >
            Ingestion
          </button>
          <button
            onClick={() => setActiveTab("infrastructure")}
            className={`pr-1 py-2 mr-2 md:mr-6 text-left text-base font-medium whitespace-nowrap hover:text-primary-600 ${activeTab === "infrastructure" ? "text-primary-500 border-b-2 border-primary-500" : ""}`}
          >
            Platform + Analytics
          </button>
        </div>
      </div>

      <div className="flex flex-col gap-4 md:gap-8 md:flex-row">
        {/* Usage calculator */}
        <div className="w-full md:w-2/3 py-4 md:py-8 border-b md:border-b-0 border-gray-800">
          <div className="space-y-10">
            {/* Search Tab Content */}
            {activeTab === "search" && (
              <>
                <div className="flex flex-col gap-2 mb-6">
                  <h3 className="text-lg font-medium">Chunks Stored</h3>
                  <p>First 1,000 chunks (11MB) are free!</p>
                  <div>
                    <div className="relative">
                      <PriceSlider
                        min={1000}
                        max={1_000_000_000}
                        markers={[
                          1_000, 10_000, 100_000, 1_000_000, 10_000_000,
                          100_000_000, 1_000_000_000,
                        ]}
                        defaultValue={usage.chunksStored}
                        beforeValueText=""
                        afterValueText="chunks"
                        onChange={(value: any) =>
                          handleUsageChange("chunksStored", value)
                        }
                      />
                    </div>
                  </div>
                </div>
                <div className="flex flex-col gap-2 mb-6">
                  <h3 className="text-lg font-medium">Searches / month</h3>
                  <p>First 3M tokens are free! (~8,000 searches)</p>
                  <PriceSlider
                    min={1_000}
                    max={100_000_00000}
                    markers={[
                      1_000, 10_000, 100_000, 1_000_000, 1_000_000, 10_000_000,
                      100_000_000, 100_000_0000, 100_000_00000,
                    ]}
                    defaultValue={usage.searchesSent}
                    beforeValueText=""
                    afterValueText="searches"
                    onChange={(value: any) =>
                      handleUsageChange("searchesSent", value)
                    }
                  />
                </div>
                <div className="flex flex-col gap-2 mb-6">
                  <h3 className="text-lg font-medium">Messages / month</h3>
                  <p>First 263,000 message tokens free! (1,000 messages)</p>
                  <div>
                    <div className="relative">
                      <PriceSlider
                        min={1_000}
                        max={100_000_000}
                        markers={[
                          1_000, 10_000, 100_000, 1_000_000, 10_000_000,
                          100_000_000,
                        ]}
                        defaultValue={usage.messagesSent}
                        beforeValueText=""
                        afterValueText="messages"
                        onChange={(value: any) =>
                          handleUsageChange("messagesSent", value)
                        }
                      />
                    </div>
                  </div>
                </div>

                <div className="flex flex-col gap-2 mb-6">
                  <h3 className="text-lg font-medium">Writes / month</h3>
                  <p>First 100,000 writes are free!</p>
                  <PriceSlider
                    min={100_000}
                    max={10_000000}
                    markers={[
                      100_000, 100_000, 200_000, 500_000, 1_000_000, 3_000_000,
                      5_000_000, 10_000000,
                    ]}
                    defaultValue={usage.writesPerMo}
                    beforeValueText=""
                    afterValueText="writes"
                    onChange={(value: string) =>
                      handleUsageChange("writesPerMo", value)
                    }
                  />
                </div>
              </>
            )}
            {/* Data Ingestion Tab Content */}
            {activeTab === "ingestion" && (
              <>
                <div className="flex flex-col gap-2 mb-6">
                  <h3 className="text-lg font-medium">Website Crawler</h3>
                  <p>First 10 pages are free!</p>
                  <div>
                    <div className="relative">
                      <PriceSlider
                        min={10}
                        max={100000}
                        markers={[
                          10, 100, 1000, 10000, 100000, 1000000, 10000000,
                        ]}
                        defaultValue={usage.pagesCrawled}
                        beforeValueText="Crawling"
                        afterValueText="web pages per month"
                        onChange={(value: any) =>
                          handleUsageChange("pagesCrawled", value)
                        }
                      />
                    </div>
                  </div>
                </div>

                <div className="flex flex-col gap-2 mb-6">
                  <h3 className="text-lg font-medium">OCR Processing</h3>
                  <p>First 100 pages are free!</p>
                  <div>
                    <div className="relative">
                      <PriceSlider
                        min={10}
                        max={100000}
                        markers={[10, 1000, 10000, 100000, 1000000]}
                        defaultValue={usage.ocrPages}
                        beforeValueText=""
                        afterValueText="pages"
                        onChange={(value: any) =>
                          handleUsageChange("ocrPages", value)
                        }
                      />
                    </div>
                  </div>
                </div>
                <div className="flex flex-col gap-2 mb-6">
                  <h3 className="text-lg font-medium">File Storage</h3>
                  <p>First 10GB are free!</p>
                  <div>
                    <div className="relative">
                      <PriceSlider
                        min={10}
                        max={1000000}
                        markers={[10, 100, 1000, 10000, 100000, 1000000]}
                        defaultValue={usage.fileStoredGb}
                        beforeValueText=""
                        afterValueText="GB(s) of files"
                        onChange={(value: any) =>
                          handleUsageChange("fileStoredGb", value)
                        }
                      />
                    </div>
                  </div>
                </div>
              </>
            )}

            {/* Infrastructure Tab Content */}
            {activeTab === "infrastructure" && (
              <>
                <div className="flex flex-col gap-2 mb-6">
                  <h3 className="text-lg font-medium">Datasets</h3>
                  <p>First 2 datasets free!</p>
                  <div className="relative">
                    <PriceSlider
                      min={1}
                      max={10_000}
                      markers={[10, 15, 50, 100, 1_000, 5_000, 10_000]}
                      defaultValue={usage.datasets}
                      beforeValueText="Using"
                      afterValueText="datasets"
                      onChange={(value: any) =>
                        handleUsageChange("datasets", value)
                      }
                    />
                  </div>
                </div>

                <div className="flex flex-col gap-2 mb-6">
                  <h3 className="text-lg font-medium">Users</h3>
                  <p>First 5 users free!</p>
                  <div className="relative">
                    <PriceSlider
                      min={1}
                      max={20}
                      markers={[1, 2, 3, 5, 7, 10, 15, 20]}
                      defaultValue={usage.users}
                      beforeValueText="With"
                      afterValueText="users"
                      onChange={(value: any) =>
                        handleUsageChange("users", value)
                      }
                    />
                  </div>
                </div>

                <div className="flex flex-col gap-2 mb-6">
                  <h3 className="text-lg font-medium">Analytics</h3>
                  <p>First 1M events free!</p>
                  <div>
                    <div className="relative">
                      <PriceSlider
                        min={1_000_000}
                        max={100_000_000_000}
                        markers={[
                          100_000, 1_000_000, 1_000_000, 10_000_000,
                          100_000_000, 1_000_000_000, 10_000_000_000,
                          100_000_000_000,
                        ]}
                        defaultValue={usage.analyticsEvents}
                        beforeValueText="Tracking"
                        afterValueText="analytics events"
                        onChange={(value: any) =>
                          handleUsageChange("analyticsEvents", value)
                        }
                      />
                    </div>
                  </div>
                </div>
              </>
            )}
          </div>
        </div>

        {/* Line items section */}
        <div className="w-full md:w-1/3 py-8">
          <h3 className="text-3xl mb-6 flex items-center">Line Items</h3>

          {/* Display active cost items */}
          <div className="space-y-4 overflow-y-auto pr-2 bg-primary-300 rounded-lg p-4 md:p-8">
            {costItems.map((item, index) => (
              <div key={index} className="border-b border-gray-700 pb-4">
                <div className="flex justify-between mb-1">
                  <span className="text-xl font-medium">{item.name}</span>
                  {item.total > 0 ? (
                    <span className="">
                      <span className="font-medium">
                        {formatCurrency(item.total)}
                      </span>
                    </span>
                  ) : (
                    <div>FREE</div>
                  )}
                </div>
                {item.amount > 0 && (
                  <div className="text-sm text-gray-700 mb-3">
                    {formatValue(item.amount)} {item.unit}
                  </div>
                )}
                {item.total > 0 && item.breakdown.length > 0 && (
                  <p>Breakdown</p>
                )}
                {item.total > 0 &&
                  item.breakdown.map((breakdownItem, index) => (
                    <div className="flex justify-between mb-1">
                      <span className="ml-4 text-sm font-medium">
                        - {breakdownItem.name}
                      </span>
                      <div className="flex flex-col justify-end items-end">
                        <span className="text-sm font-medium">
                          {formatCurrency(breakdownItem.price)}
                        </span>
                      </div>
                    </div>
                  ))}
              </div>
            ))}
            <div className="flex justify-between mb-1">
              <span className="text-xl font-medium">Total</span>
              <div className="text-3xl font-bold mt-4 md:mt-0">
                {formatCurrency(total.price + total.usageDiscount)}{" "}
                <span className="text-sm font-normal text-gray-600 ">
                  / month
                </span>
              </div>
            </div>

            {costItems.length === 0 && (
              <div className="text-gray-400 italic">
                No charges yet. Adjust the sliders to see cost breakdown.
              </div>
            )}
          </div>
        </div>
      </div>

      {/* Total section */}
      <div className="w-full md:w-[calc(66%_-_1rem)] rounded-xl mt-8 p-6 bg-white">
        <div className="flex flex-col md:flex-row justify-between items-center">
          <div>
            <h3 className="text-xl font-semibold">Estimated total</h3>
            <p className="text-base text-gray-600 ">
              for all products & add-ons
            </p>
          </div>
          <div className="flex flex-col align-end">
            <div className="text-3xl font-bold mt-4 md:mt-0">
              {formatCurrency(total.price + total.usageDiscount)}{" "}
              <span className="text-sm font-normal text-gray-600 ">
                / month
              </span>
            </div>
          </div>
          <div className="mt-4 md:mt-0">
            <a href="https://dashboard.trieve.ai" className="button primary">
              Get started free
            </a>
          </div>
        </div>
      </div>
    </div>
  );
};

export default PricingCalculator;
