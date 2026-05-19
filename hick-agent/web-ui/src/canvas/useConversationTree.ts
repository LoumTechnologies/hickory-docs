import { useMemo } from 'react'
import type { UiEvent } from '../types'
import { buildTreeModel, type TreeModel } from './treeModel'

export function useConversationTree(events: UiEvent[]): TreeModel {
  return useMemo(() => buildTreeModel(events), [events])
}
