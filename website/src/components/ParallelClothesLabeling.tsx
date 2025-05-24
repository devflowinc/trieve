import React, { useRef, useState } from "react";
import { TrieveSDK } from "trieve-ts-sdk";

const defaultImages = [
  "https://trieve.b-cdn.net/clothesclassifier/defaults/Boncoura-Type-II-Jacket-15oz-Indigo-Denim-01-680x1025.jpg",
  "https://trieve.b-cdn.net/clothesclassifier/defaults/Devoa-EON-Cotton-Silk-Linen-Knit-T-Shirt-Blue-Gray-1-680x1025.jpg",
  "https://trieve.b-cdn.net/clothesclassifier/defaults/Devoa-x-Incarnation-Piece-Dyed-Horsehide-Boots-1-680x1025.jpg",
  "https://trieve.b-cdn.net/clothesclassifier/defaults/fc_virginia_typeII_44_03-1025x680.jpg",
  "https://trieve.b-cdn.net/clothesclassifier/defaults/fh_black_engineer_01-681x1025.jpg",
  "https://trieve.b-cdn.net/clothesclassifier/defaults/Flat_Head_Heavy_Duty_Socks-1-680x1025.jpg",
  "https://trieve.b-cdn.net/clothesclassifier/defaults/Flat-Head-18oz-Indigo-Denim-Jeans-FN-8004-Wide-Tapered-03-680x1025.jpg",
  "https://trieve.b-cdn.net/clothesclassifier/defaults/Flat-Head-Heavyweight-Border-Stripe-T-Shirt-Ivory-Purple-1-680x1025.jpg",
  "https://trieve.b-cdn.net/clothesclassifier/defaults/GOOD_ART_CREAM_DREAM_MONEY_CLIP_FILIGREE-1-680x1025.jpg",
  "https://trieve.b-cdn.net/clothesclassifier/defaults/IRON_HEART_HEAVY_DUTY_COWHIDE_BELT_NICKEL_BROWN-1-680x1025.jpg",
  "https://trieve.b-cdn.net/clothesclassifier/defaults/Iron-Heart-Waist-Bag-21oz-Superblack-Denim-1-680x1025.jpg",
  "https://trieve.b-cdn.net/clothesclassifier/defaults/long-sleeve.jpeg",
  "https://trieve.b-cdn.net/clothesclassifier/defaults/Merz_B._Schwanen_Cashmere_Wool_Beanie_Ribbed_Chestnut_LOBN03.15-1-680x1025.jpg",
  "https://trieve.b-cdn.net/clothesclassifier/defaults/Poten-Japanese-Made-Cap-Adjustable-Black-Denim-1-680x1025.jpg",
  "https://trieve.b-cdn.net/clothesclassifier/defaults/Rick-Owens-DRKSHDW-Ramones-Orange-Milk-Milk-13oz-Overdyed-Denim-1-680x1025.jpg",
  "https://trieve.b-cdn.net/clothesclassifier/defaults/shirt.jpeg",
  "https://trieve.b-cdn.net/clothesclassifier/defaults/Werkstatt_Munchen_Sterling_Silver_Bracelet_Long_Link_Flat-1-680x1025.jpg",
  "https://trieve.b-cdn.net/clothesclassifier/defaults/Werkstatt-Munchen-Sterling-Silver-Ring-Hammered-Oval-Signet-1-680x1025.jpg",
];

// Move default category labels to a constant
const defaultCategoryLabels = [
  "Outerwear",
  "Tops",
  "Bottoms",
  "Footwear",
  "Headwear",
  "Accessories",
  "Denim Jacket",
  "Leather Jacket",
  "T-Shirt",
  "Long Sleeve Shirt",
  "Sweater",
  "Jeans",
  "Socks",
  "Boots",
  "Sneakers",
  "Cap",
  "Beanie",
  "Belt",
  "Bag",
  "Bracelet",
  "Ring",
  "Money Clip",
  "Sports Bra",
];

