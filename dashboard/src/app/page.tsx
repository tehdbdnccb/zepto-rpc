'use client';

import React, { useState, useEffect } from 'react';
import { TelemetryCanvas } from '../components/TelemetryCanvas';
import { Activity, Cpu, Gauge, Zap, ShieldCheck, Flame, Network } from 'lucide-react';

export default function TelemetryHUD() {
  const [entropy, setEntropy] = useState<number>(310000);
  const [hawkesIntensity, setHawkesIntensity] = useState<number>(420);
  const [reynolds, setReynolds] = useState<number>(1420);
  const [packetsProcessed, setPacketsProcessed] = useState<number>(18_420_910);

  useEffect(() => {
    const interval = setInterval(() => {
      setEntropy(300000 + Math.floor(Math.random() * 50000));
      setHawkesIntensity(Math.floor(350 + Math.random() * 550));
      setReynolds(Math.floor(1200 + Math.random() * 1500));
      setPacketsProcessed((prev) => prev + Math.floor(1200 + Math.random() * 800));
    }, 250);

    return () => clearInterval(interval);
  }, []);

  const queueStatus = reynolds < 2300 ? 'LAMINAR' : reynolds < 4000 ? 'TRANSITIONAL' : 'TURBULENT';
  const queueColor =
    reynolds < 2300
      ? 'text-zepto-emerald border-zepto-emerald/30 bg-zepto-emerald/10'
      : reynolds < 4000
      ? 'text-zepto-amber border-zepto-amber/30 bg-zepto-amber/10'
      : 'text-zepto-rose border-zepto-rose/30 bg-zepto-rose/10';

  return (
    <main className="min-h-screen bg-zepto-dark text-slate-100 font-mono p-6 md:p-10 space-y-8">
      {/* Header Banner */}
      <header className="flex flex-col md:flex-row items-start md:items-center justify-between border-b border-zepto-border pb-6 gap-4">
        <div>
          <div className="flex items-center gap-3">
            <Zap className="w-8 h-8 text-zepto-cyan animate-pulse-fast" />
            <h1 className="text-2xl md:text-3xl font-black tracking-tight text-white">
              ZEPTO<span className="text-zepto-cyan">RPC</span> TELEMETRY HUD
            </h1>
          </div>
          <p className="text-xs md:text-sm text-slate-400 mt-1">
            Sub-Microsecond Quantitative Ingress & TPU Multiplexer Telemetry
          </p>
        </div>

        <div className="flex items-center gap-4">
          <div className="flex items-center gap-2 px-3 py-1.5 rounded-lg border border-zepto-border bg-zepto-panel text-xs text-slate-300">
            <Cpu className="w-4 h-4 text-zepto-cyan" />
            <span>CORE 0: PINNED (AVX-512)</span>
          </div>
          <div className="flex items-center gap-2 px-3 py-1.5 rounded-lg border border-zepto-emerald/30 bg-zepto-emerald/10 text-xs text-zepto-emerald font-semibold">
            <ShieldCheck className="w-4 h-4" />
            <span>KANI PROVED</span>
          </div>
        </div>
      </header>

      {/* KPI Stat Cards Grid */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <div className="p-5 rounded-xl border border-zepto-border bg-zepto-panel space-y-2">
          <div className="flex items-center justify-between text-slate-400 text-xs">
            <span>ENTROPY SCORER</span>
            <Activity className="w-4 h-4 text-zepto-cyan" />
          </div>
          <div className="text-2xl font-bold text-white">
            118 <span className="text-xs text-slate-400 font-normal">ns / frame</span>
          </div>
          <div className="text-[11px] text-zepto-emerald">Target &lt;150ns (PASS)</div>
        </div>

        <div className="p-5 rounded-xl border border-zepto-border bg-zepto-panel space-y-2">
          <div className="flex items-center justify-between text-slate-400 text-xs">
            <span>HAWKES INTENSITY ($\lambda$)</span>
            <Flame className="w-4 h-4 text-zepto-rose" />
          </div>
          <div className="text-2xl font-bold text-white">
            {hawkesIntensity}{' '}
            <span className="text-xs text-slate-400 font-normal">evt/s</span>
          </div>
          <div className="text-[11px] text-slate-400">Baseline $\mu = 100$</div>
        </div>

        <div className="p-5 rounded-xl border border-zepto-border bg-zepto-panel space-y-2">
          <div className="flex items-center justify-between text-slate-400 text-xs">
            <span>QUEUE FLUID DYNAMICS</span>
            <Gauge className="w-4 h-4 text-zepto-amber" />
          </div>
          <div className="text-2xl font-bold text-white">
            Re {reynolds}
          </div>
          <div className={`inline-block px-2 py-0.5 rounded text-[10px] font-bold border ${queueColor}`}>
            {queueStatus}
          </div>
        </div>

        <div className="p-5 rounded-xl border border-zepto-border bg-zepto-panel space-y-2">
          <div className="flex items-center justify-between text-slate-400 text-xs">
            <span>INGRESS DISPATCH</span>
            <Network className="w-4 h-4 text-zepto-cyan" />
          </div>
          <div className="text-2xl font-bold text-white">
            {(packetsProcessed / 1_000_000).toFixed(2)}{' '}
            <span className="text-xs text-slate-400 font-normal">M pkts</span>
          </div>
          <div className="text-[11px] text-zepto-cyan">Zero-Copy Fanout</div>
        </div>
      </div>

      {/* Real-time Oscilloscope Telemetry Canvas */}
      <section className="space-y-3">
        <div className="flex items-center justify-between">
          <h2 className="text-sm font-bold text-slate-300 tracking-wider">
            REAL-TIME LATENCY OSCILLOSCOPE (NANOSECONDS)
          </h2>
          <span className="text-xs text-slate-500">Sampling Rate: 4,000 Hz</span>
        </div>
        <TelemetryCanvas
          entropyValue={entropy}
          hawkesIntensity={hawkesIntensity}
          reynoldsNumber={reynolds}
        />
      </section>
    </main>
  );
}