import { Routes, Route } from 'react-router-dom'
import './App.css'
import { AppLayout } from './layout/AppLayout'
import { ChatPage } from './chat/ChatPage'
import { MemoriesPage } from './memories/MemoriesPage'
import { TasksPage } from './tasks/TasksPage'

export default function App() {
  return (
    <Routes>
      <Route element={<AppLayout />}>
        <Route path="/" element={<ChatPage />} />
        <Route path="/c/:conversationId" element={<ChatPage />} />
        <Route path="/memories" element={<MemoriesPage />} />
        <Route path="/tasks" element={<TasksPage />} />
      </Route>
    </Routes>
  )
}