const labelClothingItem = async (
  itemImageSrc: string,
  abortController: AbortController,
  categoryLabels: string[],
) => {
  const trieveSDK = new TrieveSDK({
    apiKey: "tr-AXWQCahuFoUBgFSByUm7XlATwMTwVPWq",
    datasetId: "3437caf4-667d-46a1-b5cc-7eced57fde71",
  });

  // Build tool_function.parameters from categoryLabels
  const parameters = categoryLabels.map((cat) => ({
    name: cat,
    description: `Set this to true when the item is a or part of the category ${cat.toLowerCase()}`,
    parameter_type: "boolean" as const,
  }));

  const result = await trieveSDK.getToolCallFunctionParams(
    {
      user_message_text: `Please analyze this clothing item and categorize it according to both broad and specific categories.`,
      image_urls: [itemImageSrc],
      tool_function: {
        name: "save_clothing_item",
        description:
          "Categorize clothing items into broad and specific categories",
        parameters,
      },
    },
    abortController.signal,
  );

  return result;
};

interface UploadToBunnyParams {
  fileName: string;
  fileData: Blob;
}

const uploadToBunny = async ({ fileName, fileData }: UploadToBunnyParams) => {
  const BASE_HOSTNAME = "storage.bunnycdn.com";
  const REGION = "ny"; // Your region code
  const HOSTNAME = REGION ? `${REGION}.${BASE_HOSTNAME}` : BASE_HOSTNAME;
  const STORAGE_ZONE = "trieve"; // Your storage zone name
  const uniqueFileName = `clothesclassifier/${crypto.randomUUID()}-${fileName}`;

  const response = await fetch(
    `https://${HOSTNAME}/${STORAGE_ZONE}/${uniqueFileName}`,
    {
      method: "PUT",
      headers: {
        AccessKey: "f77961f6-b66c-4247-853e798ace3a-48b4-4a7f",
        "Content-Type": "application/octet-stream",
      },
      body: fileData, // Raw binary data
    },
  );

  if (!response.ok) {
    throw new Error(`Upload failed: ${response.statusText}`);
  }

  return `https://cdn.trieve.ai/${uniqueFileName}`;
};

interface ParallelClothesLabelingProps {
  categoryLabels?: string[];
}

