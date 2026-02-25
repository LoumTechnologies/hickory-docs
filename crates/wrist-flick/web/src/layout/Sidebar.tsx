import { NavLink } from 'react-router-dom'
import { ConversationList } from '../chat/ConversationList'

export function Sidebar() {
  return (
    <aside className="sidebar">
      <div className="sidebar-header">
        <h1 className="sidebar-title">Wrist Flick</h1>
      </div>
      <nav className="sidebar-nav">
        <NavLink to="/" className="sidebar-link" end>
          New Chat
        </NavLink>
        <NavLink to="/memories" className="sidebar-link">
          Memories
        </NavLink>
        <NavLink to="/tasks" className="sidebar-link">
          Tasks
        </NavLink>
      </nav>
      <div className="sidebar-conversations">
        <ConversationList />
      </div>
    </aside>
  )
}
