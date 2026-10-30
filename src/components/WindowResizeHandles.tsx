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
  const handleResize = (direction: ResizeDirection) => (e: React.MouseEvent) => {
    if (e.button !== 0) return;
    e.preventDefault();
    e.stopPropagation();
    triggerResize(direction);
  };

  return (
    <>
      {/* Top Edge */}
      <div
        data-resize-handle
        onMouseDown={handleResize(ResizeDirection.North)}
        className="absolute top-0 left-3 right-3 h-2 z-50 select-none cursor-ns-resize"
        style={{ cursor: 'ns-resize' }}
        title="Resize"
      />
      {/* Bottom Edge */}
      <div
        data-resize-handle
        onMouseDown={handleResize(ResizeDirection.South)}
        className="absolute bottom-0 left-3 right-3 h-2 z-50 select-none cursor-ns-resize"
        style={{ cursor: 'ns-resize' }}
        title="Resize"
      />
      {/* Left Edge */}
      <div
        data-resize-handle
        onMouseDown={handleResize(ResizeDirection.West)}
        className="absolute left-0 top-3 bottom-3 w-2 z-50 select-none cursor-ew-resize"
        style={{ cursor: 'ew-resize' }}
        title="Resize"
      />
      {/* Right Edge */}
      <div
        data-resize-handle
        onMouseDown={handleResize(ResizeDirection.East)}
        className="absolute right-0 top-3 bottom-3 w-2 z-50 select-none cursor-ew-resize"
        style={{ cursor: 'ew-resize' }}
        title="Resize"
      />
      {/* Top-Left Corner */}
      <div
        data-resize-handle
        onMouseDown={handleResize(ResizeDirection.NorthWest)}
        className="absolute top-0 left-0 w-3.5 h-3.5 z-50 select-none cursor-nwse-resize"
        style={{ cursor: 'nwse-resize' }}
        title="Resize"
      />
      {/* Top-Right Corner */}
      <div
        data-resize-handle
        onMouseDown={handleResize(ResizeDirection.NorthEast)}
        className="absolute top-0 right-0 w-3.5 h-3.5 z-50 select-none cursor-nesw-resize"
        style={{ cursor: 'nesw-resize' }}
        title="Resize"
      />
      {/* Bottom-Left Corner */}
      <div
        data-resize-handle
        onMouseDown={handleResize(ResizeDirection.SouthWest)}
        className="absolute bottom-0 left-0 w-3.5 h-3.5 z-50 select-none cursor-nesw-resize"
        style={{ cursor: 'nesw-resize' }}
        title="Resize"
      />
      {/* Bottom-Right Corner */}
      <div
        data-resize-handle
        onMouseDown={handleResize(ResizeDirection.SouthEast)}
        className="absolute bottom-0 right-0 w-3.5 h-3.5 z-50 select-none cursor-nwse-resize"
        style={{ cursor: 'nwse-resize' }}
        title="Resize"
      />
    </>
  );
};