export const ParallelClothesLabeling = ({
  categoryLabels = defaultCategoryLabels,
}: ParallelClothesLabelingProps) => {
  const [images, setImages] = useState<any[]>([]); // { file, url, status, labels }
  const [dragActive, setDragActive] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  // Tag customization state
  const [customLabels, setCustomLabels] = useState<string[]>(categoryLabels);
  const [newTag, setNewTag] = useState<string>("");
  const [editIdx, setEditIdx] = useState<number | null>(null);
  const [editTag, setEditTag] = useState<string>("");

  // Reset handlers
  const handleResetTags = () => {
    setCustomLabels([...defaultCategoryLabels]);
    setNewTag("");
    setEditIdx(null);
    setEditTag("");
  };
  const handleResetImages = () => {
    setImages([]);
  };

  // Add: Use Demo Images handler
  const handleUseDefaults = async () => {
    setImages([]); // Reset images before loading demo images
    // Show all images as 'labeling' immediately
    const tempImages = defaultImages.map((url) => ({
      file: null,
      url,
      status: "labeling",
      labels: null,
      error: null,
      tempId: crypto.randomUUID(),
    }));
    setImages(tempImages);

    // Start labeling in parallel
    await Promise.all(
      tempImages.map(async (img) => {
        const abortController = new AbortController();
        try {
          const result = await labelClothingItem(
            img.url,
            abortController,
            customLabels,
          );
          setImages((prev) =>
            prev.map((i) =>
              i.tempId === img.tempId
                ? { ...i, status: "done", labels: result.parameters }
                : i,
            ),
          );
        } catch (error) {
          setImages((prev) =>
            prev.map((i) =>
              i.tempId === img.tempId
                ? { ...i, status: "error", error: "Failed to label image" }
                : i,
            ),
          );
        }
      }),
    );
  };

  // Upload handler
  const handleFiles = (files: FileList | null) => {
    if (!files || files.length === 0) return;
    setImages([]); // Reset images before uploading new ones
    Array.from(files).forEach((file) => {
      if (!file.type.startsWith("image/")) {
        console.error("Only image files are supported");
        return;
      }

      const tempId = crypto.randomUUID();
      setImages((prev) => [
        ...prev,
        {
          file,
          url: URL.createObjectURL(file),
          status: "uploading",
          labels: null,
          error: null,
          tempId,
        },
      ]);

      const reader = new FileReader();
      reader.onloadend = async () => {
        try {
          if (!reader.result) return;
          const fileData = new Blob([reader.result], { type: file.type });
          const publicUrl = await uploadToBunny({
            fileName: file.name,
            fileData: fileData,
          });
          setImages((prev) =>
            prev.map((img) =>
              img.tempId === tempId
                ? { ...img, url: publicUrl, status: "labeling" }
                : img,
            ),
          );
          // Start labeling automatically
          const abortController = new AbortController();
          try {
            const result = await labelClothingItem(
              publicUrl,
              abortController,
              customLabels,
            );
            setImages((prev) =>
              prev.map((img) =>
                img.tempId === tempId
                  ? { ...img, status: "done", labels: result.parameters }
                  : img,
              ),
            );
          } catch (error) {
            setImages((prev) =>
              prev.map((img) =>
                img.tempId === tempId
                  ? { ...img, status: "error", error: "Failed to label image" }
                  : img,
              ),
            );
          }
        } catch (error) {
          console.error("Error uploading file:", error);
          setImages((prev) =>
            prev.map((img) =>
              img.tempId === tempId
                ? { ...img, status: "error", error: "Upload failed" }
                : img,
            ),
          );
        }
      };
      reader.readAsArrayBuffer(file);
    });
  };

  // Drag and drop events
  const handleDrag = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    if (e.type === "dragenter" || e.type === "dragover") setDragActive(true);
    else if (e.type === "dragleave") setDragActive(false);
  };

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setDragActive(false);
    handleFiles(e.dataTransfer.files);
  };

  // File input change
  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    handleFiles(e.target.files);
  };

  // Remove image
  const removeImage = (idx: number) => {
    setImages((prev) => prev.filter((_, i) => i !== idx));
  };

  // Export to CSV handler
  const handleExportCSV = () => {
    // Only include images with labels
    const labeledImages = images.filter(
      (img) => img.status === "done" && img.labels,
    );
    if (labeledImages.length === 0) return;
    const headers = ["Image URL", ...customLabels];
    const rows = labeledImages.map((img) => {
      return [
        img.url,
        ...customLabels.map((cat) => (img.labels[cat] ? "TRUE" : "")),
      ];
    });
    const csvContent = [headers, ...rows]
      .map((row) =>
        row.map((v) => `"${String(v).replace(/"/g, '""')}"`).join(","),
      )
      .join("\n");
    const blob = new Blob([csvContent], { type: "text/csv;charset=utf-8;" });
    const url = URL.createObjectURL(blob);
    const link = document.createElement("a");
    link.href = url;
    link.setAttribute("download", "labeled_clothing_images.csv");
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    URL.revokeObjectURL(url);
  };

  // Tag customization handlers
  const handleAddTag = () => {
    if (!newTag.trim()) return;
    if (
      customLabels.some((t) => t.toLowerCase() === newTag.trim().toLowerCase())
    )
      return;
    setCustomLabels((prev) => [...prev, newTag.trim()]);
    setNewTag("");
  };
  const handleRemoveTag = (idx: number) => {
    setCustomLabels((prev) => prev.filter((_, i) => i !== idx));
  };
  const handleEditTag = (idx: number) => {
    setEditIdx(idx);
    setEditTag(customLabels[idx]);
  };
  const handleSaveEditTag = () => {
    if (editIdx === null) return;
    if (!editTag.trim()) return;
    setCustomLabels((prev) =>
      prev.map((t, i) => (i === editIdx ? editTag.trim() : t)),
    );
    setEditIdx(null);
    setEditTag("");
  };
  const handleCancelEdit = () => {
    setEditIdx(null);
    setEditTag("");
  };

  return (
    <div className="mx-auto rounded-xl overflow-hidden mt-2 bg-primary-300 p-4 sm:p-6 max-w-4xl w-full">
      <h2 className="text-3xl sm:text-4xl font-bold mb-2">
        Clothing Image Labeler
      </h2>
      <p className="mb-4 sm:mb-6 text-base sm:text-lg">
        Upload images of clothing and automatically categorize them. Powered by
        Trieve.
      </p>
      {/* Step 1: Tag customization UI */}
      <div className="mb-6 sm:mb-8">
        <div className="mb-2 text-base sm:text-lg font-semibold">
          Step 1. Customize or Select Tags
        </div>
        <div className="mb-2 text-sm">
          Add, edit, or remove tags to define the clothing categories you want
          to use for labeling.
        </div>
        <div className="mb-6 bg-white rounded-xl p-2 sm:p-4 border border-primary-200">
          <div className="flex flex-wrap gap-2 mb-2">
            {customLabels.map((tag, idx) => (
              <div
                key={tag}
                className="flex items-center gap-1 bg-primary-100 border border-primary-300 rounded-full px-2 sm:px-3 py-1 text-sm font-medium shadow-sm"
              >
                {editIdx === idx ? (
                  <>
                    <input
                      className="w-32 px-1 py-0.5 rounded border text-sm mr-1"
                      value={editTag}
                      onChange={(e) => setEditTag(e.target.value)}
                      placeholder="tag"
                    />
                    <button
                      className="text-green-600 text-sm font-bold mr-1"
                      onClick={handleSaveEditTag}
                      title="Save"
                    >
                      ✔
                    </button>
                    <button
                      className="text-gray-400 text-sm font-bold"
                      onClick={handleCancelEdit}
                      title="Cancel"
                    >
                      ✕
                    </button>
                  </>
                ) : (
                  <>
                    <span className="font-mono text-primary-700">{tag}</span>
                    <button
                      className="ml-1 text-blue-500 hover:text-blue-700 text-sm"
                      onClick={() => handleEditTag(idx)}
                      title="Edit"
                    >
                      ✎
                    </button>
                    <button
                      className="ml-1 text-red-500 hover:text-red-700 text-sm"
                      onClick={() => handleRemoveTag(idx)}
                      title="Remove"
                    >
                      ×
                    </button>
                  </>
                )}
              </div>
            ))}
          </div>
          <div className="flex flex-col sm:flex-row gap-2 sm:gap-1 mt-4 sm:mt-6 items-stretch sm:items-center w-full">
            <input
              className="w-full sm:w-32 px-2 py-1 rounded border text-sm"
              value={newTag}
              onChange={(e) => setNewTag(e.target.value)}
              placeholder="New tag"
            />
            <button
              className="bg-primary-500 hover:bg-primary-600 text-white text-sm font-semibold px-3 py-1 rounded-lg sm:ml-2 w-full sm:w-auto"
              onClick={handleAddTag}
              type="button"
            >
              Add Tag
            </button>
            {customLabels.length > 0 && (
              <button
                className="sm:ml-2 bg-gray-100 hover:bg-gray-300 text-gray-900 text-sm font-semibold px-3 py-1 rounded-lg border border-gray-300 w-full sm:w-auto"
                onClick={handleResetTags}
                type="button"
                title="Reset tags to default"
              >
                Reset Tags
              </button>
            )}
          </div>
        </div>
      </div>
      {/* Step 2: Upload or select demo images */}
      <div className="mb-6 sm:mb-8">
        <div className="mb-2 text-base sm:text-lg font-semibold">
          Step 2. Upload Images or Use Demo Images
        </div>
        <div className="mb-4 text-sm">
          Upload your own clothing images or try the demo images below. Images
          will be automatically labeled using your selected tags.
        </div>

        {/* Upload area (now includes demo images and button) */}
        <div className="rounded-xl p-2 sm:p-4 bg-white flex flex-col sm:flex-row gap-4 sm:gap-8 items-stretch sm:items-center justify-center transition-colors duration-200 w-full">
          {/* Demo Images Preview and Button (moved here) */}
          <div
            className="flex flex-col items-center w-full sm:max-w-1/2 group border-4 border-dashed border-gray-300 rounded-xl hover:border-primary-500 p-4 sm:p-8 transition-colors duration-200 hover:cursor-pointer mb-2 sm:mb-0"
            onClick={(e) => {
              e.stopPropagation();
              handleUseDefaults();
            }}
          >
            <div className="flex justify-center gap-2 mb-2">
              {defaultImages.slice(0, 5).map((url, idx) => (
                <img
                  key={idx}
                  src={url}
                  alt={`demo-preview-${idx}`}
                  className="w-10 h-10 sm:w-16 sm:h-16 object-cover rounded opacity-60 group-hover:opacity-100 transition-opacity border border-primary-200 shadow-sm"
                  style={{ pointerEvents: "none" }}
                />
              ))}
            </div>
            <span className="text-sm text-gray-500 group-hover:text-black text-center my-2 sm:my-4">
              No upload needed for demo – click above to try with sample images!
            </span>
            <button
              className="bg-primary-500 group-hover:bg-primary-700 text-white font-semibold py-2 px-4 sm:px-6 rounded-lg shadow mb-2 transition-colors duration-150 w-full sm:w-auto"
              onClick={(e) => {
                e.stopPropagation();
                handleUseDefaults();
              }}
              type="button"
            >
              Use Demo Images
            </button>
          </div>
          {/* Or */}
          <div className="text-center text-gray-500 w-full sm:w-fit flex items-center justify-center">
            <span className="text-sm">or</span>
          </div>
          <div
            className={`flex flex-col items-center group border-4 border-dashed border-gray-300 rounded-xl hover:border-primary-500 p-4 sm:p-8 transition-colors duration-200 w-full sm:max-w-1/2 hover:cursor-pointer h-full ${
              dragActive
                ? "border-primary-500 bg-primary-100"
                : "border-gray-300"
            }`}
            onDragEnter={handleDrag}
            onDragOver={handleDrag}
            onDragLeave={handleDrag}
            onDrop={handleDrop}
            onClick={() => inputRef.current?.click()}
          >
            {/* add an image icon here */}
            <div className="w-12 h-12 sm:w-18 sm:h-18 text-gray-400 group-hover:text-primary-500 mb-1">
              <svg
                xmlns="http://www.w3.org/2000/svg"
                viewBox="0 0 24 24"
                fill="currentColor"
              >
                <path
                  fill-rule="evenodd"
                  clip-rule="evenodd"
                  d="M1.5 6a2.25 2.25 0 0 1 2.25-2.25h16.5A2.25 2.25 0 0 1 22.5 6v12a2.25 2.25 0 0 1-2.25 2.25H3.75A2.25 2.25 0 0 1 1.5 18V6ZM3 16.06V18c0 .414.336.75.75.75h16.5A.75.75 0 0 0 21 18v-1.94l-2.69-2.689a1.5 1.5 0 0 0-2.12 0l-.88.879.97.97a.75.75 0 1 1-1.06 1.06l-5.16-5.159a1.5 1.5 0 0 0-2.12 0L3 16.061Zm10.125-7.81a1.125 1.125 0 1 1 2.25 0 1.125 1.125 0 0 1-2.25 0Z"
                ></path>
              </svg>
            </div>
            <input
              type="file"
              accept="image/*"
              multiple
              ref={inputRef}
              className="hidden"
              onChange={handleChange}
            />
            <p className="text-sm text-gray-500 group-hover:text-black text-center my-2 sm:my-4">
              Click or drag images here to upload - PNG, JPG, and WEBP
              supported.
            </p>
            <button
              className="bg-primary-500 group-hover:bg-primary-700 text-white font-semibold py-2 px-4 sm:px-6 rounded-lg shadow mb-2 transition-colors duration-150 w-full sm:w-auto"
              onClick={(e) => {
                e.preventDefault();
                e.stopPropagation();
                inputRef.current?.click();
              }}
              type="button"
            >
              Try Your Own Images
            </button>
          </div>
        </div>
      </div>
      {/* Image grid */}
      {images.length > 0 && (
        <>
          {/* Export to CSV button */}
          {images.some((img) => img.status === "done" && img.labels) && (
            <div className="flex flex-col sm:flex-row justify-end mb-4 gap-2 w-full">
              <button
                className="bg-primary-500 hover:bg-primary-700 text-white text-sm font-semibold px-3 py-1 rounded-lg w-full sm:w-auto"
                onClick={handleExportCSV}
                type="button"
              >
                Export to CSV
              </button>
              <button
                className="bg-gray-100 hover:bg-gray-300 text-gray-900 text-sm font-semibold px-3 py-1 rounded-lg border border-gray-300 h-10 w-full sm:w-auto"
                onClick={handleResetImages}
                type="button"
                title="Reset all images"
              >
                Reset Images
              </button>
            </div>
          )}
          <div className="mt-6 sm:mt-8 grid grid-cols-1 sm:grid-cols-2 md:grid-cols-4 gap-4 sm:gap-6">
            {images.map((img, idx) => (
              <div
                key={idx}
                className="bg-white rounded-xl shadow p-2 sm:p-3 flex flex-col items-center relative border border-gray-200"
              >
                <img
                  src={img.url}
                  alt={`upload-${idx}`}
                  className="w-24 h-24 sm:w-28 sm:h-28 object-cover rounded mb-2 border border-primary-200"
                />
                <button
                  className="absolute top-2 right-2 text-gray-400 hover:text-red-500"
                  onClick={(e) => {
                    e.stopPropagation();
                    removeImage(idx);
                  }}
                  title="Remove"
                >
                  ×
                </button>
                {img.status === "uploading" && (
                  <div className="mt-2 flex flex-col items-center">
                    <span className="loader border-primary-500 mb-1"></span>
                    <span className="text-sm text-primary-500">
                      Uploading...
                    </span>
                  </div>
                )}
                {img.status === "labeling" && (
                  <div className="mt-2 flex flex-col items-center">
                    <span className="loader border-primary-500 mb-1"></span>
                    <span className="text-sm text-primary-500">
                      Labeling...
                    </span>
                  </div>
                )}
                {img.status === "done" && img.labels && (
                  <div className="mt-2 w-full">
                    <div className="text-sm font-semibold mb-1 text-primary-700">
                      Categories:
                    </div>
                    <div className="flex flex-wrap gap-2">
                      {customLabels.map((cat) =>
                        img.labels[cat] ? (
                          <span
                            key={cat}
                            className="inline-block bg-primary-100 text-primary-700 px-3 py-1 rounded-full text-sm font-medium border border-primary-300 shadow-sm"
                          >
                            {cat}
                          </span>
                        ) : null,
                      )}
                    </div>
                  </div>
                )}
                {img.status === "error" && (
                  <div className="mt-2 text-sm text-red-500">{img.error}</div>
                )}
              </div>
            ))}
          </div>
        </>
      )}
      {/* Loader CSS */}
      <style>{`
        .loader {
          border: 3px solid #e5e7eb;
          border-top: 3px solid #7c3aed;
          border-radius: 50%;
          width: 24px;
          height: 24px;
          animation: spin 1s linear infinite;
        }
        @keyframes spin {
          0% { transform: rotate(0deg); }
          100% { transform: rotate(360deg); }
        }
        /* Prevent horizontal scroll on mobile */
        html, body, #__next, #root {
          max-width: 100vw;
          overflow-x: hidden;
        }
      `}</style>
    </div>
  );
};

export default ParallelClothesLabeling;
