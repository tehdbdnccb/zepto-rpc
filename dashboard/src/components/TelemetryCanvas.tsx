'use client';

import React, { useEffect, useRef } from 'react';

interface TelemetryCanvasProps {
  entropyValue: number;
  hawkesIntensity: number;
  reynoldsNumber: number;
}

export const TelemetryCanvas: React.FC<TelemetryCanvasProps> = ({
  entropyValue,
  hawkesIntensity,
  reynoldsNumber,
}) => {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const historyRef = useRef<number[]>(new Array(120).fill(110));

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    let animationFrameId: number;

    const render = () => {
      // Shift history and append synthetic jittered latency point
      const history = historyRef.current;
      history.shift();
      const nextPoint = 110 + (Math.sin(Date.now() / 200) * 15) + (Math.random() * 12);
      history.push(nextPoint);

      // DPR scaling for Retina displays
      const dpr = window.devicePixelRatio || 1;
      const rect = canvas.getBoundingClientRect();
      canvas.width = rect.width * dpr;
      canvas.height = rect.height * dpr;
      ctx.scale(dpr, dpr);

      const width = rect.width;
      const height = rect.height;

      // Clear Canvas
      ctx.fillStyle = '#0B0F19';
      ctx.fillRect(0, 0, width, height);

      // Draw Grid Lines
      ctx.strokeStyle = '#1E293B';
      ctx.lineWidth = 1;
      for (let y = 0; y < height; y += 30) {
        ctx.beginPath();
        ctx.moveTo(0, y);
        ctx.lineTo(width, y);
        ctx.stroke();
      }

      // Draw Real-time Latency Sparkline Waveform
      ctx.beginPath();
      ctx.strokeStyle = hawkesIntensity > 800 ? '#F43F5E' : '#06B6D4';
      ctx.lineWidth = 2;
      const stepX = width / (history.length - 1);

      history.forEach((val, index) => {
        const x = index * stepX;
        const y = height - (val / 200) * height;
        if (index === 0) {
          ctx.moveTo(x, y);
        } else {
          ctx.lineTo(x, y);
        }
      });
      ctx.stroke();

      // Draw Gradient Fill under line
      const gradient = ctx.createLinearGradient(0, 0, 0, height);
      gradient.addColorStop(0, hawkesIntensity > 800 ? 'rgba(244, 63, 94, 0.25)' : 'rgba(6, 182, 212, 0.25)');
      gradient.addColorStop(1, 'rgba(11, 15, 25, 0)');
      ctx.fillStyle = gradient;
      ctx.lineTo(width, height);
      ctx.lineTo(0, height);
      ctx.closePath();
      ctx.fill();

      // Render Overlay Text Badges
      ctx.fillStyle = '#94A3B8';
      ctx.font = '11px JetBrains Mono, monospace';
      ctx.fillText(`SW-ENTROPY: ${(entropyValue / 65536).toFixed(3)} bits/byte`, 12, 22);
      ctx.fillText(`HAWKES-INTENSITY: ${hawkesIntensity.toFixed(0)} evt/s`, 12, 38);
      ctx.fillText(`REYNOLDS-NO: ${reynoldsNumber} (${reynoldsNumber < 2300 ? 'LAMINAR' : 'TURBULENT'})`, 12, 54);

      animationFrameId = requestAnimationFrame(render);
    };

    render();

    return () => {
      cancelAnimationFrame(animationFrameId);
    };
  }, [entropyValue, hawkesIntensity, reynoldsNumber]);

  return (
    <div className="relative w-full h-64 rounded-xl border border-zepto-border bg-zepto-panel overflow-hidden p-1 shadow-2xl">
      <canvas ref={canvasRef} className="w-full h-full block" />
    </div>
  );
};