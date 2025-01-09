import React from "react";
import { useModalState } from "../utils/hooks/modal-context";

const ImagePreview = ({
  imageUrl,
  isUploading,
  active,
}: {
  imageUrl: string;
  isUploading: boolean;
  active?: boolean;
}) => {
  const { setImageUrl } = useModalState();
  return (
    <>
      {isUploading ? (
        <div className="relative w-fit">
          <div className={`max-h-96 max-w-32 ${active ? "border p-2" : ""}`}>
            <div className="animate-spin h-8 w-8 border-4 border-blue-500 rounded-full border-t-transparent"></div>
          </div>
        </div>
      ) : imageUrl ? (
        <div className="relative w-fit">
          {active && (
            <button
              onClick={() => {
                setImageUrl("");
              }}
              className={`flex items-center justify-center absolute -right-3 -top-2 rounded-full bg-zinc-500 `}
            >
              <i className="fa-solid fa-close p-1 w-4 h-4 cursor-pointer z-10"></i>
            </button>
          )}
          <div className={`max-h-96 max-w-32 ${active ? "border p-2" : ""}`}>
            <img src={imageUrl} alt="" />
          </div>
        </div>
      ) : null}
    </>
  );
};

export default ImagePreview;
