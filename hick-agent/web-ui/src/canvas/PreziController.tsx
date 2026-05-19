import { useCallback, useRef, useState } from 'react'
import type { BranchId, BranchMeta } from './treeModel'

export const COLUMN_WIDTH = 560
export const COLUMN_GAP = 48

/** Left-edge x coordinate of the column with the given index in the sorted branch row. */
export function columnFlexOffset(
  columnIndex: number,
  sortedBranches: BranchMeta[],
): number {
  const pos = sortedBranches.findIndex((b) => b.columnIndex === columnIndex)
  return pos < 0 ? 0 : pos * (COLUMN_WIDTH + COLUMN_GAP)
}

export function usePreziController(
  activeBranchId: BranchId,
  branches: BranchMeta[],
): {
  navigateToBranch: (id: BranchId) => void
  isTransitioning: boolean
  /** Apply to the 3D-tilt wrapper (the outer div). */
  tiltStyle: React.CSSProperties
  /** Apply to the flex row (translateX to center the active column). */
  rowStyle: React.CSSProperties
} {
  const sortedBranches = [...branches].sort((a, b) => a.columnIndex - b.columnIndex)

  const activeBranch = branches.find((b) => b.id === activeBranchId)
  const activeFlexOffset = activeBranch
    ? columnFlexOffset(activeBranch.columnIndex, sortedBranches)
    : 0

  const [isTransitioning, setIsTransitioning] = useState(false)
  const [tiltStyle, setTiltStyle] = useState<React.CSSProperties>({
    transition: 'transform 200ms ease-in-out',
    transform: 'perspective(1200px) rotateX(0deg) rotateY(0deg)',
  })
  const [rowStyle, setRowStyle] = useState<React.CSSProperties>({
    transform: `translateX(-${activeFlexOffset}px)`,
    transition: 'transform 200ms ease-in-out',
  })

  const cancelRef = useRef<(() => void) | null>(null)

  const navigateToBranch = useCallback(
    (id: BranchId) => {
      if (cancelRef.current) {
        cancelRef.current()
        cancelRef.current = null
      }

      const target = branches.find((b) => b.id === id)
      if (!target) return

      const currentBranch = branches.find((b) => b.id === activeBranchId)
      const currentColIdx = currentBranch?.columnIndex ?? 0
      const targetColIdx = target.columnIndex
      const direction = targetColIdx > currentColIdx ? 1 : targetColIdx < currentColIdx ? -1 : 0

      const targetOffset = columnFlexOffset(targetColIdx, sortedBranches)

      let cancelled = false
      const timers: ReturnType<typeof setTimeout>[] = []
      cancelRef.current = () => {
        cancelled = true
        timers.forEach(clearTimeout)
      }

      const delay = (ms: number) =>
        new Promise<void>((resolve) => {
          const t = setTimeout(() => {
            if (!cancelled) resolve()
          }, ms)
          timers.push(t)
        })

      setIsTransitioning(true)

      const run = async () => {
        // Phase 1 — zoom out + tilt (~200ms)
        setTiltStyle({
          transition: 'transform 200ms ease-in-out',
          transform: `perspective(1200px) rotateX(6deg) rotateY(${direction * 4}deg) scale(0.92)`,
        })
        setRowStyle({
          transform: `translateX(-${activeFlexOffset}px)`,
          transition: 'transform 200ms ease-in-out',
        })
        await delay(200)
        if (cancelled) return

        // Phase 2 — pan to target (~200ms)
        setRowStyle({
          transform: `translateX(-${targetOffset}px)`,
          transition: 'transform 200ms ease-in-out',
        })
        await delay(200)
        if (cancelled) return

        // Phase 3 — zoom in + flatten tilt (~200ms)
        setTiltStyle({
          transition: 'transform 200ms ease-in-out',
          transform: 'perspective(1200px) rotateX(0deg) rotateY(0deg) scale(1)',
        })
        await delay(200)

        if (!cancelled) setIsTransitioning(false)
      }

      run()
    },
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [branches, activeBranchId, activeFlexOffset, sortedBranches],
  )

  return { navigateToBranch, isTransitioning, tiltStyle, rowStyle }
}
