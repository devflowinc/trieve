import React, { ChangeEvent, useEffect, useRef, useState } from "react";
import { UploadIcon } from "./icons";
import { useModalState } from "../utils/hooks/modal-context";
import { getPresignedUrl, uploadFile } from "../utils/trieve";
import { toBase64 } from "./Search/UploadImage";

export const LargeImageUpload = () => {
  const [isDragging, setIsDragging] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const [file, setFile] = React.useState<File | null>(null);
  const { trieveSDK, setImageUrl, setUploadingImage, setQuery, props } =
    useModalState();

  const handleFileChange = (e: ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (file) {
      setFile(file);
    }
  };

  const handleClick = () => {
    fileInputRef.current?.click();
  };

  useEffect(() => {
    const internalFile = file;
    setFile(null);
    if (internalFile) {
      setQuery("");
      setUploadingImage(true);
      (async () => {
        const data = await toBase64(internalFile);
        const base64File = data
          .split(",")[1]
          .replace(/\+/g, "-")
          .replace(/\//g, "_")
          .replace(/=+$/, "");

        const fileId = await uploadFile(
          trieveSDK,
          internalFile.name,
          base64File,
        );
        const imageUrl = await getPresignedUrl(trieveSDK, fileId);
        setImageUrl(imageUrl);
        setUploadingImage(false);
      })();
    }
  }, [file, trieveSDK]);

  const handleDragOver = (e: React.DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(true);
  };

  const handleDragLeave = (e: React.DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(false);
  };

  const onDrop = (e: React.DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(false);
    const file = e.dataTransfer.files[0];
    if (file) {
      setFile(file);
    }
  };

  return (
    <div
      className={`tv-flex tv-flex-col tv-w-full tv-h-full tv-items-center tv-justify-center tv-gap-4 tv-rounded-lg tv-p-8 tv-bg-zinc-50 tv-border tv-cursor-pointer ${
        isDragging
          ? "tv-bg-zinc-100 tv-border-zinc-300 tv-border-dashed"
          : "tv-border-zinc-200"
      }`}
      onDragOver={handleDragOver}
      onDragLeave={handleDragLeave}
      onDrop={onDrop}
      onClick={handleClick}
    >
      <input
        type="file"
        accept="image/*"
        className="tv-hidden"
        ref={fileInputRef}
        onChange={handleFileChange}
      />
      <UploadIcon className="tv-text-zinc-700 upload-icon" />
      <p className="tv-text-zinc-700 tv-text-sm tv-text-center">
        {props.imageStarterText ||
          "Drag and drop an image here or click to upload"}
      </p>
    </div>
  );
};
