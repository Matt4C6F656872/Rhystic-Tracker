import React from 'react';
import { getCurrentWindow } from '@tauri-apps/api/window';

export const ResizeDirection = {
  North: 'North',
  South: 'South',
  East: 'East',
  West: 'West',
  NorthEast: 'NorthEast',
  NorthWest: 'NorthWest',
  SouthEast: 'SouthEast',
  SouthWest: 'SouthWest',
} as const;

export type ResizeDirection = typeof ResizeDirection[keyof typeof ResizeDirection];

export const triggerResize = (direction: ResizeDirection) => {
  const win = getCurrentWindow() as any;
  if (typeof win.startResizing === 'function') {
    win.startResizing(direction).catch(console.error);
  } else if (typeof win.startResizeDragging === 'function') {
    win.startResizeDragging(direction).catch(console.error);
  }
};

export const WindowResizeHandles: React.FC = () => {
  const handlePointerDown = (direction: ResizeDirection) => (e: React.PointerEvent) => {
    if (e.button !== 0) return;
    e.preventDefault();
    e.stopPropagation();
    const win = getCurrentWindow() as any;
    if (typeof win.startResizing === 'function') {
      win.startResizing(direction).catch(console.error);
    } else if (typeof win.startResizeDragging === 'function') {
      win.startResizeDragging(direction).catch(console.error);
    }
  };

  return (
    <>
      {/* Top Edge (8px thick) */}
      <div
        data-resize-handle
        onPointerDown={handlePointerDown(ResizeDirection.North)}
        className="absolute top-0 left-2 right-2 h-2 z-50 select-none cursor-ns-resize"
        style={{ cursor: 'ns-resize' }}
        title="Resize"
      />
      {/* Bottom Edge (8px thick) */}
      <div
        data-resize-handle
        onPointerDown={handlePointerDown(ResizeDirection.South)}
        className="absolute bottom-0 left-2 right-2 h-2 z-50 select-none cursor-ns-resize"
        style={{ cursor: 'ns-resize' }}
        title="Resize"
      />
      {/* Left Edge (8px thick) */}
      <div
        data-resize-handle
        onPointerDown={handlePointerDown(ResizeDirection.West)}
        className="absolute left-0 top-2 bottom-2 w-2 z-50 select-none cursor-ew-resize"
        style={{ cursor: 'ew-resize' }}
        title="Resize"
      />
      {/* Right Edge (8px thick) */}
      <div
        data-resize-handle
        onPointerDown={handlePointerDown(ResizeDirection.East)}
        className="absolute right-0 top-2 bottom-2 w-2 z-50 select-none cursor-ew-resize"
        style={{ cursor: 'ew-resize' }}
        title="Resize"
      />
      {/* Top-Left Corner (8x8px) */}
      <div
        data-resize-handle
        onPointerDown={handlePointerDown(ResizeDirection.NorthWest)}
        className="absolute top-0 left-0 w-2 h-2 z-50 select-none cursor-nwse-resize"
        style={{ cursor: 'nwse-resize' }}
        title="Resize"
      />
      {/* Top-Right Corner (8x8px) */}
      <div
        data-resize-handle
        onPointerDown={handlePointerDown(ResizeDirection.NorthEast)}
        className="absolute top-0 right-0 w-2 h-2 z-50 select-none cursor-nesw-resize"
        style={{ cursor: 'nesw-resize' }}
        title="Resize"
      />
      {/* Bottom-Left Corner (8x8px) */}
      <div
        data-resize-handle
        onPointerDown={handlePointerDown(ResizeDirection.SouthWest)}
        className="absolute bottom-0 left-0 w-2 h-2 z-50 select-none cursor-nesw-resize"
        style={{ cursor: 'nesw-resize' }}
        title="Resize"
      />
      {/* Bottom-Right Corner (8x8px) */}
      <div
        data-resize-handle
        onPointerDown={handlePointerDown(ResizeDirection.SouthEast)}
        className="absolute bottom-0 right-0 w-2 h-2 z-50 select-none cursor-nwse-resize"
        style={{ cursor: 'nwse-resize' }}
        title="Resize"
      />
    </>
  );
};
