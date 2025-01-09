import React, { ChangeEvent, useEffect, useRef } from "react";
import { useModalState } from "../../utils/hooks/modal-context";
import { getPresignedUrl, uploadFile } from "../../utils/trieve";

export const UploadImage = () => {
  const fileInputRef = useRef(null);
  const [file, setFile] = React.useState<File | null>(null);
  const { trieveSDK, setImageUrl } = useModalState();

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
    if (file) {
      (async () => {
        const data = await toBase64(file);
        const base64File = data
          .split(",")[1]
          .replace(/\+/g, "-")
          .replace(/\//g, "_")
          .replace(/=+$/, "");

        const fileId = await uploadFile(trieveSDK, file.name, base64File);
        const imageUrl = await getPresignedUrl(trieveSDK, fileId);
        setImageUrl(imageUrl);
        setFile(null);
      })();
    }
  }, [file, trieveSDK]);

  return (
    <div>
      <button
        onClick={handleClick}
        className="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600"
      >
        Upload Image
      </button>
      <input
        ref={fileInputRef}
        type="file"
        accept="image/*"
        onChange={handleFileChange}
        className="!hidden"
      />
    </div>
  );
};
