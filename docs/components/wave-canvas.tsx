"use client";

import { useEffect, useRef } from "react";

const BACKING_WIDTH = 949;
const BACKING_HEIGHT = 500;
const LINE_SPACING = 22;
const SIDE_RAIL_WIDTH = 92;
const AMPLITUDE = 10;
const SECONDARY_AMPLITUDE = 4;
const WAVE_LENGTH = 172;
const SECONDARY_WAVE_LENGTH = 311;
const HORIZONTAL_PHASE = 0.021;
const FALL_PIXELS_PER_MS = 0.018;
const DRIFT_PHASE_PER_MS = 0.00025;
const LINE_WIDTH = 2.15;
const FALLBACK_STROKE_COLOR = "rgba(0, 0, 0, 0.72)";
const SAMPLE_STEP = 7;

export function WaveCanvas() {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = canvasRef.current;

    if (!canvas) {
      return undefined;
    }

    const context = canvas.getContext("2d");
    const prefersReducedMotion = window.matchMedia("(prefers-reduced-motion: reduce)");

    if (!context) {
      return undefined;
    }

    const activeCanvas = canvas;
    const drawingContext = context;
    let animationFrame = 0;

    function initializeCanvas() {
      if (activeCanvas.width !== BACKING_WIDTH) {
        activeCanvas.width = BACKING_WIDTH;
      }

      if (activeCanvas.height !== BACKING_HEIGHT) {
        activeCanvas.height = BACKING_HEIGHT;
      }

      drawingContext.setTransform(1, 0, 0, 1, 0, 0);
    }

    function drawWaterfallLine(startX: number, panelHeight: number, phase: number, fallOffset: number) {
      drawingContext.beginPath();

      for (let y = -SAMPLE_STEP; y <= panelHeight + SAMPLE_STEP; y += SAMPLE_STEP) {
        const streamY = y + fallOffset;
        const primaryOffset = Math.sin(streamY / WAVE_LENGTH + phase) * AMPLITUDE;
        const secondaryOffset =
          Math.sin(streamY / SECONDARY_WAVE_LENGTH + phase * 1.8) * SECONDARY_AMPLITUDE;
        const depthTaper = 0.76 + (Math.max(y, 0) / panelHeight) * 0.24;
        const x = startX + (primaryOffset + secondaryOffset) * depthTaper;

        if (y === -SAMPLE_STEP) {
          drawingContext.moveTo(x, y);
        } else {
          drawingContext.lineTo(x, y);
        }
      }

      drawingContext.stroke();
    }

    function draw(timestamp: number) {
      initializeCanvas();

      const panelWidth = activeCanvas.width;
      const panelHeight = activeCanvas.height;
      const startX = SIDE_RAIL_WIDTH;
      const endX = panelWidth - SIDE_RAIL_WIDTH;
      const fallOffset = prefersReducedMotion.matches ? 0 : timestamp * FALL_PIXELS_PER_MS;
      const driftPhase = prefersReducedMotion.matches ? 0 : timestamp * DRIFT_PHASE_PER_MS;

      drawingContext.clearRect(0, 0, panelWidth, panelHeight);
      drawingContext.strokeStyle =
        window.getComputedStyle(activeCanvas).getPropertyValue("--wave-stroke").trim() || FALLBACK_STROKE_COLOR;
      drawingContext.lineWidth = LINE_WIDTH;
      drawingContext.lineCap = "round";
      drawingContext.lineJoin = "round";

      for (let x = startX - LINE_SPACING; x <= endX + LINE_SPACING; x += LINE_SPACING) {
        drawWaterfallLine(x, panelHeight, x * HORIZONTAL_PHASE + driftPhase, fallOffset);
      }

      if (!prefersReducedMotion.matches) {
        animationFrame = window.requestAnimationFrame(draw);
      }
    }

    function redraw() {
      window.cancelAnimationFrame(animationFrame);
      animationFrame = window.requestAnimationFrame(draw);
    }

    prefersReducedMotion.addEventListener("change", redraw);
    window.addEventListener("resize", redraw, { passive: true });
    const themeObserver = new MutationObserver(redraw);
    themeObserver.observe(document.documentElement, { attributes: true, attributeFilter: ["data-theme"] });
    redraw();

    return () => {
      prefersReducedMotion.removeEventListener("change", redraw);
      window.removeEventListener("resize", redraw);
      themeObserver.disconnect();
      window.cancelAnimationFrame(animationFrame);
    };
  }, []);

  return (
    <canvas
      ref={canvasRef}
      id="waveCanvas"
      className="absolute inset-0 pointer-events-none opacity-40"
      width={BACKING_WIDTH}
      height={BACKING_HEIGHT}
    />
  );
}
