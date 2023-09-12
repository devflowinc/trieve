import { Transition } from "solid-headless";
import { Accessor, createEffect, createSignal, For, onCleanup } from "solid-js";
import { CgChevronDoubleRight } from "solid-icons/cg";
import { getThemeMode } from "../Atoms/OnScreenThemeModeController";

const question = "What feature do you want most?";

interface SurveyResult {
  answer: string;
  count: number;
  percentage: number;
}

const answers = [
  {
    id: "A",
    primaryText: "Chat With My Brief",
    secondaryText:
      "Upload your brief and chat with it. Addditionally, when the LLM is providing counterarguments, it will pull evidence from the brief.",
    color: "purple-500",
  },
  {
    id: "B",
    primaryText: "UI Improvements",
    secondaryText:
      'Add buttons like "judge this round" and "summarize clash", etc.',
    color: "blue-500",
  },
  {
    id: "C",
    primaryText: "Voice to Text",
    secondaryText:
      "Use your voice to chat with the LLM so you can get practice actually speaking.",
    color: "green-500",
  },
  {
    id: "D",
    primaryText: "Video Calling Debate Rooms",
    secondaryText:
      "Debate with a friend in a video call and get coaching and feedback from ChatGPT",
    color: "yellow-500",
  },
];

const getGradientStyle = (percentage: number) => {
  const theme = getThemeMode();
  let isDarkMode = false;
  if (theme === "dark") {
    isDarkMode = true;
  } else if (theme === "light") {
    isDarkMode = false;
  } else {
    isDarkMode = window.matchMedia("(prefers-color-scheme: dark)").matches;
  }
  const baseColor = isDarkMode ? "#3C3C3C" : "#DCDCDC";
  const secondaryColor = isDarkMode ? "#505050" : "#F8F8F8";
  return `linear-gradient(90deg, ${baseColor} 0%, ${baseColor} ${percentage}%, ${secondaryColor} ${percentage}%, ${secondaryColor} 100%)`;
};

const setLocalStorageAnswer = (answer: string) => {
  window.localStorage.setItem("surveyAnswer", answer);
};
const getLocalStorageAnswer = () => {
  return window.localStorage.getItem("surveyAnswer") ?? "";
};
const setLocalStorageJoined = (email: string) => {
  window.localStorage.setItem("joined", email);
};
const getLocalStorageJoined = () => {
  return window.localStorage.getItem("joined") ?? "";
};

export const Survey = () => {
  const [showAnswerChoices, setShowAnswerChoices] = createSignal(
    getLocalStorageAnswer() === "",
  );
  const [showWaitlistForm, setShowWaitlistForm] = createSignal(
    getLocalStorageAnswer() !== "" && getLocalStorageJoined() === "",
  );
  const [showThankYou, setShowThankYou] = createSignal(
    getLocalStorageAnswer() !== "" && getLocalStorageJoined() !== "",
  );
  const [showSurveyResults, setShowSurveyResults] = createSignal(
    getLocalStorageAnswer() !== "",
  );

  createEffect(() => {
    void fetch("https://api.arguflow.gg/visits", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        page_visited: "https://arguflow.gg/",
      }),
    });
  });

  return (
    <div>
      <div class="z-0">
        <AnswerChoices
          show={showAnswerChoices}
          setShow={(newShow: boolean) => {
            setShowAnswerChoices(newShow);
            setShowWaitlistForm(!newShow);
            setShowSurveyResults(!newShow);
          }}
        />
      </div>
      <div class="z-10">
        <WaitlistForm
          show={showWaitlistForm}
          setShow={(newShow: boolean) => {
            setShowWaitlistForm(newShow);
            setShowThankYou(!newShow);
          }}
        />
        <ThankYou show={showThankYou} />
        <SurveyResults
          show={showSurveyResults}
          setShow={setShowSurveyResults}
        />
      </div>
    </div>
  );
};

const AnswerChoices = (props: {
  show: Accessor<boolean>;
  setShow: (value: boolean) => void;
}) => {
  const submitSurvey = (answer: string) => {
    void fetch("https://api.arguflow.gg/surveys", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        question,
        answer,
      }),
    });

    setLocalStorageAnswer(answer);
    props.setShow(false);
  };

  return (
    <div class="">
      <Transition
        show={props.show()}
        class="flex h-full w-full flex-col items-center space-y-4"
        enter="transform transition duration-[600ms]"
        enterFrom="opacity-0"
        enterTo="opacity-100"
        leave="transform duration-200 transition ease-in-out"
        leaveFrom="opacity-100 "
        leaveTo="opacity-0 "
      >
        <div class="flex w-full flex-col space-y-2 font-medium">
          <For each={answers}>
            {(answer) => (
              <button
                class="flex w-full flex-row items-center space-x-4 rounded-xl border border-alabaster-600/30 bg-alabaster-500 p-2 text-left shadow-lg focus:bg-alabaster-600 dark:border-none dark:bg-mine-shaft-500 focus:dark:bg-mine-shaft-400"
                onClick={() =>
                  submitSurvey(
                    answer.primaryText + " - " + answer.secondaryText,
                  )
                }
              >
                <div
                  class={`rounded-lg text-${answer.color} border border-${answer.color}`}
                >
                  <p class="align-center h-6 w-6 text-center">{answer.id}</p>
                </div>
                <p class="text-cod-gray dark:text-white">
                  <span class={`text-${answer.color}`}>
                    {answer.primaryText}
                  </span>{" "}
                  {` - `} {answer.secondaryText}
                </p>
              </button>
            )}
          </For>
        </div>
        <span class="hidden border border-purple-500 text-purple-500" />
        <span class="hidden border border-blue-500 text-blue-500" />
        <span class="hidden border border-green-500 text-green-500" />
        <span class="hidden border border-yellow-500 text-yellow-500" />
      </Transition>
    </div>
  );
};

