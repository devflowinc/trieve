import { createEffect, createSignal, on, onMount } from "solid-js";
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

const centeredRandom = (factor: number) => {
  return Math.random() * factor - factor / 2;
};

export const TrendExplorerCanvas = (props: TrendExplorerCanvas) => {
  const [canvasElement, setCanvasElement] = createSignal<HTMLCanvasElement>();
  const [render, setRender] = createSignal<Render | null>(null);

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

  const runner = Runner.create();

  createEffect(
    on(
      () => containerSize.width,
      () => {
        // Set the render options to the size of the container
        const localRender = render();
        if (localRender === null) {
          return;
        }
        // Update the canvas size
        localRender.canvas.width = containerSize.width;
        localRender.canvas.height = containerSize.height;
      },
    ),
  );

  createEffect(() => {
    console.log("updating");
    const render = Render.create({
      canvas: canvasElement(),
      engine: engine,
      options: {
        background: "#f5f5f5",
        height: 800,
        width: 700,
        wireframes: false,
      },
    });

    const circles = props.topics.map((topic) => {
      const circle = Bodies.circle(
        centeredRandom(3),
        centeredRandom(3),
        1 * topic.density,
      );
      // @ts-expect-error just debugging
      circle.id = topic.topic;
      circle.render.fillStyle = getColorFromDensity(topic.avg_score);
      circle.render.strokeStyle = "#333";
      circle.render.lineWidth = 1;
      circle.timeScale = 0.2;
      circle.friction = 0.9999;
      circle.density = 0.9999;

      return circle;
    });

    Composite.add(engine.world, [...circles]);

    const response = Events.on(runner, "beforeTick", () => {
      // Pull the circles towards the center
      circles.forEach((circle) => {
        const x = circle.position.x;
        const y = circle.position.y;
        const fx = -0.0005 * x * 0.5;
        const fy = -0.0005 * y * 0.5;

        Body.applyForce(circle, { x: x, y: y }, { x: fx, y: fy });
      });
    });

    // center the camera on (0, 0)
    setRender(render);

    // console
    Render.lookAt(render, {
      min: { x: -containerSize.width / 2, y: -containerSize.height / 2 },
      max: { x: containerSize.width / 2, y: containerSize.height / 2 },
    });

    Render.run(render);

    Runner.run(runner, engine);

    return () => {
      console.log("cleaning up");
      response();
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
