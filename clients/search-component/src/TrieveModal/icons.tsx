import React, { SVGAttributes } from "react";
export const LoadingIcon = (props: SVGAttributes<SVGElement>) => (
  <svg
    xmlns="http://www.w3.org/2000/svg"
    width="120"
    height="30"
    className={props.className}
    viewBox="0 0 120 30"
  >
    <circle cx="15" cy="15" r="15" fill="currentColor">
      <animate
        attributeName="r"
        begin="0s"
        calcMode="linear"
        dur="0.8s"
        from="15"
        repeatCount="indefinite"
        to="15"
        values="15;9;15"
      ></animate>
      <animate
        attributeName="fill-opacity"
        begin="0s"
        calcMode="linear"
        dur="0.8s"
        from="1"
        repeatCount="indefinite"
        to="1"
        values="1;.5;1"
      ></animate>
    </circle>
    <circle cx="60" cy="15" r="9" fillOpacity="0.3" fill="currentColor">
      <animate
        attributeName="r"
        begin="0s"
        calcMode="linear"
        dur="0.8s"
        from="9"
        repeatCount="indefinite"
        to="9"
        values="9;15;9"
      ></animate>
      <animate
        attributeName="fill-opacity"
        begin="0s"
        calcMode="linear"
        dur="0.8s"
        from="0.5"
        repeatCount="indefinite"
        to="0.5"
        values=".5;1;.5"
      ></animate>
    </circle>
    <circle cx="105" cy="15" r="15" fill="currentColor">
      <animate
        attributeName="r"
        begin="0s"
        calcMode="linear"
        dur="0.8s"
        from="15"
        repeatCount="indefinite"
        to="15"
        values="15;9;15"
      ></animate>
      <animate
        attributeName="fill-opacity"
        begin="0s"
        calcMode="linear"
        dur="0.8s"
        from="1"
        repeatCount="indefinite"
        to="1"
        values="1;.5;1"
      ></animate>
    </circle>
  </svg>
);

export const ArrowIcon = () => {
  return (
    <svg
      className="arrow-link"
      xmlns="http://www.w3.org/2000/svg"
      width="24"
      height="24"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      <path stroke="none" d="M0 0h24v24H0z" fill="none" />
      <path d="M9 6l6 6l-6 6" />
    </svg>
  );
};

export const AIIcon = () => {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width="24"
      height="24"
      viewBox="0 0 24 24"
      fill="currentColor"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      <path stroke="none" d="M0 0h24v24H0z" fill="none" />
      <path d="M16 18a2 2 0 0 1 2 2a2 2 0 0 1 2 -2a2 2 0 0 1 -2 -2a2 2 0 0 1 -2 2zm0 -12a2 2 0 0 1 2 2a2 2 0 0 1 2 -2a2 2 0 0 1 -2 -2a2 2 0 0 1 -2 2zm-7 12a6 6 0 0 1 6 -6a6 6 0 0 1 -6 -6a6 6 0 0 1 -6 6a6 6 0 0 1 6 6z" />
    </svg>
  );
};

export const CheckIcon = () => (
  <svg
    xmlns="http://www.w3.org/2000/svg"
    width="24"
    height="24"
    viewBox="0 0 24 24"
    fill="currentColor"
  >
    <path stroke="none" d="M0 0h24v24H0z" fill="none" />
    <path d="M17 3.34a10 10 0 1 1 -14.995 8.984l-.005 -.324l.005 -.324a10 10 0 0 1 14.995 -8.336zm-1.293 5.953a1 1 0 0 0 -1.32 -.083l-.094 .083l-3.293 3.292l-1.293 -1.292l-.094 -.083a1 1 0 0 0 -1.403 1.403l.083 .094l2 2l.094 .083a1 1 0 0 0 1.226 0l.094 -.083l4 -4l.083 -.094a1 1 0 0 0 -.083 -1.32z" />
  </svg>
);

export const BackIcon = () => (
  <svg
    xmlns="http://www.w3.org/2000/svg"
    width="24"
    height="24"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2"
    strokeLinecap="round"
    strokeLinejoin="round"
    className="search-icon"
  >
    <path stroke="none" d="M0 0h24v24H0z" fill="none" />
    <path d="M15 6l-6 6l6 6" />
  </svg>
);

export const LoadingAIIcon = () => {
  return (
    <svg
      width="23"
      height="23"
      viewBox="0 0 24 24"
      xmlns="http://www.w3.org/2000/svg"
    >
      <path
        d="M12,4a8,8,0,0,1,7.89,6.7A1.53,1.53,0,0,0,21.38,12h0a1.5,1.5,0,0,0,1.48-1.75,11,11,0,0,0-21.72,0A1.5,1.5,0,0,0,2.62,12h0a1.53,1.53,0,0,0,1.49-1.3A8,8,0,0,1,12,4Z"
        fill="currentColor"
      >
        <animateTransform
          attributeName="transform"
          type="rotate"
          dur="0.75s"
          values="0 12 12;360 12 12"
          repeatCount="indefinite"
        />
      </path>
    </svg>
  );
};
export const EscKeyIcon = () => (
  <svg width="15" height="15" aria-label="Escape key" role="img">
    <g
      fill="none"
      stroke="currentColor"
      strokeLinecap="round"
      strokeLinejoin="round"
      strokeWidth="1.2"
    >
      <path d="M13.6167 8.936c-.1065.3583-.6883.962-1.4875.962-.7993 0-1.653-.9165-1.653-2.1258v-.5678c0-1.2548.7896-2.1016 1.653-2.1016.8634 0 1.3601.4778 1.4875 1.0724M9 6c-.1352-.4735-.7506-.9219-1.46-.8972-.7092.0246-1.344.57-1.344 1.2166s.4198.8812 1.3445.9805C8.465 7.3992 8.968 7.9337 9 8.5c.032.5663-.454 1.398-1.4595 1.398C6.6593 9.898 6 9 5.963 8.4851m-1.4748.5368c-.2635.5941-.8099.876-1.5443.876s-1.7073-.6248-1.7073-2.204v-.4603c0-1.0416.721-2.131 1.7073-2.131.9864 0 1.6425 1.031 1.5443 2.2492h-2.956"></path>
    </g>
  </svg>
);

export const ArrowDownKey = () => (
  <svg width="15" height="15" aria-label="Arrow down" role="img">
    <g
      fill="none"
      stroke="currentColor"
      strokeLinecap="round"
      strokeLinejoin="round"
      strokeWidth="1.2"
    >
      <path d="M7.5 3.5v8M10.5 8.5l-3 3-3-3"></path>
    </g>
  </svg>
);

export const ArrowUpIcon = () => (
  <svg width="15" height="15" aria-label="Arrow up" role="img">
    <g
      fill="none"
      stroke="currentColor"
      strokeLinecap="round"
      strokeLinejoin="round"
      strokeWidth="1.2"
    >
      <path d="M7.5 11.5v-8M10.5 6.5l-3-3-3 3"></path>
    </g>
  </svg>
);

export const EnterKeyIcon = () => (
  <svg width="15" height="15" aria-label="Enter key" role="img">
    <g
      fill="none"
      stroke="currentColor"
      strokeLinecap="round"
      strokeLinejoin="round"
      strokeWidth="1.2"
    >
      <path d="M12 3.53088v3c0 1-1 2-2 2H4M7 11.53088l-3-3 3-3"></path>
    </g>
  </svg>
);
