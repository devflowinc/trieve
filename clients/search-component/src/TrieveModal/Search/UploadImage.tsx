import React, { ChangeEvent, useEffect, useRef } from "react";
import { useModalState } from "../../utils/hooks/modal-context";
import { getPresignedUrl, uploadFile } from "../../utils/trieve";

export const UploadImage = () => {
  const fileInputRef = useRef(null);
  const [file, setFile] = React.useState<File | null>(null);
  const { trieveSDK, setImageUrl, setUploadingImage, mode, setQuery } =
    useModalState();

  const handleClick = () => {
    if (!fileInputRef.current) return;
    (fileInputRef.current as HTMLInputElement).click();
  };

  const handleFileChange = (e: ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (file) {
      setFile(file);
    }
  };

  const toBase64 = (file: File) =>
    new Promise<string>((resolve, reject) => {
      const reader = new FileReader();
      reader.readAsDataURL(file);
      reader.onload = () => resolve(reader.result as string);
      reader.onerror = reject;
    });

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

  return (
    <div>
      <button
        onClick={handleClick}
        className={`tv-rounded ${
          mode === "chat" && "tv-right-9 tv-top-[0.825rem] tv-absolute"
        } tv-z-20 tv-dark-text-white tv-text-zinc-700`}
      >
        <i className="fa-solid fa-image"> </i>
      </button>
      <input
        ref={fileInputRef}
        type="file"
        accept="image/*"
        onChange={handleFileChange}
        className="!tv-hidden"
      />
    </div>
  );
};
