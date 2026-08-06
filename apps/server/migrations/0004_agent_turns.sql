-- Agent conversation turns, as a TREE rather than a list.
--
-- `parent_id` is the turn this one continues from. Sending a message while an
-- older turn is selected forks a branch instead of overwriting history, so a
-- conversation can be rewound and explored down more than one path. The root
-- of a conversation has parent_id NULL.
CREATE TABLE agent_turns (
    id          uuid PRIMARY KEY,
    doc_id      uuid NOT NULL REFERENCES docs (id) ON DELETE CASCADE,
    user_id     uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    parent_id   uuid REFERENCES agent_turns (id) ON DELETE CASCADE,
    prompt      text NOT NULL,
    answer      text,
    status      text NOT NULL DEFAULT 'running',
    error       text,
    created_at  timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX agent_turns_doc_idx ON agent_turns (doc_id, created_at);
CREATE INDEX agent_turns_parent_idx ON agent_turns (parent_id);
