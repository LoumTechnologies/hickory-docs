import { useCallback, useRef, useState } from 'react'
import { useReactFlow } from '@xyflow/react'
import type { BranchId, BranchMeta } from './treeModel'

export const COLUMN_WIDTH = 560
export const COLUMN_GAP = 48

function columnX(colIndex: number): number {
  return colIndex * (COLUMN_WIDTH + COLUMN_GAP)
}

export function usePreziController(
  _activeBranchId: BranchId,
  branches: BranchMeta[],
): {
  navigateToBranch: (id: BranchId) => void
  isTransitioning: boolean
  tiltStyle: React.CSSProperties
} {
  const { setViewport } = useReactFlow()
  const [isTransitioning, setIsTransitioning] = useState(false)
  const [tiltStyle, setTiltStyle] = useState<React.CSSProperties>({
    transition: 'transform 200ms ease-in-out',
    transform: 'perspective(1200px) rotateX(0deg) rotateY(0deg)',
  })

  const cancelRef = useRef<(() => void) | null>(null)
  const activeIdRef = useRef(_activeBranchId)
  activeIdRef.current = _activeBranchId

  const navigateToBranch = useCallback(
    (id: BranchId) => {
      // Cancel any in-progress transition
      if (cancelRef.current) {
        cancelRef.current()
        cancelRef.current = null
      }

      const target = branches.find((b) => b.id === id)
      if (!target) return

      const currentBranch = branches.find((b) => b.id === activeIdRef.current)
      const currentColIdx = currentBranch?.columnIndex ?? 0
      const targetColIdx = target.columnIndex

      const direction = targetColIdx > currentColIdx ? 1 : targetColIdx < currentColIdx ? -1 : 0

      let cancelled = false
      cancelRef.current = () => {
        cancelled = true
      }

      const delay = (ms: number) =>
        new Promise<void>((resolve, reject) => {
          const t = setTimeout(() => {
            if (cancelled) reject(new Error('cancelled'))
            else resolve()
          }, ms)
          const prevCancel = cancelRef.current
          cancelRef.current = () => {
            clearTimeout(t)
            if (prevCancel) prevCancel()
            reject(new Error('cancelled'))
          }
        })

      setIsTransitioning(true)

      const run = async () => {
        try {
          // Phase 1: zoom out + tilt
          setTiltStyle({
            transition: 'transform 200ms ease-in-out',
            transform: `perspective(1200px) rotateX(6deg) rotateY(${direction * 4}deg)`,
          })
          setViewport({ x: 0, y: 0, zoom: 0.28 }, { duration: 200 })
          await delay(200)

          if (cancelled) return

          // Phase 2: pan to target column
          const targetX = -columnX(targetColIdx)
          setViewport({ x: targetX, y: 0, zoom: 0.28 }, { duration: 200 })
          await delay(200)

          if (cancelled) return

          // Phase 3: zoom in + flatten tilt
          setTiltStyle({
            transition: 'transform 200ms ease-in-out',
            transform: 'perspective(1200px) rotateX(0deg) rotateY(0deg)',
          })
          setViewport({ x: targetX, y: 0, zoom: 1.0 }, { duration: 200 })
          await delay(200)

          if (!cancelled) {
            setIsTransitioning(false)
          }
        } catch {
          // Cancelled — reset tilt
          setTiltStyle({
            transition: 'transform 200ms ease-in-out',
            transform: 'perspective(1200px) rotateX(0deg) rotateY(0deg)',
          })
          setIsTransitioning(false)
        }
      }

      run()
    },
    [branches, setViewport],
  )

  return { navigateToBranch, isTransitioning, tiltStyle }
}