const WaitlistForm = (props: {
  show: Accessor<boolean>;
  setShow: (value: boolean) => void;
}) => {
  const [email, setEmail] = createSignal("");

  const submitEmail = () => {
    void fetch("https://api.arguflow.gg/waitlists", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        email: email(),
      }),
    });

    setLocalStorageJoined(email());
    props.setShow(false);
  };

  return (
    <Transition
      show={props.show()}
      class="flex w-full flex-col items-center space-y-4 px-4"
      enter="transform transition duration-[2000ms]"
      enterFrom="opacity-0 "
      enterTo="opacity-100 "
      leave="transform duration-200 transition ease-in-out"
      leaveFrom="opacity-100 "
      leaveTo="opacity-0 "
    >
      <div class="flex w-full flex-col items-center space-y-4 px-4">
        <p class="text-center text-xl font-medium text-cod-gray dark:text-white">
          Join our waitlist!
        </p>

        <form class="flex w-full flex-row items-center justify-center space-x-2">
          <input
            placeholder="email address"
            type="email"
            value={email()}
            onInput={(e) => setEmail(e.currentTarget.value)}
            class="rounded-xl bg-[#235761] px-3 py-1 text-white placeholder:text-gray-400 dark:bg-[#00DDE7] dark:text-black dark:placeholder:text-gray-600"
          />
          <button
            class="rounded-2xl border border-black p-1 dark:border-white dark:text-white"
            onClick={(e) => {
              e.preventDefault();
              submitEmail();
            }}
          >
            <CgChevronDoubleRight class="text-xl" />
          </button>
        </form>
      </div>
    </Transition>
  );
};

const ThankYou = (props: { show: Accessor<boolean> }) => {
  return (
    <Transition
      show={props.show()}
      class="flex w-full flex-col items-center space-y-4 px-4"
      enter="transform transition duration-[2000ms]"
      enterFrom="opacity-0 "
      enterTo="opacity-100 "
      leave="transform duration-200 transition ease-in-out"
      leaveFrom="opacity-100 "
      leaveTo="opacity-0 "
    >
      <div class="flex w-full flex-col items-center space-y-4 px-4">
        <p class="text-center font-medium text-cod-gray dark:text-white">
          Thank you for joining our waitlist! We're excited to have you on
          board. We'll keep you updated on our progress and reach out as soon as
          we have new features. Stay tuned!
        </p>
      </div>
    </Transition>
  );
};

const SurveyResults = (props: {
  show: Accessor<boolean>;
  setShow: (value: boolean) => void;
}) => {
  const [surveyResults, setSurveyResults] = createSignal<SurveyResult[]>([]);

  createEffect(() => {
    const fetchInterval = setInterval(() => {
      void fetch("https://api.arguflow.gg/surveys/percentages", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          question,
        }),
      })
        .then((res) => res.json())
        .then((data) => {
          setSurveyResults(data);
        });
    }, 1000);

    onCleanup(() => {
      clearInterval(fetchInterval);
    });
  });

  return (
    <Transition
      show={props.show() && surveyResults().length > 0}
      class="mt-12 flex w-full flex-col items-center space-y-4 px-4"
      enter="transform transition duration-[2000ms]"
      enterFrom="opacity-0 "
      enterTo="opacity-100 "
      leave="transform duration-200 transition ease-in-out"
      leaveFrom="opacity-100 "
      leaveTo="opacity-0 "
    >
      <div class="flex w-full flex-col items-center space-y-4 px-4">
        <p class="text-center text-xl font-medium text-cod-gray dark:text-white">
          Survey Results
        </p>
        <For each={surveyResults()}>
          {(result) => (
            <div
              class={`flex w-full flex-row items-center space-x-2 rounded-xl border border-alabaster-600/30 bg-alabaster-500 p-2 shadow-lg dark:border-none dark:bg-mine-shaft-500`}
              style={{
                background: getGradientStyle(result.percentage),
              }}
            >
              <p class="font-medium text-cod-gray dark:text-white">
                {result.answer}
              </p>
              <p class="text-cod-gray dark:text-white">({result.count})</p>
              <p class="font-medium text-cod-gray dark:text-white">
                {result.percentage.toFixed(2)}%
              </p>
            </div>
          )}
        </For>
      </div>
    </Transition>
  );
};
