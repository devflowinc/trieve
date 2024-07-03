import { createEffect, createSignal, on, onMount } from "solid-js";
import { colord, extend } from "colord";
import mixPlugin from "colord/plugins/mix";
extend([mixPlugin]);
import {
  Composite,
  Engine,
  Render,
  Bodies,
  Runner,
  Body,
  Events,
  Mouse,
  MouseConstraint,
} from "matter-js";
import { SearchClusterTopics } from "shared/types";
import { createStore } from "solid-js/store";
import Matter from "matter-js";

interface TrendExplorerCanvasProps {
  topics: SearchClusterTopics[];
  onSelectTopic: (topicId: string) => void;
}

const getColorFromDensity = (density: number) => {
  // Mix white with a deep purple color
  const color = colord("#914fc2") // Deep purple
    .lighten(density * 0.082) // Mix with white
    .toRgbString(); // Convert to RGB string

  return color;
};

const centeredRandom = (factor: number) => {
  return Math.random() * factor - factor / 2;
};

export const TrendExplorerCanvas = (props: TrendExplorerCanvasProps) => {
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
        height: 1600,
        width: 700,
        wireframes: false,
      },
    });

    const circles = props.topics.map((topic) => {
      const circle = Bodies.circle(
        centeredRandom(3) + 400,
        centeredRandom(3) + 400,
        Math.max(1.2 * topic.density, 30),
        {
          id: topic.density,
          label: topic.topic,
        },
      );
      // @ts-expect-error just debugging
      circle.id = topic.id;
      circle.render.fillStyle = getColorFromDensity(topic.avg_score);
      circle.render.strokeStyle = "#333";
      circle.render.lineWidth = 1;
      circle.timeScale = 0.2;
      circle.friction = 0.9999;

      return circle;
    });

    Composite.add(engine.world, [...circles]);

    const response = Events.on(runner, "beforeTick", () => {
      // Pull the circles towards the point (400, 400)
      circles.forEach((circle) => {
        const x = circle.position.x;
        const y = circle.position.y;
        const targetX = 400;
        const targetY = 400;

        // Calculate the difference between current position and target
        const dx = targetX - x;
        const dy = targetY - y;

        // Apply force proportional to the distance from the target point
        const forceMagnitude = 0.00025; // Adjust this value to control the strength of attraction
        const fx = dx * forceMagnitude;
        const fy = dy * forceMagnitude;

        Body.applyForce(circle, { x: x, y: y }, { x: fx, y: fy });
      });
    });

    const mouse = Mouse.create(render.canvas);
    const mouseConstraint = MouseConstraint.create(engine, {
      mouse: mouse,
      constraint: {
        stiffness: 0.2,
        render: {
          visible: false,
        },
      },
    });

    // eslint-disable-next-line solid/reactivity
    Events.on(mouseConstraint, "mousedown", (event) => {
      const mousePosition = event.mouse.position;
      const bodiesUnderMouse = Matter.Query.point(circles, mousePosition);

      if (bodiesUnderMouse.length > 0) {
        const clickedCircle = bodiesUnderMouse[0];
        const topicId = clickedCircle.id;
        // @ts-expect-error accessing custom property
        props.onSelectTopic(topicId);
      }
    });

    // eslint-disable-next-line solid/reactivity
    Events.on(render, "afterRender", function () {
      const ctx = render.context;
      circles.forEach((circle) => {
        ctx.font = "12px Arial";
        ctx.fillStyle = "black";
        ctx.textAlign = "center";
        ctx.textBaseline = "middle";

        const density =
          props.topics.find((t) => t.id === (circle.id as unknown as string))
            ?.density || 0;

        let label = circle.label; // Truncate long labels
        if (density < 50) {
          ctx.font = "10px Arial";
          label = label.substring(0, 10) + "...";
        }
        if (density < 80) {
          // Replace spaces with newlines
          label = label.replace(/ /g, "\n");
        }
        if (density > 80) {
          ctx.font = "14px Arial";
        }
        ctx.fillText(label, circle.position.x, circle.position.y);
      });
    });

    Composite.add(engine.world, mouseConstraint);

    // Ensure the mouse captures events even when outside the canvas
    render.mouse = mouse;

    // center the camera on (0, 0)
    setRender(render);

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
        width: "100%",
        height: "100%",
      }}
      ref={setCanvasElement}
    />
  );
};
