import { createEffect, createSignal } from "solid-js";
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

interface TrendExplorerCanvas {
  topics: SearchClusterTopics[];
}

// Get a shade of gray
const getColorFromDensity = (density: number) => {
  const color = Math.floor(255 - 70 * density);
  console.log(color);
  return `rgb(${color}, ${color}, ${color})`;
};

export const TrendExplorerCanvas = (props: TrendExplorerCanvas) => {
  const [canvasElement, setCanvasElement] = createSignal<HTMLCanvasElement>();

  createEffect(() => {
    console.log("updating");
    const engine = Engine.create({
      gravity: {
        scale: 0,
      },
    });
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
      const circle = Bodies.circle(300, 300, 1 * topic.density);
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

    Events.on(runner, "beforeTick", (r) => {
      // Pull the circles towards the center
      circles.forEach((circle) => {
        const x = circle.position.x;
        const y = circle.position.y;
        let dx = 300 - x;
        dx += Math.random() * 8 - 1;
        const dy = 300 - y;
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
    <div>
      <div>Trend explorer canvas</div>
      <canvas ref={setCanvasElement} width={700} height={800} />
    </div>
  );
};
