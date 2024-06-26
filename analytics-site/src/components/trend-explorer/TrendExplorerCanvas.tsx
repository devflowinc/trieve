import { createEffect, createSignal, onMount } from "solid-js";
import {
  Composite,
  Engine,
  Render,
  Bodies,
  Runner,
  Body,
  Events,
} from "matter-js";
import { SearchClusterTopics } from "shared/types";
import { createStore, unwrap } from "solid-js/store";

interface TrendExplorerCanvas {
  topics: SearchClusterTopics[];
}

// Get a shade of gray
const getColorFromDensity = (density: number) => {
  const color = Math.floor(255 - 70 * density);
  return `rgb(${color}, ${color}, ${color})`;
};

export const TrendExplorerCanvas = (props: TrendExplorerCanvas) => {
  const [canvasElement, setCanvasElement] = createSignal<HTMLCanvasElement>();

  const [containerSize, setContainerSize] = createStore({
    width: 700,
    height: 800,
  });

  // Subscribe with resize observer
  onMount(() => {
    const observer = new ResizeObserver((entries) => {
      const entry = entries[0];
      setContainerSize({
        width: entry.contentRect.width,
        height: entry.contentRect.height,
      });
    });
    if (canvasElement() !== undefined) {
      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      observer.observe(canvasElement()!);
    }
    return () => observer.disconnect();
  });

  const engine = Engine.create({
    gravity: {
      scale: 0,
    },
  });

  createEffect(() => {
    console.log("updating");
    const render = Render.create({
      canvas: canvasElement(),
      engine: engine,
      options: {
        background: "#f5f5f5",
        showIds: true,
        height: 800,
        width: 700,
        wireframes: false,
      },
    });

    const circles = props.topics.map((topic) => {
      const circle = Bodies.circle(
        containerSize.width / 2,
        containerSize.height / 2,
        1 * topic.density,
      );
      // @ts-expect-error just debugging
      circle.id = topic.topic;
      // Make the circle gray
      circle.render.fillStyle = getColorFromDensity(topic.avg_score);
      // Add a border
      circle.render.strokeStyle = "#333";
      circle.render.lineWidth = 1;
      return circle;
    });

    Composite.add(engine.world, [...circles]);

    Render.run(render);

    const runner = Runner.create();

    Events.on(runner, "beforeTick", () => {
      // Pull the circles towards the center
      circles.forEach((circle) => {
        const x = circle.position.x;
        const y = circle.position.y;
        let dx = unwrap(containerSize).width / 2 - x;
        dx += Math.random() * 8 - 1;
        const dy = unwrap(containerSize).height / 2 - y;
        const angle = Math.atan2(dy, dx);
        const force = 0.001 * circle.density;
        const fx = Math.cos(angle) * force;
        const fy = Math.sin(angle) * force;
        Body.applyForce(circle, { x: x, y: y }, { x: fx, y: fy });
      });
    });

    Runner.run(runner, engine);

    return () => {
      console.log("cleaning up");
      Render.stop(render);
      Runner.stop(runner);
      Engine.clear(engine);
    };
  });

  return (
    <canvas
      style={{
        border: "1px solid red",
        width: "100%",
        height: "100%",
      }}
      ref={setCanvasElement}
    />
  );
};
