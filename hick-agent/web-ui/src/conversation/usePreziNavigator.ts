import { animate } from 'framer-motion'
import type { ReactFlowInstance, Viewport } from '@xyflow/react'
import { useCallback, useRef } from 'react'

const EASE: [number, number, number, number] = [0.4, 0, 0.2, 1]

function lerp(a: number, b: number, u: number): number {
  return a + (b - a) * u
}

/**
 * Three-phase Prezi-style viewport transition (~600ms, interruptible).
 * Tilt (degrees) is reported to the caller for CSS on a wrapper outside React Flow.
 */
export function usePreziNavigator(
  rf: ReactFlowInstance | null,
  getContainer: () => HTMLElement | null,
  setTilt: (rx: number, ryDeg: number) => void,
) {
  const stopRef = useRef<(() => void) | null>(null)

  const cancel = useCallback(() => {
    stopRef.current?.()
    stopRef.current = null
    setTilt(0, 0)
  }, [setTilt])

  const navigate = useCallback(
    (target: Viewport, directionSign: number) => {
      if (!rf) return
      cancel()
      const start = rf.getViewport()

      const ctrl = animate(0, 1, {
        duration: 0.6,
        ease: EASE,
        onUpdate: (t) => {
          let zoom: number
          let x: number
          let y: number

          if (t <= 1 / 3) {
            const u = t * 3
            zoom = lerp(start.zoom, 0.27, u)
            x = start.x
            y = start.y
            setTilt(lerp(0, 6, u), lerp(0, directionSign * 4, u))
          } else if (t <= 2 / 3) {
            const u = (t - 1 / 3) * 3
            zoom = 0.27
            x = lerp(start.x, target.x, u)
            y = lerp(start.y, target.y, u)
            setTilt(lerp(6, 3, u), lerp(directionSign * 4, directionSign * 2, u))
          } else {
            const u = (t - 2 / 3) * 3
            zoom = lerp(0.27, target.zoom, u)
            x = target.x
            y = target.y
            setTilt(lerp(3, 0, u), lerp(directionSign * 2, 0, u))
          }

          rf.setViewport({ x, y, zoom }, { duration: 0 })
        },
        onComplete: () => {
          setTilt(0, 0)
          stopRef.current = null
        },
      })

      stopRef.current = () => ctrl.stop()
    },
    [rf, cancel, setTilt],
  )

  /** Compute viewport so flow point (cx,cy) centers horizontally & vertically at zoom `zoom`. */
  const viewportCenterOn = useCallback((cx: number, cy: number, zoom: number): Viewport => {
    const containerEl = getContainer()
    if (!containerEl) return { x: 0, y: 0, zoom }
    const vw = containerEl.clientWidth
    const vh = containerEl.clientHeight
    return {
      zoom,
      x: vw / 2 - cx * zoom,
      y: vh / 2 - cy * zoom,
    }
  }, [getContainer])

  return { navigate, cancel, viewportCenterOn }
}
