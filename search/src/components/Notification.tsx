import { Accessor, Show, createSignal } from "solid-js";
import {
  FileUploadCompleteNotificationDTO,
  NotificationDTO,
  VerificationDTO,
  isFileUploadCompleteNotificationDTO,
  isVerificationNotificationDTO,
} from "../../utils/apiTypes";
import { VsCheck } from "solid-icons/vs";
import { getLocalTime } from "./CardMetadataDisplay";

export const getTimeIn12HourFormat = (date: Date): string => {
  return date.toLocaleString("en-US", {
    hour12: true,
    year: "2-digit",
    month: "numeric",
    day: "numeric",
    hour: "numeric",
    minute: "numeric",
  });
};

export interface NotificationProps {
  notification: NotificationDTO;
  notifs: Accessor<NotificationDTO[]>;
  markAsRead: (notif: NotificationDTO) => void;
  setState: (state: boolean) => void;
  similarityScoreThreshold: number;
}

export const SingleNotification = (props: NotificationProps) => {
  // eslint-disable-next-line solid/reactivity
  const isVerif = isVerificationNotificationDTO(props.notification);
  // eslint-disable-next-line solid/reactivity
  const isFileUpload = isFileUploadCompleteNotificationDTO(props.notification);

  const [read, setRead] = createSignal(props.notification.user_read);

  const markNotificationAsRead = () => {
    props.markAsRead(props.notification);
    setRead(true);
    props.setState(true);
  };

  return (
    <div
      classList={{
        "focus:bg-neutral-100 rounded-md p-0.5 sm:p-1": true,
        "bg-blue-50 dark:bg-gray-600": !read(),
        "bg-neutral-100 dark:bg-neutral-600": read(),
      }}
    >
      <div class="flex space-x-2 rounded-md px-1 hover:cursor-pointer focus:outline-none dark:hover:bg-none sm:px-2">
        <button
          type="button"
          classList={{
            "flex w-full items-center justify-between rounded p-1 focus:text-black focus:outline-none dark:hover:text-white dark:focus:text-white":
              true,
          }}
        >
          <div class="flex flex-row justify-start space-x-2 py-[16px] text-sm">
            <Show when={isVerif}>
              <span class="text-left">
                {isVerif &&
                (props.notification as VerificationDTO).similarity_score >
                  props.similarityScoreThreshold ? (
                  <a
                    href={`/card/${
                      (props.notification as VerificationDTO).card_uuid
                    }`}
                    onClick={() => {
                      markNotificationAsRead();
                    }}
                  >
                    Your <text class="underline dark:text-acid-500">card</text>{" "}
                    was verified! 🎉
                  </a>
                ) : (
                  <a
                    href={`/card/${
                      (props.notification as VerificationDTO).card_uuid
                    }`}
                    onClick={() => {
                      markNotificationAsRead();
                    }}
                  >
                    Your <text class="underline dark:text-acid-500">card</text>{" "}
                    could not be verified
                  </a>
                )}
              </span>
            </Show>
            <Show when={isFileUpload}>
              <span class="text-left">
                <a
                  // eslint-disable-next-line @typescript-eslint/restrict-template-expressions
                  href={`/collection/${
                    (props.notification as FileUploadCompleteNotificationDTO)
                      .collection_uuid
                  }`}
                  onClick={() => {
                    markNotificationAsRead();
                  }}
                >
                  <span class="underline dark:text-acid-500">
                    {(
                      props.notification as FileUploadCompleteNotificationDTO
                    ).collection_name.replace("Collection for file ", "")}
                  </span>{" "}
                  has been uploaded and processed
                </a>
              </span>
            </Show>
          </div>
        </button>
        <Show when={!props.notification.user_read}>
          <button>
            <VsCheck
              class="mt-1 fill-current"
              onClick={() => {
                markNotificationAsRead();
              }}
            />
          </button>
        </Show>
        <div class="absolute right-2 py-0.5 text-xs">
          {getTimeIn12HourFormat(getLocalTime(props.notification.created_at))}
        </div>
      </div>
    </div>
  );
};
