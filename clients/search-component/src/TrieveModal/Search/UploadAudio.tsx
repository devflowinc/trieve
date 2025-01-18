import React, { useState } from "react";
import { useModalState } from "../../utils/hooks/modal-context";

export const UploadAudio = () => {
  const { mode, query, setAudioBase64 } = useModalState();

  const [isRecording, setIsRecording] = useState(false);
  const [mediaRecorder, setMediaRecorder] = useState<MediaRecorder | null>(
    null,
  );

  const startRecording = async () => {
    try {
      const stream = await navigator.mediaDevices.getUserMedia({ audio: true });

      const recorder = new MediaRecorder(stream);
      const audioChunks: BlobPart[] = [];

      recorder.ondataavailable = (event) => {
        audioChunks.push(event.data);
      };

      recorder.onstop = () => {
        setAudioBase64("");
        const audioBlob = new Blob(audioChunks, { type: "audio/mp3" });
        const reader = new FileReader();
        reader.readAsDataURL(audioBlob);
        reader.onloadend = () => {
          let base64data = reader.result as string;
          base64data = base64data?.split(",")[1];
          setAudioBase64(base64data);
        };
      };

      setMediaRecorder(recorder);
      recorder.start();
      setIsRecording(true);
    } catch (err) {
      console.error("Error accessing microphone:", err);
      alert(
        "Error accessing microphone. Please make sure you have granted microphone permissions.",
      );
    }
  };

  const stopRecording = () => {
    if (mediaRecorder && mediaRecorder?.state !== "inactive") {
      mediaRecorder?.stop();
      setIsRecording(false);
    }
  };

  const toggleRecording = () => {
    if (isRecording) {
      stopRecording();
    } else {
      void startRecording();
    }
  };

  return (
    <div>
      <button
        type="button"
        className={`tv-rounded tv-top-[0.825rem] ${
          mode === "chat"
            ? "tv-right-20"
            : query.length == 0
              ? "tv-right-[11.5rem] inline:tv-right-[2.5rem]"
              : "tv-right-[4rem]"
        } tv-absolute tv-z-20 cursor-pointer ${
          isRecording
            ? "tv-text-red-500"
            : "tv-dark:text-white tv-text-zinc-700"
        }`}
        onClick={(e) => {
          e.preventDefault();
          toggleRecording();
        }}>
        <i className="fa-solid fa-microphone"> </i>
      </button>
    </div>
  );
};
