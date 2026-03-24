/**
 * Pure utility functions for keyboard navigation in lists.
 */

/**
 * Returns the next index when cycling forward (Ctrl+` style navigation).
 * Wraps around from last to first.
 */
export function cycleNext(current: number, length: number): number {
  if (length === 0) return -1;
  if (current < 0) return 0;
  return (current + 1) % length;
}

/**
 * Returns the previous index. Clamps at 0.
 */
export function movePrev(current: number, _length: number): number {
  return Math.max(0, current - 1);
}

/**
 * Returns the next index. Clamps at length - 1.
 */
export function moveNext(current: number, length: number): number {
  if (length === 0) return -1;
  return Math.min(current + 1, length - 1);
}

/**
 * Returns 0 if list is non-empty, otherwise -1.
 * Used to initialise selection when opening a list via keyboard.
 */
export function initialIndex(length: number): number {
  return length > 0 ? 0 : -1;
}
