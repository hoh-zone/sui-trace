import { useId } from 'react';

interface SparklineProps {
  data: number[];
  width?: number;
  height?: number;
  stroke?: string;
  fill?: string;
  className?: string;
}

/** Tiny inline SVG sparkline; no chart lib for the header tiles. */
export function Sparkline({
  data,
  width = 120,
  height = 36,
  stroke = 'rgb(var(--accent))',
  fill = 'rgba(92,165,255,0.15)',
  className,
}: SparklineProps) {
  const id = useId();
  if (data.length < 2) {
    return (
      <svg width={width} height={height} className={className}>
        <rect width={width} height={height} fill="transparent" />
      </svg>
    );
  }
  const min = Math.min(...data);
  const max = Math.max(...data, min + 1);
  const stepX = width / (data.length - 1);
  const norm = (v: number) => height - ((v - min) / (max - min)) * (height - 4) - 2;
  const points = data.map((v, i) => `${(i * stepX).toFixed(2)},${norm(v).toFixed(2)}`).join(' ');
  const area = `M0,${height} L${points.replace(/ /g, ' L')} L${width},${height} Z`;
  return (
    <svg width={width} height={height} className={className} preserveAspectRatio="none">
      <defs>
        <linearGradient id={`grad-${id}`} x1="0" x2="0" y1="0" y2="1">
          <stop offset="0%" stopColor={fill} />
          <stop offset="100%" stopColor="rgba(0,0,0,0)" />
        </linearGradient>
      </defs>
      <path d={area} fill={`url(#grad-${id})`} />
      <polyline points={points} fill="none" stroke={stroke} strokeWidth={1.5} strokeLinejoin="round" />
    </svg>
  );
}
