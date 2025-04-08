import React, { useState } from "react";
import { useModalState } from "../../utils/hooks/modal-context";
import { cn } from "../../utils/styles";
import { StopSquareIcon, MicIcon } from "../icons";
import { motion } from "motion/react";

export const UploadAudio = () => {
  const { props, mode, setAudioBase64, isRecording, setIsRecording } =
    useModalState();

  const [mediaRecorder, setMediaRecorder] = useState<MediaRecorder | null>(
    null,
  );

  const startRecording = async () => {
    try {
      const stream = await navigator.mediaDevices.getUserMedia({
        audio: true,
      });

      const isFirefox = navigator.userAgent.toLowerCase().includes("firefox");

      const recorder = new MediaRecorder(stream, {
        mimeType: isFirefox ? "audio/webm" : "audio/mp4",
      });
      const audioChunks: BlobPart[] = [];

      recorder.ondataavailable = (event) => {
        audioChunks.push(event.data);
      };

      recorder.onstop = () => {
        stream.getTracks().forEach((track) => track.stop());
        setAudioBase64("");
        const audioBlob = new Blob(audioChunks, {
          type: isFirefox ? "audio/webm" : "audio/mp4",
        });
        const reader = new FileReader();
        reader.readAsDataURL(audioBlob);
        reader.onloadend = () => {
          let base64data = reader.result as string;
          base64data = base64data?.split(",")[1];
          setAudioBase64(base64data);
        };
      };

      setMediaRecorder(recorder);
      recorder.start(100);
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
        className={cn(
          "tv-rounded tv-z-20 tv-cursor-pointer tv-bg-transparent mic-button-container",
          mode === "chat" && "tv-right-[60px] tv-top-[17px] tv-absolute",
        )}
        onClick={(e) => {
          e.preventDefault();
          toggleRecording();
        }}
        disabled={props.previewTopicId != undefined}
      >
        {isRecording ? (
          <motion.div
            animate={{
              opacity: [1, 0.7, 1],
            }}
            transition={{
              duration: 2,
              repeat: Infinity,
              ease: "easeInOut",
            }}
          >
            <StopSquareIcon
              width={16}
              height={16}
              className={"tv-text-red-500"}
            />
          </motion.div>
        ) : (
          <MicIcon
            width={16}
            height={16}
            className="tv-text-zinc-700 tv-dark-text-white"
          />
        )}
      </button>
    </div>
  );
};
